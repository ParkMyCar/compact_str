//! A module that contains the implementations for optional features. For example `serde` support

#[cfg(feature = "bytes")]
mod bytes;
#[cfg(feature = "serde")]
mod serde;
