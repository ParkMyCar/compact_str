use core::ptr;

use super::{
    Repr,
    LENGTH_MASK,
    MAX_SIZE,
};

/// A buffer stored on the stack whose size is equal to the stack size of `String`
#[repr(transparent)]
pub struct InlineBuffer(pub [u8; MAX_SIZE]);
static_assertions::assert_eq_size!(InlineBuffer, Repr);

impl InlineBuffer {
    /// Construct a new [`InlineString`]. A string that lives in a small buffer on the stack
    ///
    /// SAFETY:
    /// * The caller must guarantee that the length of `text` is less than [`MAX_SIZE`]
    #[inline]
    pub unsafe fn new(text: &str) -> Self {
        debug_assert!(text.len() <= MAX_SIZE);

        let len = text.len();
        let mut buffer = [0u8; MAX_SIZE];

        // set the length in the last byte
        buffer[MAX_SIZE - 1] = len as u8 | LENGTH_MASK;

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
        ptr::copy_nonoverlapping(text.as_ptr(), buffer.as_mut_ptr(), len);

        InlineBuffer(buffer)
    }

    #[inline]
    pub const fn new_const(text: &str) -> Self {
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
    pub const fn empty() -> Self {
        Self::new_const("")
    }

    /// Set's the length of the content for this [`InlineBuffer`]
    ///
    /// # SAFETY:
    /// * The caller must guarantee that `len` bytes in the buffer are valid UTF-8
    #[inline]
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= MAX_SIZE);

        // If `length` == MAX_SIZE, then we infer the length to be the capacity of the buffer. We
        // can infer this because the way we encode length doesn't overlap with any valid UTF-8
        // bytes
        if len < MAX_SIZE {
            self.0[MAX_SIZE - 1] = len as u8 | LENGTH_MASK;
        }
    }

    #[inline(always)]
    pub fn copy(&self) -> Self {
        InlineBuffer(self.0)
    }
}

#[cfg(test)]
mod tests {
    use rayon::prelude::*;

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
                match buf[c.len_utf8() - 1] {
                    x @ 192..=255 => panic!("last byte within 192..=255, {}", x),
                    _ => (),
                }
            }
        })
    }
}
