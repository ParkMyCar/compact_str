use super::discriminant::LEADING_BIT_MASK;
use super::MAX_SIZE;

pub const MAX_INLINE_SIZE: usize = MAX_SIZE - core::mem::size_of::<Metadata>();

#[derive(Debug, Copy, Clone)]
pub struct Metadata(u8);

impl Metadata {
    pub fn new(data: u8) -> Self {
        // assert no bits from `data` will overlap with the Discriminant
        debug_assert_eq!(data & LEADING_BIT_MASK, 0);

        let mut metadata = data;

        // clear all the bits used by the Discriminant
        metadata &= !LEADING_BIT_MASK;
        // set the disciminant
        metadata |= LEADING_BIT_MASK;

        Metadata(metadata)
    }

    #[inline]
    pub const fn data(&self) -> u8 {
        // return the underlying u8, sans any bits from the discriminant
        self.0 & !LEADING_BIT_MASK
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InlineString {
    pub metadata: Metadata,
    pub buffer: [u8; MAX_INLINE_SIZE],
}

impl InlineString {
    pub fn new(text: &str) -> Self {
        debug_assert!(text.len() <= MAX_INLINE_SIZE);

        let metadata = Metadata::new(text.len() as u8);
        let mut buffer = [0u8; MAX_INLINE_SIZE];

        buffer[..text.len()].copy_from_slice(text.as_bytes());

        InlineString { metadata, buffer }
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
