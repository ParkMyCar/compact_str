use super::{Repr, LENGTH_MASK, MAX_SIZE};

/// A buffer stored on the stack whose size is equal to the stack size of `String`
#[cfg(target_pointer_width = "64")]
#[repr(C, align(8))]
pub(crate) struct InlineBuffer(pub(crate) [u8; MAX_SIZE]);

#[cfg(target_pointer_width = "32")]
#[repr(C, align(4))]
pub(crate) struct InlineBuffer(pub(crate) [u8; MAX_SIZE]);

static_assertions::assert_eq_size!(InlineBuffer, Repr);
static_assertions::assert_eq_align!(InlineBuffer, Repr);

impl InlineBuffer {
    /// Construct a new [`InlineString`]. A string that lives in a small buffer on the stack
    ///
    /// SAFETY:
    /// * The caller must guarantee that the length of `text` is less than [`MAX_SIZE`]
    #[inline]
    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    pub(crate) unsafe fn new(text: &str) -> Self {
        debug_assert!(text.len() <= MAX_SIZE);

        use core::ptr::read_unaligned as load;

        let len = text.len();
        let src = text.as_ptr();

        // Note(parker): This implementation builds an `InlineBuffer` entirely in
        // registers. It solves a long standing performance issue `compact_str` has had
        // since it's creation.
        // The simple implementation is to copy the string into a stack buffer of
        // MAX_SIZE. Via our `copy_small` helper or libc's `memcpy` function, it is
        // executed as two overlapping stores.
        //
        // For example, the string "hello world" is copied as two 8-byte ops:
        //
        // 1. "hello wo" into [0, 7]
        // 2. "lo world" into [3, 10]
        //
        // Then we have do a _third_ store to the final byte for the length.
        //
        // ( Note: we also need to consider the stores that zero the original buffer, but
        //   for illustrative purposes considering just "hello world" is enough )
        //
        // Immediately after, the stack buffer gets moved, returned as a Repr and then to
        // the user as a CompactString. These consumers read the data as _whole aligned words_.
        // At this point the data is in the CPU's store buffer, not yet written out to
        // the L1 cache.
        //
        // Enter the ~~SPOOKY ZONE~~ of micro-architectures.
        //
        // On Intel, the CPU can forward a load from the store buffer __if the load is
        // fully contained within a single store__. Our issue is that the first word/8-bytes
        // _overlaps two stores_ and thus causes a pipeline hazard
        // Apple's `aarch64` can forward multi-store loads, so this is less of a problem
        // there.
        //
        //
        // Below, we build an `InlineBuffer` in registers, replacing the overlapping
        // stores. Afterwards, the buffer is materialized with at most three aligned
        // non-overlapping stores, or kept in registers entirely.

        let last_byte = (len as u64 | LENGTH_MASK as u64) << 56;

        let (w0, w1, w2);
        if len == 0 {
            // Checked first so the empty string is a single compare, and so the final
            // `else` below is exactly `len == 1` without needing its own test. Hoisting
            // this case is net-zero code size: the branch added here replaces the one
            // removed at the bottom of the ladder.
            w0 = 0;
            w1 = 0;
            w2 = last_byte;
        } else if len >= 16 {
            // SAFETY: `len >= 16` means `src` is valid for 16 bytes. and `src + len - 8`
            // is valid for 8 bytes because `len <= text.len()`.
            w0 = load(src as *const u64);
            w1 = load(src.add(8) as *const u64);
            // N.B. For any length < MAX_SIZE, this is an overlaping read of `w1` and the
            // tail of the string.
            let tail = load(src.add(len - 8) as *const u64);

            // bytes `[16, len)` of the string are the top `len - 16` bytes of `tail`.
            let data = if len == 16 {
                0
            } else {
                tail >> ((MAX_SIZE - len) * 8)
            };
            w2 = if len == MAX_SIZE {
                data
            } else {
                data | last_byte
            };
        } else if len >= 8 {
            // SAFETY: `src` is valid for `len >= 8` bytes.
            w0 = load(src as *const u64);
            // N.B. Overlapping load with w0 for the tail of the string.
            let tail = load(src.add(len - 8) as *const u64);

            // bytes `[8, len)` of the string are the top `len - 8` bytes of `tail`
            w1 = if len == 8 {
                0
            } else {
                tail >> ((16 - len) * 8)
            };
            w2 = last_byte;
        } else if len >= 4 {
            // SAFETY: `src` is valid for `len >= 4` bytes.
            let head = load(src as *const u32) as u64;
            let tail = load(src.add(len - 4) as *const u32) as u64;
            w0 = head | (tail << ((len - 4) * 8));
            w1 = 0;
            w2 = last_byte;
        } else if len >= 2 {
            // SAFETY: `src` is valid for `len >= 2` bytes.
            let head = load(src as *const u16) as u64;
            let tail = load(src.add(len - 2) as *const u16) as u64;
            w0 = head | (tail << ((len - 2) * 8));
            w1 = 0;
            w2 = last_byte;
        } else {
            // `len == 0` was handled above and `len >= 2` just failed, so this is `len == 1`.
            w0 = *src as u64;
            w1 = 0;
            w2 = last_byte;
        }

        // SAFETY: `[u64; 3]` and `InlineBuffer` have the same size, and on little-endian
        // the byte order of the words matches the byte order of the buffer.
        core::mem::transmute([w0, w1, w2])
    }

    /// Construct a new [`InlineString`]. A string that lives in a small buffer on the stack
    ///
    /// SAFETY:
    /// * The caller must guarantee that the length of `text` is less than [`MAX_SIZE`]
    #[inline]
    #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
    pub(crate) unsafe fn new(text: &str) -> Self {
        debug_assert!(text.len() <= MAX_SIZE);

        let len = text.len();
        let mut buffer = InlineBuffer([0u8; MAX_SIZE]);

        // set the length in the last byte
        buffer.0[MAX_SIZE - 1] = len as u8 | LENGTH_MASK;

        // copy the string into our buffer
        //
        // note: in the case where len == MAX_SIZE, we'll overwrite the len, but that's okay because
        // when reading the length we can detect that the last byte is part of UTF-8 and return a
        // length of MAX_SIZE
        //
        // SAFETY:
        // * src (`text`) is valid for `len` bytes because `len` comes from `text`
        // * dst (`buffer`) is valid for `len` bytes because we assert src is less than MAX_SIZE
        // * src and dst don't overlap because we created dst
        //
        super::copy_small(text.as_ptr(), buffer.0.as_mut_ptr(), len);

        buffer
    }

    #[inline]
    pub(crate) const fn new_const(text: &str) -> Self {
        if text.len() > MAX_SIZE {
            panic!("Provided string has a length greater than our MAX_SIZE");
        }

        let len = text.len();
        let mut buffer = [0u8; MAX_SIZE];

        // set the length
        buffer[MAX_SIZE - 1] = len as u8 | LENGTH_MASK;

        // Note: for loops aren't allowed in `const fn`, hence the while.
        // Note: Iterating forward results in badly optimized code, because the compiler tries to
        //       unroll the loop.
        let text = text.as_bytes();
        let mut i = len;
        while i > 0 {
            buffer[i - 1] = text[i - 1];
            i -= 1;
        }

        InlineBuffer(buffer)
    }

    /// Returns an empty [`InlineBuffer`]
    #[inline(always)]
    pub(crate) const fn empty() -> Self {
        Self::new_const("")
    }

    /// Consumes the [`InlineBuffer`] returning the entire underlying array and the length of the
    /// string that it contains
    #[inline]
    #[cfg(feature = "smallvec")]
    pub(crate) fn into_array(self) -> ([u8; MAX_SIZE], usize) {
        let mut buffer = self.0;

        let length = core::cmp::min(
            (buffer[MAX_SIZE - 1].wrapping_sub(LENGTH_MASK)) as usize,
            MAX_SIZE,
        );

        let last_byte_ref = &mut buffer[MAX_SIZE - 1];

        // unset the last byte of the buffer if it's just storing the length of the string
        //
        // Note: we should never add an `else` statement here, keeping the conditional simple allows
        // the compiler to optimize this to a conditional-move instead of a branch
        if length < MAX_SIZE {
            *last_byte_ref = 0;
        }

        (buffer, length)
    }

    /// Set's the length of the content for this [`InlineBuffer`]
    ///
    /// # SAFETY:
    /// * The caller must guarantee that `len` bytes in the buffer are valid UTF-8
    #[inline]
    pub(crate) unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= MAX_SIZE);

        // If `length` == MAX_SIZE, then we infer the length to be the capacity of the buffer. We
        // can infer this because the way we encode length doesn't overlap with any valid UTF-8
        // bytes
        if len < MAX_SIZE {
            self.0[MAX_SIZE - 1] = len as u8 | LENGTH_MASK;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore] // we run this in CI, but unless you're compiling in release, this takes a while
    fn test_unused_utf8_bytes() {
        use rayon::prelude::*;

        // test to validate for all char the first and last bytes are never within a specified range
        // note: according to the UTF-8 spec it shouldn't be, but we double check that here
        (0..u32::MAX).into_par_iter().for_each(|i| {
            if let Ok(c) = char::try_from(i) {
                let mut buf = [0_u8; 4];
                c.encode_utf8(&mut buf);

                // check ranges for first byte
                match buf[0] {
                    x @ 128..=191 => panic!("first byte within 128..=191, {}", x),
                    x @ 248..=255 => panic!("first byte within 248..=255, {}", x),
                    _ => (),
                }

                // check ranges for last byte
                if let x @ 192..=255 = buf[c.len_utf8() - 1] {
                    panic!("last byte within 192..=255, {}", x)
                }
            }
        })
    }

    #[cfg(feature = "smallvec")]
    mod smallvec {
        use alloc::string::String;

        use quickcheck_macros::quickcheck;

        use crate::repr::{InlineBuffer, MAX_SIZE};

        #[test]
        fn test_into_array() {
            let s = "hello world!";

            let inline = unsafe { InlineBuffer::new(s) };
            let (array, length) = inline.into_array();

            assert_eq!(s.len(), length);

            // all bytes after the length should be 0
            assert!(array[length..].iter().all(|b| *b == 0));

            // taking a string slice should give back the same string as the original
            let ex_s = unsafe { core::str::from_utf8_unchecked(&array[..length]) };
            assert_eq!(s, ex_s);
        }

        #[quickcheck]
        #[cfg_attr(miri, ignore)]
        fn quickcheck_into_array(s: String) {
            let mut total_length = 0;
            let s: String = s
                .chars()
                .take_while(|c| {
                    total_length += c.len_utf8();
                    total_length < MAX_SIZE
                })
                .collect();

            let inline = unsafe { InlineBuffer::new(&s) };
            let (array, length) = inline.into_array();
            assert_eq!(s.len(), length);

            // all bytes after the length should be 0
            assert!(array[length..].iter().all(|b| *b == 0));

            // taking a string slice should give back the same string as the original
            let ex_s = unsafe { core::str::from_utf8_unchecked(&array[..length]) };
            assert_eq!(s, ex_s);
        }
    }
}
