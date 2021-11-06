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
    pub const fn discriminant(&self) -> Discriminant {
        if self.val == HEAP_MASK {
            Discriminant::Heap
        } else if self.val ^ LEADING_BIT_MASK < MAX_SIZE as u8 {
            Discriminant::Inline
        } else if self.val <= 127 {
            Discriminant::Packed
        } else {
            // HACK: This allows us to make assertions within a `const fn` without requiring nightly,
            // see unstable `const_panic` feature. This results in a build failure, not a runtime panic
            #[allow(clippy::no_effect)]
            #[allow(unconditional_panic)]
            ["Invalid discriminant!"][42];

            // We should never actually reach here
            Discriminant::Packed
        }
    }
}

static_assertions::assert_eq_size!(DiscriminantMask, String);
