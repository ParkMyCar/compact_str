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
    _padding: [u8; PADDING_SIZE],
    val: u8,
}

impl DiscriminantMask {
    #[inline(always)]
    pub const fn discriminant(&self) -> Discriminant {
        if self.val == HEAP_MASK {
            Discriminant::Heap
        } else {
            Discriminant::Inline
        }
    }
}

crate::asserts::assert_size_eq!(DiscriminantMask, String);
