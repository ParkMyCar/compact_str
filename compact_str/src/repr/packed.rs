use super::MAX_SIZE;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PackedString {
    buffer: [u8; MAX_SIZE],
}

impl PackedString {
    #[inline]
    pub fn new(text: &str) -> Self {
        debug_assert_eq!(text.len(), MAX_SIZE);
        debug_assert!(text.as_bytes()[0] <= 127);

        let mut buffer = [0u8; MAX_SIZE];
        buffer[..text.len()].copy_from_slice(text.as_bytes());

        PackedString { buffer }
    }

    #[inline]
    pub const fn new_const(text: &str) -> Self {
        if text.len() != MAX_SIZE {
            // HACK: This allows us to make assertions within a `const fn` without requiring
            // nightly, see unstable `const_panic` feature. This results in a build
            // failure, not a runtime panic
            #[allow(clippy::no_effect)]
            #[allow(unconditional_panic)]
            ["Provided string has a length greater than MAX_SIZE!"][42];
        }
        if text.as_bytes()[0] > 127 {
            // HACK: This allows us to make assertions within a `const fn` without requiring
            // nightly, see unstable `const_panic` feature. This results in a build
            // failure, not a runtime panic
            #[allow(clippy::no_effect)]
            #[allow(unconditional_panic)]
            ["leading character of packed string isn't ASCII!"][42];
        }

        let mut buffer = [0u8; MAX_SIZE];
        let mut i = 0;
        while i < text.len() {
            buffer[i] = text.as_bytes()[i];
            i += 1;
        }

        PackedString { buffer }
    }

    /// Creates a `PackedString` from raw parts without checking that it's valid UTF-8, or if the
    /// first character is <= 127 (aka ASCII)
    #[inline]
    pub const unsafe fn from_parts(buffer: [u8; MAX_SIZE]) -> Self {
        PackedString { buffer }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        MAX_SIZE
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        MAX_SIZE
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: You can only construct a PackedString via a &str
        unsafe { ::std::str::from_utf8_unchecked(self.as_slice()) }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[..]
    }

    /// Provides a mutable reference to the underlying buffer
    ///
    /// # Invariants
    /// * Please see `super::Repr` for all invariants
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buffer[..]
    }

    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        // An invariant of the Packed representation is that the size always equals MAX_SIZE, so
        // theres no work to do here, other than assert the length we're trying to set is MAX_SIZE
        debug_assert_eq!(length, MAX_SIZE);
    }
}

crate::asserts::assert_size_eq!(PackedString, String);
