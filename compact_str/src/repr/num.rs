//! Implementations for efficiently converting a number into a [`Repr`]
//!
//! Adapted from the implementation in the `std` library at
//! <https://github.com/rust-lang/rust/blob/b8214dc6c6fc20d0a660fb5700dca9ebf51ebe89/src/libcore/fmt/num.rs#L188-L266>

use core::{mem, num};
#[cfg(target_pointer_width = "32")]
use core::ptr;

use super::traits::IntoRepr;
use super::{InlineBuffer, Repr, LENGTH_MASK, MAX_SIZE};
use crate::ToCompactStringError;

const DEC_DIGITS_LUT: &[u8] = b"\
      0001020304050607080910111213141516171819\
      2021222324252627282930313233343536373839\
      4041424344454647484950515253545556575859\
      6061626364656667686970717273747576777879\
      8081828384858687888990919293949596979899";


/// OR a two-byte digit pair from the LUT into `words` at byte offset `byte_off`.
///
/// The pair may straddle a word boundary (`byte_off % 8 == 7`); the high byte then goes into the
/// next word. Callers only pass `byte_off <= 21`, so `idx + 1 <= 2` stays in bounds.
#[cfg(target_pointer_width = "64")]
#[inline(always)]
fn or_pair(words: &mut [u64; 3], pair: u16, byte_off: usize) {
    debug_assert!(byte_off <= 21);
    let idx = byte_off / 8;
    let sh = (byte_off % 8) * 8;
    words[idx] |= (pair as u64) << sh;
    if sh == 56 {
        words[idx + 1] |= (pair as u64) >> 8;
    }
}

/// OR a single byte into `words` at byte offset `byte_off`.
#[cfg(target_pointer_width = "64")]
#[inline(always)]
fn or_byte(words: &mut [u64; 3], byte: u8, byte_off: usize) {
    words[byte_off / 8] |= (byte as u64) << ((byte_off % 8) * 8);
}

/// Read a two-byte digit pair from the LUT.
#[cfg(target_pointer_width = "64")]
#[inline(always)]
fn read_pair(lut: *const u8, offset: isize) -> u16 {
    // SAFETY: callers pass an even `offset <= 198`, so the two-byte read is within the LUT.
    u16::from_le_bytes(unsafe { *(lut.offset(offset) as *const [u8; 2]) })
}

/// Defines the implementation of [`IntoRepr`] for integer types
macro_rules! impl_IntoRepr {
    ($t:ident, $conv_ty:ident) => {
        impl IntoRepr for $t {
            #[inline]
            fn into_repr(self) -> Result<Repr, ToCompactStringError> {
                // The formatted value is at most 20 characters (`i64::MIN`). On 64-bit that always
                // fits inline, so we can write straight into an `InlineBuffer` and skip the
                // discriminant dispatch of `with_capacity`/`as_mut_ptr`/`set_len`. The only case
                // that doesn't fit is `u64`/`i64` on a 32-bit target (`MAX_SIZE` is 12 there); we
                // hand those off to `itoa` like the 128-bit types.
                //
                // `MAX_LEN` is an upper bound on the formatted length including the sign; it's `<
                // MAX_SIZE` (not `<=`) so the length still fits in the last byte.
                const MAX_LEN: usize = <$t>::MAX.ilog10() as usize + 2;

                if MAX_LEN >= MAX_SIZE {
                    let mut itoa_buf = itoa::Buffer::new();
                    return Ok(Repr::new(itoa_buf.format(self))?);
                }

                // Number of characters to write, including a leading `-` for negatives.
                let num_digits = NumChars::num_chars(self);

                #[allow(unused_comparisons)]
                let is_nonnegative = self >= 0;
                let mut n = if is_nonnegative {
                    self as $conv_ty
                } else {
                    // convert the negative num to positive by summing 1 to it's 2 complement
                    (!(self as $conv_ty)).wrapping_add(1)
                };
                // On 64-bit targets, assemble the digits into the buffer's three 8-byte words
                // in registers; on 32-bit targets keep the byte-buffer path (`MAX_SIZE` is 12
                // there, so the `[u64; 3]` representation doesn't apply).
                #[cfg(target_pointer_width = "64")]
                {
                let mut curr = num_digits;

                // Assemble the digits into the buffer's three 8-byte words in registers instead
                // of writing 1-2 byte stores into a stack buffer: whole-word reads of the
                // resulting `Repr` would straddle those small stores and defeat store-to-load
                // forwarding on x86.
                let mut words = [0u64; 3];
                let lut_ptr = DEC_DIGITS_LUT.as_ptr();

                // need at least 16 bits for the 4-characters-at-a-time to work.
                if mem::size_of::<$t>() >= 2 {
                    // eagerly decode 4 characters at a time
                    while n >= 10000 {
                        let rem = (n % 10000) as isize;
                        n /= 10000;

                        let d1 = (rem / 100) << 1;
                        let d2 = (rem % 100) << 1;
                        curr -= 4;
                        or_pair(&mut words, read_pair(lut_ptr, d1), curr);
                        or_pair(&mut words, read_pair(lut_ptr, d2), curr + 2);
                    }
                }

                // if we reach here numbers are <= 9999, so at most 4 chars long
                let mut n = n as isize; // possibly reduce 64bit math

                // decode 2 more chars, if > 2 chars
                if n >= 100 {
                    let d1 = (n % 100) << 1;
                    n /= 100;
                    curr -= 2;
                    or_pair(&mut words, read_pair(lut_ptr, d1), curr);
                }

                // decode last 1 or 2 chars
                if n < 10 {
                    curr -= 1;
                    or_byte(&mut words, (n as u8) + b'0', curr);
                } else {
                    let d1 = n << 1;
                    curr -= 2;
                    or_pair(&mut words, read_pair(lut_ptr, d1), curr);
                }

                if !is_nonnegative {
                    curr -= 1;
                    or_byte(&mut words, b'-', curr);
                }

                // we should have moved all the way down our buffer
                debug_assert_eq!(curr, 0);

                // `num_digits < MAX_SIZE`, so the length lives in the last byte, distinct from the
                // digits.
                words[2] |= (num_digits as u64 | LENGTH_MASK as u64) << 56;

                // SAFETY: `[u64; 3]` has the same size as the inline buffer, and `to_le` makes the
                // in-memory byte order match on any endianness. The leading `num_digits` bytes are
                // ASCII digits (with an optional `-`), and the last byte is a valid inline length
                // marker.
                let buffer =
                    unsafe {
                    mem::transmute::<[u64; 3], [u8; MAX_SIZE]>([
                        words[0].to_le(),
                        words[1].to_le(),
                        words[2].to_le(),
                    ])
                };
                return Ok(Repr::from_inline(InlineBuffer(buffer)));
                }

                #[cfg(target_pointer_width = "32")]
                {
                let mut buffer = [0u8; MAX_SIZE];
                let mut curr = num_digits as isize;

                let buf_ptr = buffer.as_mut_ptr();
                let lut_ptr = DEC_DIGITS_LUT.as_ptr();

                unsafe {
                    // need at least 16 bits for the 4-characters-at-a-time to work.
                    if mem::size_of::<$t>() >= 2 {
                        // eagerly decode 4 characters at a time
                        while n >= 10000 {
                            let rem = (n % 10000) as isize;
                            n /= 10000;

                            let d1 = (rem / 100) << 1;
                            let d2 = (rem % 100) << 1;
                            curr -= 4;
                            ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
                            ptr::copy_nonoverlapping(
                                lut_ptr.offset(d2),
                                buf_ptr.offset(curr + 2),
                                2,
                            );
                        }
                    }

                    // if we reach here numbers are <= 9999, so at most 4 chars long
                    let mut n = n as isize; // possibly reduce 64bit math

                    // decode 2 more chars, if > 2 chars
                    if n >= 100 {
                        let d1 = (n % 100) << 1;
                        n /= 100;
                        curr -= 2;
                        ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
                    }

                    // decode last 1 or 2 chars
                    if n < 10 {
                        curr -= 1;
                        *buf_ptr.offset(curr) = (n as u8) + b'0';
                    } else {
                        let d1 = n << 1;
                        curr -= 2;
                        ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
                    }

                    if !is_nonnegative {
                        curr -= 1;
                        *buf_ptr.offset(curr) = b'-';
                    }
                }

                // we should have moved all the way down our buffer
                debug_assert_eq!(curr, 0);

                // `num_digits < MAX_SIZE`, so the length lives in the last byte, distinct from the
                // digits.
                buffer[MAX_SIZE - 1] = num_digits as u8 | LENGTH_MASK;

                // SAFETY: the leading `num_digits` bytes are ASCII digits (with an optional `-`),
                // and the last byte is a valid inline length marker.
                return Ok(Repr::from_inline(InlineBuffer(buffer)));
                }
            }
        }
    };
}

impl_IntoRepr!(u8, u32);
impl_IntoRepr!(i8, u32);
impl_IntoRepr!(u16, u32);
impl_IntoRepr!(i16, u32);
impl_IntoRepr!(u32, u32);
impl_IntoRepr!(i32, u32);
impl_IntoRepr!(u64, u64);
impl_IntoRepr!(i64, u64);

#[cfg(target_pointer_width = "32")]
impl_IntoRepr!(usize, u32);
#[cfg(target_pointer_width = "32")]
impl_IntoRepr!(isize, u32);

#[cfg(target_pointer_width = "64")]
impl_IntoRepr!(usize, u64);
#[cfg(target_pointer_width = "64")]
impl_IntoRepr!(isize, u64);

/// For 128-bit integer types we use the [`itoa`] crate because writing into a buffer, and then
/// copying the amount of characters we've written, is faster than determining the number of
/// characters and then writing.
impl IntoRepr for u128 {
    #[inline]
    fn into_repr(self) -> Result<Repr, ToCompactStringError> {
        let mut buffer = itoa::Buffer::new();
        Ok(Repr::new(buffer.format(self))?)
    }
}

impl IntoRepr for i128 {
    #[inline]
    fn into_repr(self) -> Result<Repr, ToCompactStringError> {
        let mut buffer = itoa::Buffer::new();
        Ok(Repr::new(buffer.format(self))?)
    }
}

/// Defines the implementation of [`IntoRepr`] for NonZero integer types
macro_rules! impl_NonZero_IntoRepr {
    ($t:path) => {
        impl IntoRepr for $t {
            #[inline]
            fn into_repr(self) -> Result<Repr, ToCompactStringError> {
                self.get().into_repr()
            }
        }
    };
}

impl_NonZero_IntoRepr!(num::NonZeroU8);
impl_NonZero_IntoRepr!(num::NonZeroI8);
impl_NonZero_IntoRepr!(num::NonZeroU16);
impl_NonZero_IntoRepr!(num::NonZeroI16);
impl_NonZero_IntoRepr!(num::NonZeroU32);
impl_NonZero_IntoRepr!(num::NonZeroI32);
impl_NonZero_IntoRepr!(num::NonZeroU64);
impl_NonZero_IntoRepr!(num::NonZeroI64);
impl_NonZero_IntoRepr!(num::NonZeroUsize);
impl_NonZero_IntoRepr!(num::NonZeroIsize);
impl_NonZero_IntoRepr!(num::NonZeroU128);
impl_NonZero_IntoRepr!(num::NonZeroI128);

/// All of these `num_chars(...)` methods are kind of crazy, but they are necessary.
///
/// An alternate way to calculate the number of digits in a value is to do:
/// ```no_run
/// let val = 42;
/// let num_digits = ((val as f32).log10().floor()) as usize + 1;
/// assert_eq!(num_digits, 2);
/// ```
/// But there are two problems with this approach:
/// 1. floating point math is slow
/// 2. results are dependent on floating point precision, which is too inaccurate for larger values
///
/// For example, consider this relatively large value...
///
/// ```no_run
/// let val = 9999995;
/// let num_digits = ((val as f32).log10().floor()) as usize + 1;
///
/// // this is wrong! There are only 7 digits in this number!
/// assert_eq!(num_digits, 8);
/// ```
///
/// you can use `f64` to get better precision, e.g.
///
/// ```no_run
/// let val = 9999995;
/// let num_digits = ((val as f64).log10().floor()) as usize + 1;
///
/// // the precision is enough to get the correct value
/// assert_eq!(num_digits, 7);
/// ```
///
/// ...but still not precise enough!
///
/// ```no_run
/// let val: u64 = 9999999999999999999;
/// let num_digits = ((val as f64).log10().floor()) as usize + 1;
///
/// // this is wrong! the number is only 19 digits but the formula returns 20
/// assert_eq!(num_digits, 20);
/// ```
trait NumChars {
    fn num_chars(val: Self) -> usize;
}

impl NumChars for u8 {
    #[inline(always)]
    fn num_chars(val: u8) -> usize {
        match val {
            u8::MIN..=9 => 1,
            10..=99 => 2,
            100..=u8::MAX => 3,
        }
    }
}

impl NumChars for i8 {
    #[inline(always)]
    fn num_chars(val: i8) -> usize {
        match val {
            i8::MIN..=-100 => 4,
            -99..=-10 => 3,
            -9..=-1 => 2,
            0..=9 => 1,
            10..=99 => 2,
            100..=i8::MAX => 3,
        }
    }
}

impl NumChars for u16 {
    #[inline(always)]
    fn num_chars(val: u16) -> usize {
        match val {
            u16::MIN..=9 => 1,
            10..=99 => 2,
            100..=999 => 3,
            1000..=9999 => 4,
            10000..=u16::MAX => 5,
        }
    }
}

impl NumChars for i16 {
    #[inline(always)]
    fn num_chars(val: i16) -> usize {
        match val {
            i16::MIN..=-10000 => 6,
            -9999..=-1000 => 5,
            -999..=-100 => 4,
            -99..=-10 => 3,
            -9..=-1 => 2,
            0..=9 => 1,
            10..=99 => 2,
            100..=999 => 3,
            1000..=9999 => 4,
            10000..=i16::MAX => 5,
        }
    }
}

impl NumChars for u32 {
    #[inline(always)]
    fn num_chars(val: u32) -> usize {
        match val {
            u32::MIN..=9 => 1,
            10..=99 => 2,
            100..=999 => 3,
            1000..=9999 => 4,
            10000..=99999 => 5,
            100000..=999999 => 6,
            1000000..=9999999 => 7,
            10000000..=99999999 => 8,
            100000000..=999999999 => 9,
            1000000000..=u32::MAX => 10,
        }
    }
}

impl NumChars for i32 {
    #[inline(always)]
    fn num_chars(val: i32) -> usize {
        match val {
            i32::MIN..=-1000000000 => 11,
            -999999999..=-100000000 => 10,
            -99999999..=-10000000 => 9,
            -9999999..=-1000000 => 8,
            -999999..=-100000 => 7,
            -99999..=-10000 => 6,
            -9999..=-1000 => 5,
            -999..=-100 => 4,
            -99..=-10 => 3,
            -9..=-1 => 2,
            0..=9 => 1,
            10..=99 => 2,
            100..=999 => 3,
            1000..=9999 => 4,
            10000..=99999 => 5,
            100000..=999999 => 6,
            1000000..=9999999 => 7,
            10000000..=99999999 => 8,
            100000000..=999999999 => 9,
            1000000000..=i32::MAX => 10,
        }
    }
}

impl NumChars for u64 {
    #[inline(always)]
    fn num_chars(val: u64) -> usize {
        // `checked_ilog10` is `None` only for `0`, which has one digit. Cheaper than a 20-arm
        // match for 64-bit values, and exact (unlike `f64::log10`).
        val.checked_ilog10().map_or(1, |log| log as usize + 1)
    }
}

impl NumChars for i64 {
    #[inline(always)]
    fn num_chars(val: i64) -> usize {
        // Digits of the magnitude plus one for the sign. `unsigned_abs` avoids `-i64::MIN`.
        val.unsigned_abs()
            .checked_ilog10()
            .map_or(1, |log| log as usize + 1)
            + (val < 0) as usize
    }
}

impl NumChars for usize {
    fn num_chars(val: usize) -> usize {
        #[cfg(target_pointer_width = "32")]
        {
            u32::num_chars(val as u32)
        }

        #[cfg(target_pointer_width = "64")]
        {
            u64::num_chars(val as u64)
        }
    }
}

impl NumChars for isize {
    fn num_chars(val: isize) -> usize {
        #[cfg(target_pointer_width = "32")]
        {
            i32::num_chars(val as i32)
        }

        #[cfg(target_pointer_width = "64")]
        {
            i64::num_chars(val as i64)
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::IntoRepr;

    #[test]
    fn test_from_u8_sanity() {
        let vals = [u8::MIN, 1, 0, 42, u8::MAX - 1, u8::MAX];

        for x in &vals {
            let repr = u8::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_i8_sanity() {
        let vals = [i8::MIN, i8::MIN + 1, 0, 42, i8::MAX - 1, i8::MAX];

        for x in &vals {
            let repr = i8::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_u16_sanity() {
        let vals = [u16::MIN, 1, 0, 42, u16::MAX - 1, u16::MAX];

        for x in &vals {
            let repr = u16::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_i16_sanity() {
        let vals = [i16::MIN, i16::MIN + 1, 0, 42, i16::MAX - 1, i16::MAX];

        for x in &vals {
            let repr = i16::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_u32_sanity() {
        let vals = [u32::MIN, 1, 0, 42, u32::MAX - 1, u32::MAX];

        for x in &vals {
            let repr = u32::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_i32_sanity() {
        let vals = [i32::MIN, i32::MIN + 1, 0, 42, i32::MAX - 1, i32::MAX];

        for x in &vals {
            let repr = i32::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_u64_sanity() {
        let vals = [u64::MIN, 1, 0, 42, u64::MAX - 1, u64::MAX];

        for x in &vals {
            let repr = u64::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_i64_sanity() {
        let vals = [i64::MIN, i64::MIN + 1, 0, 42, i64::MAX - 1, i64::MAX];

        for x in &vals {
            let repr = i64::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_usize_sanity() {
        let vals = [usize::MIN, 1, 0, 42, usize::MAX - 1, usize::MAX];

        for x in &vals {
            let repr = usize::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_isize_sanity() {
        let vals = [
            isize::MIN,
            isize::MIN + 1,
            0,
            42,
            isize::MAX - 1,
            isize::MAX,
        ];

        for x in &vals {
            let repr = isize::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_u128_sanity() {
        let vals = [u128::MIN, 1, 0, 42, u128::MAX - 1, u128::MAX];

        for x in &vals {
            let repr = u128::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_i128_sanity() {
        let vals = [i128::MIN, i128::MIN + 1, 0, 42, i128::MAX - 1, i128::MAX];

        for x in &vals {
            let repr = i128::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }
}
