use crate::repr::HEAP_MASK;

const CAPACITY_SPACE: usize = core::mem::size_of::<usize>() - 1;

#[derive(Debug)]
pub struct Capacity {
    _cap: [u8; CAPACITY_SPACE],
    _discriminant: u8,
}

impl Capacity {
    pub const fn new() -> Self {
        Capacity {
            _cap: [0u8; CAPACITY_SPACE],
            _discriminant: HEAP_MASK,
        }
    }
}

crate::asserts::assert_size_eq!(Capacity, usize);
