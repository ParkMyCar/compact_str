use super::{
    HEAP_MASK,
    PADDING_SIZE,
};

#[derive(Debug, Copy, Clone)]
pub enum Discriminant {
    Heap,
    Inline,
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
        if self.val == u8::MAX {
            panic!("Discriminant was invalid value reserved for Option::None!")
        } else if self.val == HEAP_MASK {
            Discriminant::Heap
        } else {
            Discriminant::Inline
        }
    }
}

static_assertions::assert_eq_size!(DiscriminantMask, String);
