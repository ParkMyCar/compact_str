#![deny(warnings)]

//! `CompactStr` is a compact immutable string type that stores itself on the stack, if possible, and seamlessly
//! interacts with `String`s and `&str`s.
//!
//! ### Memory Layout
//! Normally strings are stored on the heap, since they're dynamically sized. In Rust a `String` consists of
//! three things:
//! 1. A `usize` denoting the length of the string
//! 2. A pointer to a location on the heap where the string is stored
//! 3. A `usize` denoting the capacity of the string
//!
//! On 64-bit architectures this results in 24 bytes being stored on the stack, 12 bytes for 32-bit architectures.
//! For small strings, e.g. < 23 characters

use core::{
    borrow::Borrow,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};

mod repr;
use repr::Repr;

#[cfg(feature = "serde")]
mod serde;

#[cfg(test)]
mod tests;

/// A `CompactStr` is a memory efficient immuatable string that can be used almost anywhere a `String`
/// or `&str` can be used.
///
/// ## Using `CompactStr`
/// ```
/// use compact_str::CompactStr;
/// use std::collections::HashMap;
///
/// // CompactStr auto derefs into a str so you can use all methods from `str` that take a `&self`
/// if CompactStr::new("hello world!").is_ascii() {
///     println!("we're all ASCII")
/// }
///
/// // You can use a CompactStr in collections like you would a String or &str
/// let mut map: HashMap<CompactStr, CompactStr> = HashMap::new();
///
/// // directly construct a new `CompactStr`
/// map.insert(CompactStr::new("nyc"), CompactStr::new("empire state building"));
/// // create a `CompactStr` from a `&str`
/// map.insert("sf".into(), "transamerica pyramid".into());
/// // create a `CompactStr` from a `String`
/// map.insert(String::from("sea").into(), String::from("space needle").into());
///
/// fn wrapped_print<T: AsRef<str>>(text: T) {
///     println!("{}", text.as_ref());
/// }
///
/// // CompactStr impls AsRef<str> and Borrow<str>, so it can be used anywhere that excepts a generic string
/// if let Some(building) = map.get("nyc") {
///     wrapped_print(building);
/// }
///
/// // CompactStr can also be directly compared to a String or &str
/// assert_eq!(CompactStr::new("chicago"), "chicago");
/// assert_eq!(CompactStr::new("houston"), String::from("houston"));
/// ```
#[derive(Clone)]
pub struct CompactStr {
    repr: Repr,
}

impl CompactStr {
    #[inline]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        CompactStr {
            repr: Repr::new(text),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.repr.as_str()
    }

    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        self.repr.is_heap_allocated()
    }
}

impl Default for CompactStr {
    fn default() -> Self {
        CompactStr::new("")
    }
}

impl Deref for CompactStr {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for CompactStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for CompactStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Eq for CompactStr {}

impl<T: AsRef<str>> PartialEq<T> for CompactStr {
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<CompactStr> for String {
    fn eq(&self, other: &CompactStr) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Ord for CompactStr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for CompactStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for CompactStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<'a> From<&'a str> for CompactStr {
    fn from(s: &'a str) -> Self {
        CompactStr::new(s)
    }
}

impl From<String> for CompactStr {
    fn from(s: String) -> Self {
        CompactStr::new(&s)
    }
}

impl<'a> From<&'a String> for CompactStr {
    fn from(s: &'a String) -> Self {
        CompactStr::new(&s)
    }
}

impl fmt::Debug for CompactStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for CompactStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

static_assertions::assert_eq_size!(CompactStr, String);
