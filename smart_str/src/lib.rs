//! `SmartStr` is a smart immutable string type that stores itself on the stack, if possible, and seamlessly
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

/// A `SmartStr` is a memory efficient immuatable string that can be used almost anywhere a `String`
/// or `&str` can be used.
///
/// ## Using `SmartStr`
/// ```
/// use smart_str::SmartStr;
/// use std::collections::HashMap;
///
/// // SmartStr auto derefs into a str so you can use all methods from `str` that take a `&self`
/// if SmartStr::new("hello world!").is_ascii() {
///     println!("we're all ASCII")
/// }
///
/// // You can use a SmartStr in collections like you would a String or &str
/// let mut map: HashMap<SmartStr, SmartStr> = HashMap::new();
///
/// // directly construct a new `SmartStr`
/// map.insert(SmartStr::new("nyc"), SmartStr::new("empire state building"));
/// // create a `SmartStr` from a `&str`
/// map.insert("sf".into(), "transamerica pyramid".into());
/// // create a `SmartStr` from a `String`
/// map.insert(String::from("sea").into(), String::from("space needle").into());
///
/// fn wrapped_print<T: AsRef<str>>(text: T) {
///     println!("{}", text.as_ref());
/// }
///
/// // SmartStr impls AsRef<str> and Borrow<str>, so it can be used anywhere that excepts a generic string
/// if let Some(building) = map.get("nyc") {
///     wrapped_print(building);
/// }
///
/// // SmartStr can also be directly compared to a String or &str
/// assert_eq!(SmartStr::new("chicago"), "chicago");
/// assert_eq!(SmartStr::new("houston"), String::from("houston"));
/// ```
#[derive(Clone)]
pub struct SmartStr {
    repr: Repr,
}

impl SmartStr {
    #[inline]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        SmartStr {
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

impl Default for SmartStr {
    fn default() -> Self {
        SmartStr::new("")
    }
}

impl Deref for SmartStr {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for SmartStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for SmartStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Eq for SmartStr {}

impl<T: AsRef<str>> PartialEq<T> for SmartStr {
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<SmartStr> for String {
    fn eq(&self, other: &SmartStr) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Ord for SmartStr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for SmartStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for SmartStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<'a> From<&'a str> for SmartStr {
    fn from(s: &'a str) -> Self {
        SmartStr::new(s)
    }
}

impl From<String> for SmartStr {
    fn from(s: String) -> Self {
        SmartStr::new(&s)
    }
}

impl<'a> From<&'a String> for SmartStr {
    fn from(s: &'a String) -> Self {
        SmartStr::new(&s)
    }
}

impl fmt::Debug for SmartStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for SmartStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}
