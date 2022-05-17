use core::fmt::{
    self,
    Write,
};
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
/// [`Display`] trait. As such, `ToCompactStr` shouldn't be implemented directly:
/// [`Display`] should be implemented instead, and you get the `ToCompactStr`
/// implementation for free.
///
/// [`Display`]: fmt::Display
pub trait ToCompactStr {
    /// Converts the given value to a `CompactStr`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use compact_str::ToCompactStr;
    ///
    /// let i = 5;
    /// let five = "5".to_compact_str();
    ///
    /// assert_eq!(five, i.to_string());
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
/// In this implementation, the `to_compact_str` method panics
/// if the `Display` implementation returns an error.
/// This indicates an incorrect `Display` implementation
/// since `std::fmt::Write for String` never returns an error itself and
/// the implementation of `ToCompactStr::to_compact_str` only panic if the `Display`
/// implementation is incorrect..
impl<T: fmt::Display> ToCompactStr for T {
    #[inline]
    fn to_compact_str(&self) -> CompactStr {
        let repr = match_type!(self, {
            u8 as s => s.into_repr(),
            &u8 as s => s.into_repr(),
            i8 as s => s.into_repr(),
            &i8 as s => s.into_repr(),
            u16 as s => s.into_repr(),
            &u16 as s => s.into_repr(),
            i16 as s => s.into_repr(),
            &i16 as s => s.into_repr(),
            u32 as s => s.into_repr(),
            &u32 as s => s.into_repr(),
            i32 as s => s.into_repr(),
            &i32 as s => s.into_repr(),
            u64 as s => s.into_repr(),
            &u64 as s => s.into_repr(),
            i64 as s => s.into_repr(),
            &i64 as s => s.into_repr(),
            usize as s => s.into_repr(),
            &usize as s => s.into_repr(),
            isize as s => s.into_repr(),
            &isize as s => s.into_repr(),
            f32 as s => s.into_repr(),
            &f32 as s => s.into_repr(),
            f64 as s => s.into_repr(),
            &f64 as s => s.into_repr(),
            bool as s => s.into_repr(),
            &bool as s => s.into_repr(),
            char as s => s.into_repr(),
            &char as s => s.into_repr(),
            String as s => Repr::from_string(s),
            &String as s => Repr::new(&*s),
            CompactStr as s => Repr::new(&*s),
            &CompactStr as s => Repr::new(s),
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
