//! A module that contains the implementations for optional features. For example `serde` support

mod bytes;
// #[cfg(feature = "pb_jelly")]
mod pb_jelly;
#[cfg(feature = "serde")]
mod serde;
