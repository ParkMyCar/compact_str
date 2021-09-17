use super::MAX_SIZE;

const PADDING_SIZE: usize = MAX_SIZE - std::mem::size_of::<Discriminant>();

const HEAP_MASK: u8 = 0b11111111;
pub const LEADING_BIT_MASK: u8 = 0b10000000;

#[derive(Debug, Copy, Clone)]
pub enum Discriminant {
    Heap,
    Inline,
    Packed,
}

#[derive(Debug, Copy, Clone)]
pub struct DiscriminantMask {
    val: u8,
    _padding: [u8; PADDING_SIZE],
}

impl DiscriminantMask {
    pub fn disciminant(&self) -> Discriminant {
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
