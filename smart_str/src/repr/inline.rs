use super::{LEADING_BIT_MASK, MAX_SIZE};

pub const MAX_INLINE_SIZE: usize = MAX_SIZE - core::mem::size_of::<Metadata>();

type Metadata = u8;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InlineString {
    pub metadata: Metadata,
    pub buffer: [u8; MAX_INLINE_SIZE],
}

impl InlineString {
    #[inline]
    pub fn new(text: &str) -> Self {
        debug_assert!(text.len() <= MAX_INLINE_SIZE);

        let len = text.len();
        let metadata = (len as u8) | LEADING_BIT_MASK;
        let mut buffer = [0u8; MAX_INLINE_SIZE];

        buffer[..len].copy_from_slice(text.as_bytes());

        InlineString { metadata, buffer }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        (self.metadata & !LEADING_BIT_MASK) as usize
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        let len = self.len();
        let slice = &self.buffer[..len];

        // SAFETY: You can only construct an InlineString via a &str
        unsafe { ::std::str::from_utf8_unchecked(slice) }
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
