use super::{HEAP_MASK, LEADING_BIT_MASK, MAX_SIZE};

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
    #[inline]
    pub const fn disciminant(&self) -> Discriminant {
        if self.val == HEAP_MASK {
            Discriminant::Heap
        } else if self.val & LEADING_BIT_MASK == LEADING_BIT_MASK {
            Discriminant::Inline
        } else {
            Discriminant::Packed
        }
    }
}

static_assertions::assert_eq_size!(DiscriminantMask, String);
