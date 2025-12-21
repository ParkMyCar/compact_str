//! A module that contains the implementations for optional features. For example `serde` support

#[cfg(feature = "arbitrary")]
mod arbitrary;
#[cfg(feature = "borsh")]
mod borsh;
#[cfg(feature = "bytes")]
mod bytes;
#[cfg(feature = "defmt")]
mod defmt;
#[cfg(feature = "diesel")]
mod diesel;
#[cfg(feature = "markup")]
mod markup;
#[cfg(feature = "proptest")]
mod proptest;
#[cfg(feature = "quickcheck")]
mod quickcheck;
#[cfg(feature = "rkyv")]
mod rkyv;
#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "smallvec")]
mod smallvec;
#[cfg(feature = "sqlx")]
mod sqlx;
#[cfg(feature = "utoipa")]
mod utoipa;
#[cfg(feature = "zeroize")]
mod zeroize;
#[cfg(feature = "bevy-reflect")]
mod bevy_reflect;