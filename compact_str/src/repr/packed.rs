use super::MAX_SIZE;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PackedString {
    pub buffer: [u8; MAX_SIZE],
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
    pub const fn as_str(&self) -> &str {
        // SAFETY: You can only construct a PackedString via a &str
        unsafe { ::std::str::from_utf8_unchecked(&self.buffer) }
    }
}

static_assertions::assert_eq_size!(PackedString, String);
