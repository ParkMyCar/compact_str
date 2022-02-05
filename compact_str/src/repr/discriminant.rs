use super::{
    HEAP_MASK,
    MAX_SIZE,
};

const PADDING_SIZE: usize = MAX_SIZE - std::mem::size_of::<Discriminant>();

#[derive(Debug, Copy, Clone)]
pub enum Discriminant {
    Heap,
    Inline,
    Packed,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DiscriminantMask {
    val: u8,
    _padding: [u8; PADDING_SIZE],
}

impl DiscriminantMask {
    #[inline(always)]
    pub const fn discriminant(&self) -> Discriminant {
        if self.val == HEAP_MASK {
            Discriminant::Heap
        } else if self.val >> 6 == 0b00000010 {
            Discriminant::Inline
        } else {
            Discriminant::Packed
        }
    }
}

crate::asserts::assert_size_eq!(DiscriminantMask, String);
