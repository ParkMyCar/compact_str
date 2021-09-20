use super::{LEADING_BIT_MASK, MAX_SIZE};

pub const MAX_INLINE_SIZE: usize = MAX_SIZE - core::mem::size_of::<Metadata>();

type Metadata = u8;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InlineString {
    metadata: Metadata,
    buffer: [u8; MAX_INLINE_SIZE],
}

impl InlineString {
    const fn empty() -> Self {
        InlineString {
            metadata: LEADING_BIT_MASK,
            buffer: [0u8; MAX_INLINE_SIZE],
        }
    }

    #[inline]
    pub fn new(text: &str) -> Self {
        debug_assert!(text.len() <= MAX_INLINE_SIZE);

        let len = text.len();
        let mut new = Self::empty();

        // set the length
        new.metadata |= len as u8;
        // copy the string
        new.buffer.as_mut()[..len].copy_from_slice(text.as_bytes());

        new
    }

    #[inline]
    pub const fn new_const(text: &str) -> Self {
        if text.len() > MAX_INLINE_SIZE {
            // HACK: This allows us to make assertions within a `const fn` without requiring nightly,
            // see unstable `const_panic` feature. This results in a build failure, not a runtime panic
            #[allow(clippy::no_effect)]
            #[allow(unconditional_panic)]
            ["Provided string has a length greater than MAX_INLINE_SIZE!"][42];
        }

        let len = text.len();
        let metadata = (len as u8) | LEADING_BIT_MASK;
        let mut buffer = [0u8; MAX_INLINE_SIZE];

        // Note: for loops aren't allowed in `const fn`, hence the while
        let mut i = 0;
        while i < len {
            buffer[i] = text.as_bytes()[i];
            i += 1;
        }

        InlineString { metadata, buffer }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        (self.metadata & !LEADING_BIT_MASK) as usize
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        let len = self.len();

        // SAFETY: Constructors guarantee that `buffer[..len]` is a `str`,
        // and we don't mutate the data afterwards.
        unsafe {
            let slice = self.buffer.get_unchecked(..len);
            ::std::str::from_utf8_unchecked(slice)
        }
    }
}

static_assertions::assert_eq_size!(InlineString, String);

#[cfg(test)]
mod tests {
    #[test]
    fn test_sanity_not_valid_utf8() {
        assert!(std::str::from_utf8(&[0b11111111]).is_err())
    }
}
