use super::MAX_SIZE;
use crate::metadata::Metadata;

pub const MAX_INLINE_SIZE: usize = MAX_SIZE - core::mem::size_of::<Metadata>();

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InlineString {
    pub metadata: Metadata,
    pub buffer: [u8; MAX_INLINE_SIZE],
}

impl InlineString {
    pub fn new(text: &str) -> Self {
        debug_assert!(text.len() <= MAX_INLINE_SIZE);

        let metadata = Metadata::new_inline(text);
        let mut buffer = [0u8; MAX_INLINE_SIZE];

        buffer[..text.len()].copy_from_slice(text.as_bytes());

        InlineString { metadata, buffer }
    }
}

static_assertions::assert_eq_size!(InlineString, String);
