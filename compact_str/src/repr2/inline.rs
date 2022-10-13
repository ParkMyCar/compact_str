use core::ptr;

use super::{
    Repr,
    LENGTH_MASK,
    MAX_SIZE,
};

/// A buffer stored on the stack whose size is equal to the stack size of `String`
#[repr(transparent)]
pub struct InlineBuffer(pub [u8; MAX_SIZE]);
crate::asserts::assert_size_eq!(InlineBuffer, Repr);

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
