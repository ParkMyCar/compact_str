use core::mem;

use super::{
    Repr,
    MAX_SIZE,
    STATIC_STR_MASK,
};

pub(super) const DISCRIMINANT_SIZE: usize = MAX_SIZE - mem::size_of::<&'static str>();

/// A buffer stored on the stack whose size is equal to the stack size of `String`
/// The last byte is set to 0.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StaticStr {
    pub text: &'static str,
    #[allow(unused)]
    discriminant: [u8; DISCRIMINANT_SIZE],
}
static_assertions::assert_eq_size!(StaticStr, Repr);
static_assertions::assert_eq_size!(&'static str, (*const u8, usize));

impl StaticStr {
    #[inline]
    pub const fn new(text: &'static str) -> Self {
        let mut discriminant = [0; DISCRIMINANT_SIZE];
        discriminant[DISCRIMINANT_SIZE - 1] = STATIC_STR_MASK;

        Self { text, discriminant }
    }
}
