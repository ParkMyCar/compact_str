//! [`CompactStr`] is a compact string type that stores itself on the stack if possible, otherwise
//! known as a "small string optimization".
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
use core::hash::{Hash, Hasher};
use core::iter::FromIterator;
use core::ops::{Add, Deref};
use core::str::{FromStr, Utf8Error};
use std::borrow::Cow;

mod asserts;
mod features;
mod utility;

mod repr;
use repr::Repr;

mod to_compact_str_specialisation;

#[cfg(test)]
mod tests;

/// A [`CompactStr`] is a compact string type that can be used almost anywhere a
/// [`String`] or [`str`] can be used.
///
/// ## Using `CompactStr`
/// ```
/// use compact_str::CompactStr;
/// # use std::collections::HashMap;
///
/// // CompactStr auto derefs into a str so you can use all methods from `str`
/// // that take a `&self`
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
/// // CompactStr impls AsRef<str> and Borrow<str>, so it can be used anywhere
/// // that excepts a generic string
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
    /// Creates a new [`CompactStr`] from any type that implements `AsRef<str>`.
    /// If the string is short enough, then it will be inlined on the stack!
    ///
    /// # Examples
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
    ///
    /// ### Creation
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
    #[inline]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        CompactStr {
            repr: Repr::new(text),
        }
    }

    /// Creates a new inline [`CompactStr`] at compile time.
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
    /// const LONG: CompactStr = CompactStr::new_inline("this is a long string that can't be stored on the stack");
    /// ```
    #[inline]
    pub const fn new_inline(text: &str) -> Self {
        CompactStr {
            repr: Repr::new_const(text),
        }
    }

    /// Creates a new empty [`CompactStr`] with the capacity to fit at least `capacity` bytes.
    ///
    /// A `CompactStr` will inline strings on the stack, if they're small enough. Specifically, if
    /// the string has a length less than or equal to `std::mem::size_of::<String>` bytes then it
    /// will be inlined. This also means that `CompactStr`s have a minimum capacity of
    /// `std::mem::size_of::<String>`.
    ///
    /// # Examples
    ///
    /// ### "zero" Capacity
    /// ```
    /// # use compact_str::CompactStr;
    /// // Creating a CompactStr with a capacity of 0 will create
    /// // one with capacity of std::mem::size_of::<String>();
    /// let empty = CompactStr::with_capacity(0);
    /// let min_size = std::mem::size_of::<String>();
    ///
    /// assert_eq!(empty.capacity(), min_size);
    /// assert_ne!(0, min_size);
    /// assert!(!empty.is_heap_allocated());
    /// ```
    ///
    /// ### Max Inline Size
    /// ```
    /// # use compact_str::CompactStr;
    /// // Creating a CompactStr with a capacity of std::mem::size_of::<String>()
    /// // will not heap allocate.
    /// let str_size = std::mem::size_of::<String>();
    /// let empty = CompactStr::with_capacity(str_size);
    ///
    /// assert_eq!(empty.capacity(), str_size);
    /// assert!(!empty.is_heap_allocated());
    /// ```
    ///
    /// ### Heap Allocating
    /// ```
    /// # use compact_str::CompactStr;
    /// // If you create a `CompactStr` with a capacity greater than
    /// // `std::mem::size_of::<String>`, it will heap allocated
    ///
    /// let heap_size = std::mem::size_of::<String>() + 1;
    /// let empty = CompactStr::with_capacity(heap_size);
    ///
    /// assert_eq!(empty.capacity(), heap_size);
    /// assert!(empty.is_heap_allocated());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        CompactStr {
            repr: Repr::with_capacity(capacity),
        }
    }

    /// Convert a slice of bytes into a [`CompactStr`].
    ///
    /// A [`CompactStr`] is a contiguous collection of bytes (`u8`s) that is valid [`UTF-8`](https://en.wikipedia.org/wiki/UTF-8).
    /// This method converts from an arbitrary contiguous collection of bytes into a [`CompactStr`],
    /// failing if the provided bytes are not `UTF-8`.
    ///
    /// Note: If you want to create a [`CompactStr`] from a non-contiguous collection of bytes,
    /// enable the `bytes` feature of this crate, and checkout [`CompactStr::from_utf8_buf`]
    ///
    /// # Examples
    /// ### Valid UTF-8
    /// ```
    /// # use compact_str::CompactStr;
    /// let bytes = vec![240, 159, 166, 128, 240, 159, 146, 175];
    /// let compact = CompactStr::from_utf8(bytes).expect("valid UTF-8");
    ///
    /// assert_eq!(compact, "🦀💯");
    /// ```
    ///
    /// ### Invalid UTF-8
    /// ```
    /// # use compact_str::CompactStr;
    /// let bytes = vec![255, 255, 255];
    /// let result = CompactStr::from_utf8(bytes);
    ///
    /// assert!(result.is_err());
    /// ```
    #[inline]
    pub fn from_utf8<B: AsRef<[u8]>>(buf: B) -> Result<Self, Utf8Error> {
        let repr = Repr::from_utf8(buf)?;
        Ok(CompactStr { repr })
    }

    /// Returns the length of the [`CompactStr`] in `bytes`, not [`char`]s or graphemes.
    ///
    /// When using `UTF-8` encoding (which all strings in Rust do) a single character will be 1 to 4
    /// bytes long, therefore the return value of this method might not be what a human considers
    /// the length of the string.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactStr;
    /// let ascii = CompactStr::new("hello world");
    /// assert_eq!(ascii.len(), 11);
    ///
    /// let emoji = CompactStr::new("👱");
    /// assert_eq!(emoji.len(), 4);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.repr.len()
    }

    /// Returns `true` if the [`CompactStr`] has a length of 0, `false` otherwise
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

    /// Returns the capacity of the [`CompactStr`], in bytes.
    ///
    /// # Note
    /// * A `CompactStr` will always have a capacity of at least `std::mem::size_of::<String>()`
    ///
    /// # Examples
    /// ### Minimum Size
    /// ```
    /// # use compact_str::CompactStr;
    /// let min_size = std::mem::size_of::<String>();
    /// let compact = CompactStr::new("");
    ///
    /// assert!(compact.capacity() >= min_size);
    /// ```
    ///
    /// ### Heap Allocated
    /// ```
    /// # use compact_str::CompactStr;
    /// let compact = CompactStr::with_capacity(128);
    /// assert_eq!(compact.capacity(), 128);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.repr.capacity()
    }

    /// Ensures that this [`CompactStr`]'s capacity is at least `additional` bytes longer than its
    /// length. The capacity may be increased by more than `additional` bytes if it chooses, to
    /// prevent frequent reallocations.
    ///
    /// # Note
    /// * A `CompactStr` will always have at least a capacity of `std::mem::size_of::<String>()`
    /// * Reserving additional bytes may cause the `CompactStr` to become heap allocated
    ///
    /// # Panics
    /// Panics if the new capacity overflows `usize`
    ///
    /// # Examples
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

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.repr.reserve(additional)
    }

    /// Returns a string slice containing the entire [`CompactStr`].
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactStr;
    /// let s = CompactStr::new("hello");
    ///
    /// assert_eq!(s.as_str(), "hello");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        self.repr.as_str()
    }

    /// Returns a byte slice of the [`CompactStr`]'s contents.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactStr;
    /// let s = CompactStr::new("hello");
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
    /// * All Rust strings, including `CompactStr`, must be valid UTF-8. The caller must guarantee
    /// that any modifications made to the underlying buffer are valid UTF-8.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactStr;
    /// let mut s = CompactStr::new("hello");
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

    /// Appends the given [`char`] to the end of this [`CompactStr`].
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactStr;
    /// let mut s = CompactStr::new("foo");
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

    /// Removes the last character from the [`CompactStr`] and returns it.
    /// Returns `None` if this `ComapctStr` is empty.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactStr;
    /// let mut s = CompactStr::new("abc");
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

    /// Appends a given string slice onto the end of this [`CompactStr`]
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactStr;
    /// let mut s = CompactStr::new("abc");
    ///
    /// s.push_str("123");
    ///
    /// assert_eq!("abc123", s);
    /// ```
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.repr.push_str(s)
    }

    /// Forces the length of the [`CompactStr`] to `new_len`.
    ///
    /// This is a low-level operation that maintains none of the normal invariants for `CompactStr`.
    /// If you want to modify the `CompactStr` you should use methods like `push`, `push_str` or
    /// `pop`.
    ///
    /// # Safety
    /// * `new_len` must be less than or equal to `capacity()`
    /// * The elements at `old_len..new_len` must be initialized
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.repr.set_len(new_len)
    }

    /// Returns whether or not the [`CompactStr`] is heap allocated.
    ///
    /// # Examples
    /// ### Inlined
    /// ```
    /// # use compact_str::CompactStr;
    /// let hello = CompactStr::new("hello world");
    ///
    /// assert!(!hello.is_heap_allocated());
    /// ```
    ///
    /// ### Heap Allocated
    /// ```
    /// # use compact_str::CompactStr;
    /// let msg = CompactStr::new("this message will self destruct in 5, 4, 3, 2, 1 💥");
    ///
    /// assert!(msg.is_heap_allocated());
    /// ```
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

impl<'a> PartialEq<CompactStr> for Cow<'a, str> {
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
        let repr = Repr::from_string(s);
        CompactStr { repr }
    }
}

impl<'a> From<&'a String> for CompactStr {
    fn from(s: &'a String) -> Self {
        CompactStr::new(&s)
    }
}

impl<'a> From<Cow<'a, str>> for CompactStr {
    fn from(s: Cow<'a, str>) -> Self {
        CompactStr::new(s)
    }
}

impl From<Box<str>> for CompactStr {
    fn from(b: Box<str>) -> Self {
        let repr = Repr::from_box_str(b);
        CompactStr { repr }
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

impl<'a> Extend<Cow<'a, str>> for CompactStr {
    fn extend<T: IntoIterator<Item = Cow<'a, str>>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl Extend<String> for CompactStr {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.repr.extend(iter)
    }
}

impl fmt::Write for CompactStr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl Add<Self> for CompactStr {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.push_str(&rhs);
        self
    }
}

impl Add<&Self> for CompactStr {
    type Output = Self;
    fn add(mut self, rhs: &Self) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl Add<&str> for CompactStr {
    type Output = Self;
    fn add(mut self, rhs: &str) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl Add<&String> for CompactStr {
    type Output = Self;
    fn add(mut self, rhs: &String) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl Add<String> for CompactStr {
    type Output = Self;
    fn add(mut self, rhs: String) -> Self::Output {
        self.push_str(&rhs);
        self
    }
}

impl Add<CompactStr> for String {
    type Output = Self;
    fn add(mut self, rhs: CompactStr) -> Self::Output {
        self.push_str(&rhs);
        self
    }
}

crate::asserts::assert_size_eq!(CompactStr, String);

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
        if let Some(compact_str) = to_compact_str_specialisation::to_compact_str_specialised(self) {
            return compact_str;
        }

        CompactStr {
            repr: Repr::from_fmt(self),
        }
    }
}
