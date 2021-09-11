use static_assertions::const_assert_eq;
use std::mem;

const MAX_SIZE: usize = mem::size_of::<String>();

#[derive(Debug, Copy, Clone)]
pub struct Repr([u8; MAX_SIZE]);

#[cfg(target_pointer_width = "64")]
const_assert_eq!(mem::size_of::<Repr>(), 24);

#[cfg(target_pointer_width = "32")]
const_assert_eq!(mem::size_of::<Repr>(), 16);
