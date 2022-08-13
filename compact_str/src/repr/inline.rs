use super::MAX_SIZE;

const LENGTH_MASK: u8 = 0b1100_0000;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InlineString {
    buffer: [u8; MAX_SIZE],
}

impl InlineString {
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(text: &str) -> Self {
        debug_assert!(text.len() <= MAX_SIZE);

        let len = text.len();
        let mut buffer = [0u8; MAX_SIZE];

        // set the length
        buffer[MAX_SIZE - 1] = len as u8 | LENGTH_MASK;

        // copy the string
        //
        // note: in the case where len == MAX_SIZE, we'll overwrite the len, but that's okay because
        // when reading the length we can detect that the last byte is part of UTF-8 and return a
        // length of MAX_SIZE
        unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), buffer.as_mut_ptr(), len) };

        InlineString { buffer }
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn new_const(text: &str) -> Self {
        assert!(
            text.len() <= MAX_SIZE,
            "Provided string has a length greater than our MAX_SIZE",
        );

        let len = text.len();
        let mut buffer = [0u8; MAX_SIZE];

        // set the length
        buffer[MAX_SIZE - 1] = len as u8 | LENGTH_MASK;

        // Note: for loops aren't allowed in `const fn`, hence the while
        let mut i = 0;
        while i < len {
            buffer[i] = text.as_bytes()[i];
            i += 1;
        }

        InlineString { buffer }
    }

    /// Creates an [`InlineString`] from raw parts without checking that it's valid UTF-8
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub const unsafe fn from_parts(len: usize, mut buffer: [u8; MAX_SIZE]) -> Self {
        if len != MAX_SIZE {
            buffer[MAX_SIZE - 1] = len as u8 | LENGTH_MASK;
        }
        InlineString { buffer }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        let last_byte = self.buffer[MAX_SIZE - 1];
        core::cmp::min((last_byte.wrapping_sub(LENGTH_MASK)) as usize, MAX_SIZE)
    }

    #[inline]
    pub const fn capacity(self) -> usize {
        let _ = self;
        MAX_SIZE
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: You can only safely construct an InlineString via a &str which also properly sets
        // the length of the string
        unsafe {
            let slice = self.as_slice().get_unchecked(..self.len());
            ::std::str::from_utf8_unchecked(slice)
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    /// Provides a mutable reference to the underlying buffer
    ///
    /// # Safety
    /// * Please see `super::Repr` for all invariants
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        self.buffer.as_mut_slice()
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub unsafe fn set_len(&mut self, length: usize) {
        debug_assert!(length <= MAX_SIZE);

        // If `length` == MAX_SIZE, then we infer the length to be the capacity of the buffer. We
        // can infer this because the way we encode length doesn't overlap with any valid UTF-8
        // bytes
        if length < MAX_SIZE {
            self.buffer[MAX_SIZE - 1] = length as u8 | LENGTH_MASK;
        }
    }
}

crate::asserts::assert_size_eq!(InlineString, String);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use proptest::prelude::*;
    use rayon::prelude::*;
    use test_strategy::proptest;

    use super::{
        InlineString,
        MAX_SIZE,
    };
    use crate::tests::rand_unicode_with_max_len;

    #[test]
    fn test_sanity() {
        let hello = "hello world!";
        let inline = InlineString::new(hello);

        assert_eq!(inline.as_str(), hello);
        assert_eq!(inline.len(), hello.len());
        assert_eq!(inline.capacity(), MAX_SIZE);
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_roundtrip(#[strategy(rand_unicode_with_max_len(MAX_SIZE))] s: String) {
        let inline = InlineString::new(&s);

        prop_assert_eq!(inline.len(), s.len());
        prop_assert_eq!(inline.as_str(), s);
    }

    #[test]
    #[ignore] // we run this in CI, but unless you're compiling in release, this takes a while
    fn test_unused_utf8_bytes() {
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
}
