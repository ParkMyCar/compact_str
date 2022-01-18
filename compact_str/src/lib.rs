//! `CompactStr` is a compact string type that stores itself on the stack if possible, otherwise
//! known as a "small string optimization".
//!
//! ### Memory Layout
//! Normally strings are stored on the heap, since they're dynamically sized. In Rust a `String`
//! consists of three things:
//! 1. A `usize` denoting the length of the string
//! 2. A pointer to a location on the heap where the string is stored
//! 3. A `usize` denoting the capacity of the string
//!
//! On 64-bit architectures this results in 24 bytes being stored on the stack (12 bytes for 32-bit
//! architectures). For small strings, e.g. <= 23 characters, instead of storing a pointer, length,
//! and capacity on the stack, you store the string itself! This avoids the need to heap allocate
//! which reduces the amount of memory used, and improves performance.

use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{
    Hash,
    Hasher,
};
use core::iter::FromIterator;
use core::ops::Deref;
use core::str::FromStr;

mod features;

mod repr;
use repr::Repr;

#[cfg(test)]
mod tests;

/// A `CompactStr` is a compact string tupe that can be used almost anywhere a `String` or `&str`
/// can be used.
///
/// ## Using `CompactStr`
/// ```
/// use compact_str::CompactStr;
/// # use std::collections::HashMap;
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
    /// Creates a new `CompactStr` from any type that implements `AsRef<str>`. If the string is
    /// short enough, then it will be inlined on the stack!
    ///
    /// # Examples
    /// ```
    /// use compact_str::CompactStr;
    ///
    /// // Using a `&'static str`
    /// let s = "hello world!";
    /// let hello = CompactStr::new(&s);
    ///
    /// // Using a `String`
    /// let u = String::from("🦄🌈");
    /// let unicorn = CompactStr::new(u);
    ///
    /// // Using a `Box<str>`
    /// let b: Box<str> = String::from("📦📦📦").into_boxed_str();
    /// let boxed = CompactStr::new(&b);
    /// ```
    ///
    /// ### Inlined
    /// ```
    /// # use compact_str::CompactStr;
    /// // We can inline strings up to 12 characters long on 32-bit architectures...
    /// #[cfg(target_pointer_width = "32")]
    /// let s = "i'm 12 chars";
    /// // ...and up to 24 characters on 64-bit architectures!
    /// #[cfg(target_pointer_width = "64")]
    /// let s = "i am 24 characters long!";
    ///
    /// let compact = CompactStr::new(&s);
    ///
    /// assert_eq!(compact, s);
    /// // we are not allocated on the heap!
    /// assert!(!compact.is_heap_allocated());
    /// ```
    ///
    /// ### Heap
    /// ```
    /// # use compact_str::CompactStr;
    /// // For longer strings though, we get allocated on the heap
    /// let long = "I am a longer string that will be allocated on the heap";
    /// let compact = CompactStr::new(long);
    ///
    /// assert_eq!(compact, long);
    /// // we are allocated on the heap!
    /// assert!(compact.is_heap_allocated());
    /// ```
    #[inline]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        CompactStr {
            repr: Repr::new(text),
        }
    }

    /// Creates a new inline `CompactStr` at compile time.
    ///
    /// # Examples
    /// ```
    /// use compact_str::CompactStr;
    ///
    /// const DEFAULT_NAME: CompactStr = CompactStr::new_inline("untitled");
    /// ```
    ///
    /// Note: Trying to create a long string that can't be inlined, will fail to build.
    /// ```compile_fail
    /// # use compact_str::CompactStr;
    ///
    /// const LONG: CompactStr = CompactStr::new_inline("this is a long string that can't be stored on the stack");
    /// ```
    #[inline]
    pub const fn new_inline(text: &str) -> Self {
        CompactStr {
            repr: Repr::new_const(text),
        }
    }

    /// Creates a new empty `CompactStr` with the provided capacity.
    ///
    /// A `CompactStr` will inline strings on the stack, if they're small enough. Specifically, if
    /// the string is smaller than `std::mem::size_of::<String>` bytes then it will be inlined. This
    /// means that `CompactStr`s have a minimum capacity of `std::mem::size_of::<String> - 1`, e.g.
    /// ```
    /// # use compact_str::CompactStr;
    /// let empty = CompactStr::with_capacity(0);
    /// let min_size = std::mem::size_of::<String>() - 1;
    ///
    /// assert_eq!(empty.capacity(), min_size);
    /// assert_ne!(0, min_size);
    /// assert!(!empty.is_heap_allocated());
    /// ```
    ///
    /// If you create a `CompactStr` with a capacity greater than or equal to
    /// `std::mem::size_of::<String>`, it will heap allocated, e.g.
    /// ```
    /// # use compact_str::CompactStr;
    /// let heap_size = std::mem::size_of::<String>();
    /// let empty = CompactStr::with_capacity(heap_size);
    ///
    /// assert_eq!(empty.capacity(), heap_size);
    /// assert!(empty.is_heap_allocated());
    /// ```
    ///
    /// # Note
    /// A `CompactStr` can inline strings that are equal to `std::mem::size_of::<String>()`, if the
    /// first character is ASCII. But we
    /// do this by foregoing any metadata to track the string's length, and using the invariant
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        CompactStr {
            repr: Repr::with_capacity(capacity),
        }
    }

    /// Returns the length of the `CompactStr` in `bytes`, not `chars` or graphemes.
    ///
    /// When using UTF-8 encoding (which all strings in Rust do) a single character will be 1 - 4
    /// bytes long, therefore the return value of this method might not be what a human considers
    /// the length of the string.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactStr;
    /// let ascii = CompactStr::new("hello world");
    /// assert_eq!(ascii.len(), 11);
    ///
    /// let person = CompactStr::new("👱");
    /// assert_eq!(person.len(), 4);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.repr.len()
    }

    /// Returns `true` if the `CompactStr` has a length of 0, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactStr;
    /// let mut msg = CompactStr::new("");
    /// assert!(msg.is_empty());
    ///
    /// // add some characters
    /// msg.push_str("hello reader!");
    /// assert!(!msg.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the capacity of the `CompactStr`, in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.repr.capacity()
    }

    /// Ensures that this `CompactStr`'s capacity is at least `additional` bytes longer than its
    /// length. The capacity may be increased by more than `additional` bytes if it chooses, to
    /// prevent frequent reallocations.
    ///
    /// # Note
    /// * A `CompactStr` will always have at least a capacity of `(WORD * 3) - 1`
    /// * Reserving additional bytes may cause the `CompactStr` to become heap allocated
    ///
    /// ### For example:
    /// ```
    /// # use compact_str::CompactStr;
    ///
    /// const WORD: usize = std::mem::size_of::<usize>();
    /// let mut compact = CompactStr::default();
    /// assert!(compact.capacity() >= (WORD * 3) - 1);
    ///
    /// compact.reserve(200);
    /// assert!(compact.is_heap_allocated());
    /// assert!(compact.capacity() >= 200);
    /// ```
    ///
    /// # Panics
    /// Panics if the new capacity overflows `usize`
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.repr.reserve(additional)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.repr.as_str()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.repr.as_slice()
    }

    // TODO: Implement a `try_as_mut_slice(...)` that will fail if it results in cloning?
    //
    /// Provides a mutable reference to the underlying buffer of bytes.
    ///
    /// Note: If the given `CompactStr` is heap allocated, _and_ multiple references exist to the
    /// underlying buffer (e.g. you previously cloned this `CompactStr`), calling this method will
    /// clone the entire buffer to prevent silently mutating other owned `CompactStr`s.
    ///
    /// ### Futher Explanation
    /// When a `CompactStr` becomes sufficiently large, the underlying buffer becomes a reference
    /// counted buffer on the heap. Then, cloning a `CompactStr` increments a reference count
    /// instead of cloning the entire buffer (very similar to `Arc<str>`). To prevent silently
    /// mutating the data of other owned `CompactStr`s when taking a mutable slice, we clone the
    /// underlying buffer and mutate that, if more than one outstanding reference exists.
    ///
    /// # Safety
    /// * All Rust strings, including `CompactStr`, must be valid UTF-8. The caller must guarantee
    /// that any modifications made to the underlying buffer are valid UTF-8.
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        self.repr.as_mut_slice()
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        self.repr.push(ch)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        self.repr.pop()
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.repr.push_str(s)
    }

    /// # Safety
    /// * TODO: Document safety here
    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        self.repr.set_len(length)
    }

    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        self.repr.is_heap_allocated()
    }
}

impl Default for CompactStr {
    #[inline]
    fn default() -> Self {
        CompactStr::new("")
    }
}

impl Deref for CompactStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for CompactStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for CompactStr {
    #[inline]
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

impl PartialEq<CompactStr> for &str {
    fn eq(&self, other: &CompactStr) -> bool {
        *self == other.as_str()
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

impl FromStr for CompactStr {
    type Err = core::convert::Infallible;
    fn from_str(s: &str) -> Result<CompactStr, Self::Err> {
        Ok(CompactStr::from(s))
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

impl FromIterator<char> for CompactStr {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactStr { repr }
    }
}

impl<'a> FromIterator<&'a char> for CompactStr {
    fn from_iter<T: IntoIterator<Item = &'a char>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactStr { repr }
    }
}

impl<'a> FromIterator<&'a str> for CompactStr {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactStr { repr }
    }
}

impl FromIterator<Box<str>> for CompactStr {
    fn from_iter<T: IntoIterator<Item = Box<str>>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactStr { repr }
    }
}

impl FromIterator<String> for CompactStr {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactStr { repr }
    }
}

impl Extend<char> for CompactStr {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

impl<'a> Extend<&'a char> for CompactStr {
    fn extend<T: IntoIterator<Item = &'a char>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

impl<'a> Extend<&'a str> for CompactStr {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

impl Extend<Box<str>> for CompactStr {
    fn extend<T: IntoIterator<Item = Box<str>>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

impl Extend<String> for CompactStr {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

static_assertions::assert_eq_size!(CompactStr, String);
