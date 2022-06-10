//! [`CompactString`] is a compact string type that stores itself on the stack if possible,
//! otherwise known as a "small string optimization".
//!
//! ### Memory Layout
//! Normally strings are stored on the heap, since they're dynamically sized. In Rust a [`String`]
//! consists of three things:
//! 1. A `usize` denoting the length of the string
//! 2. A pointer to a location on the heap where the string is stored
//! 3. A `usize` denoting the capacity of the string
//!
//! On 64-bit architectures this results in 24 bytes being stored on the stack (12 bytes for 32-bit
//! architectures). For small strings, e.g. <= 24 characters, instead of storing a pointer, length,
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
use core::ops::{
    Add,
    Deref,
};
use core::str::{
    FromStr,
    Utf8Error,
};
use std::borrow::Cow;
use std::ffi::OsStr;

mod asserts;
mod features;
mod utility;

mod repr;
use repr::Repr;

mod traits;
pub use traits::ToCompactString;

#[cfg(test)]
mod tests;

/// A [`CompactString`] is a compact string type that can be used almost anywhere a
/// [`String`] or [`str`] can be used.
///
/// ## Using `CompactString`
/// ```
/// use compact_str::CompactString;
/// # use std::collections::HashMap;
///
/// // CompactString auto derefs into a str so you can use all methods from `str`
/// // that take a `&self`
/// if CompactString::new("hello world!").is_ascii() {
///     println!("we're all ASCII")
/// }
///
/// // You can use a CompactString in collections like you would a String or &str
/// let mut map: HashMap<CompactString, CompactString> = HashMap::new();
///
/// // directly construct a new `CompactString`
/// map.insert(CompactString::new("nyc"), CompactString::new("empire state building"));
/// // create a `CompactString` from a `&str`
/// map.insert("sf".into(), "transamerica pyramid".into());
/// // create a `CompactString` from a `String`
/// map.insert(String::from("sea").into(), String::from("space needle").into());
///
/// fn wrapped_print<T: AsRef<str>>(text: T) {
///     println!("{}", text.as_ref());
/// }
///
/// // CompactString impls AsRef<str> and Borrow<str>, so it can be used anywhere
/// // that excepts a generic string
/// if let Some(building) = map.get("nyc") {
///     wrapped_print(building);
/// }
///
/// // CompactString can also be directly compared to a String or &str
/// assert_eq!(CompactString::new("chicago"), "chicago");
/// assert_eq!(CompactString::new("houston"), String::from("houston"));
/// ```
#[derive(Clone)]
pub struct CompactString {
    repr: Repr,
}

/// # DEPRECATED
/// Renamed `CompactStr` to [`CompactString`]. Using the suffix "String" as opposed to "Str" more
/// accurately reflects that we own the underlying string.
///
/// Type alias `CompactStr` will be removed in v0.5
#[deprecated(
    since = "0.4.0",
    note = "Renamed to CompactString, type alias will be removed in v0.5"
)]
pub type CompactStr = CompactString;

impl CompactString {
    /// Creates a new [`CompactString`] from any type that implements `AsRef<str>`.
    /// If the string is short enough, then it will be inlined on the stack!
    ///
    /// # Examples
    ///
    /// ### Inlined
    /// ```
    /// # use compact_str::CompactString;
    /// // We can inline strings up to 12 characters long on 32-bit architectures...
    /// #[cfg(target_pointer_width = "32")]
    /// let s = "i'm 12 chars";
    /// // ...and up to 24 characters on 64-bit architectures!
    /// #[cfg(target_pointer_width = "64")]
    /// let s = "i am 24 characters long!";
    ///
    /// let compact = CompactString::new(&s);
    ///
    /// assert_eq!(compact, s);
    /// // we are not allocated on the heap!
    /// assert!(!compact.is_heap_allocated());
    /// ```
    ///
    /// ### Heap
    /// ```
    /// # use compact_str::CompactString;
    /// // For longer strings though, we get allocated on the heap
    /// let long = "I am a longer string that will be allocated on the heap";
    /// let compact = CompactString::new(long);
    ///
    /// assert_eq!(compact, long);
    /// // we are allocated on the heap!
    /// assert!(compact.is_heap_allocated());
    /// ```
    ///
    /// ### Creation
    /// ```
    /// use compact_str::CompactString;
    ///
    /// // Using a `&'static str`
    /// let s = "hello world!";
    /// let hello = CompactString::new(&s);
    ///
    /// // Using a `String`
    /// let u = String::from("🦄🌈");
    /// let unicorn = CompactString::new(u);
    ///
    /// // Using a `Box<str>`
    /// let b: Box<str> = String::from("📦📦📦").into_boxed_str();
    /// let boxed = CompactString::new(&b);
    /// ```
    #[inline]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        CompactString {
            repr: Repr::new(text),
        }
    }

    /// Creates a new inline [`CompactString`] at compile time.
    ///
    /// # Examples
    /// ```
    /// use compact_str::CompactString;
    ///
    /// const DEFAULT_NAME: CompactString = CompactString::new_inline("untitled");
    /// ```
    ///
    /// Note: Trying to create a long string that can't be inlined, will fail to build.
    /// ```compile_fail
    /// # use compact_str::CompactString;
    /// const LONG: CompactString = CompactString::new_inline("this is a long string that can't be stored on the stack");
    /// ```
    #[inline]
    pub const fn new_inline(text: &str) -> Self {
        CompactString {
            repr: Repr::new_const(text),
        }
    }

    /// Creates a new empty [`CompactString`] with the capacity to fit at least `capacity` bytes.
    ///
    /// A `CompactString` will inline strings on the stack, if they're small enough. Specifically,
    /// if the string has a length less than or equal to `std::mem::size_of::<String>` bytes
    /// then it will be inlined. This also means that `CompactString`s have a minimum capacity
    /// of `std::mem::size_of::<String>`.
    ///
    /// # Examples
    ///
    /// ### "zero" Capacity
    /// ```
    /// # use compact_str::CompactString;
    /// // Creating a CompactString with a capacity of 0 will create
    /// // one with capacity of std::mem::size_of::<String>();
    /// let empty = CompactString::with_capacity(0);
    /// let min_size = std::mem::size_of::<String>();
    ///
    /// assert_eq!(empty.capacity(), min_size);
    /// assert_ne!(0, min_size);
    /// assert!(!empty.is_heap_allocated());
    /// ```
    ///
    /// ### Max Inline Size
    /// ```
    /// # use compact_str::CompactString;
    /// // Creating a CompactString with a capacity of std::mem::size_of::<String>()
    /// // will not heap allocate.
    /// let str_size = std::mem::size_of::<String>();
    /// let empty = CompactString::with_capacity(str_size);
    ///
    /// assert_eq!(empty.capacity(), str_size);
    /// assert!(!empty.is_heap_allocated());
    /// ```
    ///
    /// ### Heap Allocating
    /// ```
    /// # use compact_str::CompactString;
    /// // If you create a `CompactString` with a capacity greater than
    /// // `std::mem::size_of::<String>`, it will heap allocated
    ///
    /// let heap_size = std::mem::size_of::<String>() + 1;
    /// let empty = CompactString::with_capacity(heap_size);
    ///
    /// assert_eq!(empty.capacity(), heap_size);
    /// assert!(empty.is_heap_allocated());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        CompactString {
            repr: Repr::with_capacity(capacity),
        }
    }

    /// Convert a slice of bytes into a [`CompactString`].
    ///
    /// A [`CompactString`] is a contiguous collection of bytes (`u8`s) that is valid [`UTF-8`](https://en.wikipedia.org/wiki/UTF-8).
    /// This method converts from an arbitrary contiguous collection of bytes into a
    /// [`CompactString`], failing if the provided bytes are not `UTF-8`.
    ///
    /// Note: If you want to create a [`CompactString`] from a non-contiguous collection of bytes,
    /// enable the `bytes` feature of this crate, and see `CompactString::from_utf8_buf`
    ///
    /// # Examples
    /// ### Valid UTF-8
    /// ```
    /// # use compact_str::CompactString;
    /// let bytes = vec![240, 159, 166, 128, 240, 159, 146, 175];
    /// let compact = CompactString::from_utf8(bytes).expect("valid UTF-8");
    ///
    /// assert_eq!(compact, "🦀💯");
    /// ```
    ///
    /// ### Invalid UTF-8
    /// ```
    /// # use compact_str::CompactString;
    /// let bytes = vec![255, 255, 255];
    /// let result = CompactString::from_utf8(bytes);
    ///
    /// assert!(result.is_err());
    /// ```
    #[inline]
    pub fn from_utf8<B: AsRef<[u8]>>(buf: B) -> Result<Self, Utf8Error> {
        let repr = Repr::from_utf8(buf)?;
        Ok(CompactString { repr })
    }

    /// Returns the length of the [`CompactString`] in `bytes`, not [`char`]s or graphemes.
    ///
    /// When using `UTF-8` encoding (which all strings in Rust do) a single character will be 1 to 4
    /// bytes long, therefore the return value of this method might not be what a human considers
    /// the length of the string.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    /// let ascii = CompactString::new("hello world");
    /// assert_eq!(ascii.len(), 11);
    ///
    /// let emoji = CompactString::new("👱");
    /// assert_eq!(emoji.len(), 4);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.repr.len()
    }

    /// Returns `true` if the [`CompactString`] has a length of 0, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    /// let mut msg = CompactString::new("");
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

    /// Returns the capacity of the [`CompactString`], in bytes.
    ///
    /// # Note
    /// * A `CompactString` will always have a capacity of at least `std::mem::size_of::<String>()`
    ///
    /// # Examples
    /// ### Minimum Size
    /// ```
    /// # use compact_str::CompactString;
    /// let min_size = std::mem::size_of::<String>();
    /// let compact = CompactString::new("");
    ///
    /// assert!(compact.capacity() >= min_size);
    /// ```
    ///
    /// ### Heap Allocated
    /// ```
    /// # use compact_str::CompactString;
    /// let compact = CompactString::with_capacity(128);
    /// assert_eq!(compact.capacity(), 128);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.repr.capacity()
    }

    /// Ensures that this [`CompactString`]'s capacity is at least `additional` bytes longer than
    /// its length. The capacity may be increased by more than `additional` bytes if it chooses,
    /// to prevent frequent reallocations.
    ///
    /// # Note
    /// * A `CompactString` will always have at least a capacity of `std::mem::size_of::<String>()`
    /// * Reserving additional bytes may cause the `CompactString` to become heap allocated
    ///
    /// # Panics
    /// Panics if the new capacity overflows `usize`
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    ///
    /// const WORD: usize = std::mem::size_of::<usize>();
    /// let mut compact = CompactString::default();
    /// assert!(compact.capacity() >= (WORD * 3) - 1);
    ///
    /// compact.reserve(200);
    /// assert!(compact.is_heap_allocated());
    /// assert!(compact.capacity() >= 200);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.repr.reserve(additional)
    }

    /// Returns a string slice containing the entire [`CompactString`].
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    /// let s = CompactString::new("hello");
    ///
    /// assert_eq!(s.as_str(), "hello");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        self.repr.as_str()
    }

    /// Returns a byte slice of the [`CompactString`]'s contents.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    /// let s = CompactString::new("hello");
    ///
    /// assert_eq!(&[104, 101, 108, 108, 111], s.as_bytes());
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.repr.as_slice()[..self.len()]
    }

    // TODO: Implement a `try_as_mut_slice(...)` that will fail if it results in cloning?
    //
    /// Provides a mutable reference to the underlying buffer of bytes.
    ///
    /// # Safety
    /// * All Rust strings, including `CompactString`, must be valid UTF-8. The caller must
    ///   guarantee
    /// that any modifications made to the underlying buffer are valid UTF-8.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("hello");
    ///
    /// let slice = unsafe { s.as_mut_bytes() };
    /// // copy bytes into our string
    /// slice[5..11].copy_from_slice(" world".as_bytes());
    /// // set the len of the string
    /// unsafe { s.set_len(11) };
    ///
    /// assert_eq!(s, "hello world");
    /// ```
    #[inline]
    pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.repr.as_mut_slice()
    }

    /// Appends the given [`char`] to the end of this [`CompactString`].
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("foo");
    ///
    /// s.push('b');
    /// s.push('a');
    /// s.push('r');
    ///
    /// assert_eq!("foobar", s);
    /// ```
    #[inline]
    pub fn push(&mut self, ch: char) {
        self.repr.push(ch)
    }

    /// Removes the last character from the [`CompactString`] and returns it.
    /// Returns `None` if this [`CompactString`] is empty.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("abc");
    ///
    /// assert_eq!(s.pop(), Some('c'));
    /// assert_eq!(s.pop(), Some('b'));
    /// assert_eq!(s.pop(), Some('a'));
    ///
    /// assert_eq!(s.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        self.repr.pop()
    }

    /// Appends a given string slice onto the end of this [`CompactString`]
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("abc");
    ///
    /// s.push_str("123");
    ///
    /// assert_eq!("abc123", s);
    /// ```
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.repr.push_str(s)
    }

    /// Forces the length of the [`CompactString`] to `new_len`.
    ///
    /// This is a low-level operation that maintains none of the normal invariants for
    /// `CompactString`. If you want to modify the `CompactString` you should use methods like
    /// `push`, `push_str` or `pop`.
    ///
    /// # Safety
    /// * `new_len` must be less than or equal to `capacity()`
    /// * The elements at `old_len..new_len` must be initialized
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.repr.set_len(new_len)
    }

    /// Returns whether or not the [`CompactString`] is heap allocated.
    ///
    /// # Examples
    /// ### Inlined
    /// ```
    /// # use compact_str::CompactString;
    /// let hello = CompactString::new("hello world");
    ///
    /// assert!(!hello.is_heap_allocated());
    /// ```
    ///
    /// ### Heap Allocated
    /// ```
    /// # use compact_str::CompactString;
    /// let msg = CompactString::new("this message will self destruct in 5, 4, 3, 2, 1 💥");
    ///
    /// assert!(msg.is_heap_allocated());
    /// ```
    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        self.repr.is_heap_allocated()
    }
}

impl Default for CompactString {
    #[inline]
    fn default() -> Self {
        CompactString::new("")
    }
}

impl Deref for CompactString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for CompactString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<OsStr> for CompactString {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.as_str())
    }
}

impl Borrow<str> for CompactString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Eq for CompactString {}

impl<T: AsRef<str>> PartialEq<T> for CompactString {
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<CompactString> for String {
    fn eq(&self, other: &CompactString) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<CompactString> for &str {
    fn eq(&self, other: &CompactString) -> bool {
        *self == other.as_str()
    }
}

impl<'a> PartialEq<CompactString> for Cow<'a, str> {
    fn eq(&self, other: &CompactString) -> bool {
        *self == other.as_str()
    }
}

impl Ord for CompactString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for CompactString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for CompactString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<'a> From<&'a str> for CompactString {
    fn from(s: &'a str) -> Self {
        CompactString::new(s)
    }
}

impl From<String> for CompactString {
    fn from(s: String) -> Self {
        let repr = Repr::from_string(s);
        CompactString { repr }
    }
}

impl<'a> From<&'a String> for CompactString {
    fn from(s: &'a String) -> Self {
        CompactString::new(&s)
    }
}

impl<'a> From<Cow<'a, str>> for CompactString {
    fn from(cow: Cow<'a, str>) -> Self {
        match cow {
            Cow::Borrowed(s) => s.into(),
            Cow::Owned(s) => s.into(),
        }
    }
}

impl From<Box<str>> for CompactString {
    fn from(b: Box<str>) -> Self {
        let repr = Repr::from_box_str(b);
        CompactString { repr }
    }
}

impl FromStr for CompactString {
    type Err = core::convert::Infallible;
    fn from_str(s: &str) -> Result<CompactString, Self::Err> {
        Ok(CompactString::from(s))
    }
}

impl fmt::Debug for CompactString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for CompactString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl FromIterator<char> for CompactString {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactString { repr }
    }
}

impl<'a> FromIterator<&'a char> for CompactString {
    fn from_iter<T: IntoIterator<Item = &'a char>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactString { repr }
    }
}

impl<'a> FromIterator<&'a str> for CompactString {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactString { repr }
    }
}

impl FromIterator<Box<str>> for CompactString {
    fn from_iter<T: IntoIterator<Item = Box<str>>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactString { repr }
    }
}

impl FromIterator<String> for CompactString {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactString { repr }
    }
}

impl Extend<char> for CompactString {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

impl<'a> Extend<&'a char> for CompactString {
    fn extend<T: IntoIterator<Item = &'a char>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

impl<'a> Extend<&'a str> for CompactString {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

impl Extend<Box<str>> for CompactString {
    fn extend<T: IntoIterator<Item = Box<str>>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

impl<'a> Extend<Cow<'a, str>> for CompactString {
    fn extend<T: IntoIterator<Item = Cow<'a, str>>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl Extend<String> for CompactString {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

impl fmt::Write for CompactString {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl Add<Self> for CompactString {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.push_str(&rhs);
        self
    }
}

impl Add<&Self> for CompactString {
    type Output = Self;
    fn add(mut self, rhs: &Self) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl Add<&str> for CompactString {
    type Output = Self;
    fn add(mut self, rhs: &str) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl Add<&String> for CompactString {
    type Output = Self;
    fn add(mut self, rhs: &String) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl Add<String> for CompactString {
    type Output = Self;
    fn add(mut self, rhs: String) -> Self::Output {
        self.push_str(&rhs);
        self
    }
}

impl Add<CompactString> for String {
    type Output = Self;
    fn add(mut self, rhs: CompactString) -> Self::Output {
        self.push_str(&rhs);
        self
    }
}

crate::asserts::assert_size_eq!(CompactString, String);
