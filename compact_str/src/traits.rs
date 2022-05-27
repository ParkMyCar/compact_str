use core::fmt::{
    self,
    Write,
};
use core::num;

use castaway::{
    match_type,
    LifetimeFree,
};

use super::repr::{
    IntoRepr,
    Repr,
};
use super::utility::count;
use crate::CompactStr;

/// A trait for converting a value to a `CompactStr`.
///
/// This trait is automatically implemented for any type which implements the
/// [`fmt::Display`] trait. As such, `ToCompactStr` shouldn't be implemented directly:
/// [`fmt::Display`] should be implemented instead, and you get the `ToCompactStr`
/// implementation for free.
pub trait ToCompactStr {
    /// Converts the given value to a `CompactStr`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use compact_str::ToCompactStr;
    /// # use compact_str::CompactStr;
    ///
    /// let i = 5;
    /// let five = CompactStr::new("5");
    ///
    /// assert_eq!(i.to_compact_str(), five);
    /// ```
    fn to_compact_str(&self) -> CompactStr;
}

/// # Safety
///
/// * CompactStr does not contain any lifetime
/// * CompactStr is 'static
/// * CompactStr is a container to `u8`, which is `LifetimeFree`.
unsafe impl LifetimeFree for CompactStr {}
unsafe impl LifetimeFree for Repr {}

/// # Panics
///
/// In this implementation, the `to_compact_str` method panics if the `Display` implementation
/// returns an error. This indicates an incorrect `Display` implementation since
/// `std::fmt::Write for CompactStr` never returns an error itself.
///
/// # Note
///
/// We use the [`castaway`] crate to provide zero-cost specialization for several types, those are:
/// * `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
/// * `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
/// * `NonZeroU*`, `NonZeroI*`
/// * `bool`
/// * `char`
/// * `String`, `CompactStr`
/// * `f32`, `f64`
///     * For floats we use [`ryu`] crate which sometimes provides different formatting than [`std`]
impl<T: fmt::Display> ToCompactStr for T {
    #[inline]
    fn to_compact_str(&self) -> CompactStr {
        let repr = match_type!(self, {
            &u8 as s => s.into_repr(),
            &i8 as s => s.into_repr(),
            &u16 as s => s.into_repr(),
            &i16 as s => s.into_repr(),
            &u32 as s => s.into_repr(),
            &i32 as s => s.into_repr(),
            &u64 as s => s.into_repr(),
            &i64 as s => s.into_repr(),
            &u128 as s => s.into_repr(),
            &i128 as s => s.into_repr(),
            &usize as s => s.into_repr(),
            &isize as s => s.into_repr(),
            &f32 as s => s.into_repr(),
            &f64 as s => s.into_repr(),
            &bool as s => s.into_repr(),
            &char as s => s.into_repr(),
            &String as s => Repr::new(&*s),
            &CompactStr as s => Repr::new(s),
            &num::NonZeroU8 as s => s.into_repr(),
            &num::NonZeroI8 as s => s.into_repr(),
            &num::NonZeroU16 as s => s.into_repr(),
            &num::NonZeroI16 as s => s.into_repr(),
            &num::NonZeroU32 as s => s.into_repr(),
            &num::NonZeroI32 as s => s.into_repr(),
            &num::NonZeroU64 as s => s.into_repr(),
            &num::NonZeroI64 as s => s.into_repr(),
            &num::NonZeroUsize as s => s.into_repr(),
            &num::NonZeroIsize as s => s.into_repr(),
            &num::NonZeroU128 as s => s.into_repr(),
            &num::NonZeroI128 as s => s.into_repr(),
            s => {
                let num_bytes = count(s);
                let mut repr = Repr::with_capacity(num_bytes);

                write!(&mut repr, "{}", s).expect("fmt::Display incorrectly implemented!");

                repr
            }
        });

        CompactStr { repr }
    }
}

#[cfg(test)]
mod tests {
    use core::num;

    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::ToCompactStr;

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_u8(val: u8) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_i8(val: i8) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_u16(val: u16) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_i16(val: i16) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_u32(val: u32) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_i32(val: i32) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_u64(val: u64) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_i64(val: i64) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_usize(val: usize) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_isize(val: isize) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_u128(val: u128) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_i128(val: i128) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_u8(
        #[strategy((1..=u8::MAX).prop_map(|x| unsafe { num::NonZeroU8::new_unchecked(x)} ))]
        val: num::NonZeroU8,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_u16(
        #[strategy((1..=u16::MAX).prop_map(|x| unsafe { num::NonZeroU16::new_unchecked(x)} ))]
        val: num::NonZeroU16,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_u32(
        #[strategy((1..=u32::MAX).prop_map(|x| unsafe { num::NonZeroU32::new_unchecked(x)} ))]
        val: num::NonZeroU32,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_u64(
        #[strategy((1..=u64::MAX).prop_map(|x| unsafe { num::NonZeroU64::new_unchecked(x)} ))]
        val: num::NonZeroU64,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_u128(
        #[strategy((1..=u128::MAX).prop_map(|x| unsafe { num::NonZeroU128::new_unchecked(x)} ))]
        val: num::NonZeroU128,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_usize(
        #[strategy((1..=usize::MAX).prop_map(|x| unsafe { num::NonZeroUsize::new_unchecked(x)} ))]
        val: num::NonZeroUsize,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_i8(
        #[strategy((1..=u8::MAX).prop_map(|x| unsafe { num::NonZeroI8::new_unchecked(x as i8)} ))]
        val: num::NonZeroI8,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_i16(
        #[strategy((1..=u16::MAX).prop_map(|x| unsafe { num::NonZeroI16::new_unchecked(x as i16)} ))]
        val: num::NonZeroI16,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_i32(
        #[strategy((1..=u32::MAX).prop_map(|x| unsafe { num::NonZeroI32::new_unchecked(x as i32)} ))]
        val: num::NonZeroI32,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_i64(
        #[strategy((1..=u64::MAX).prop_map(|x| unsafe { num::NonZeroI64::new_unchecked(x as i64)} ))]
        val: num::NonZeroI64,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_i128(
        #[strategy((1..=u128::MAX).prop_map(|x| unsafe { num::NonZeroI128::new_unchecked(x as i128)} ))]
        val: num::NonZeroI128,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_to_compact_str_non_zero_isize(
        #[strategy((1..=usize::MAX).prop_map(|x| unsafe { num::NonZeroIsize::new_unchecked(x as isize)} ))]
        val: num::NonZeroIsize,
    ) {
        let compact = val.to_compact_str();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
}
