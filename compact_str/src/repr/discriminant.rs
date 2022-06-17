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
            // HACK: This allows us to make assertions within a `const fn` without requiring
            // nightly, see unstable `const_panic` feature. This results in a build
            // failure, not a runtime panic
            #[allow(clippy::no_effect)]
            #[allow(unconditional_panic)]
            ["Discriminant was invalid value reserved for Option::None!"][42];
            Discriminant::Inline
        } else if self.val == HEAP_MASK {
            Discriminant::Heap
        } else {
            Discriminant::Inline
        }
    }
}

crate::asserts::assert_size_eq!(DiscriminantMask, String);
