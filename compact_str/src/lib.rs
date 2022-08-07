#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::borrow::{
    Borrow,
    BorrowMut,
};
use core::cmp::Ordering;
use core::fmt;
use core::hash::{
    Hash,
    Hasher,
};
use core::iter::FromIterator;
use core::ops::{
    Add,
    AddAssign,
    Bound,
    Deref,
    DerefMut,
    RangeBounds,
};
use core::str::{
    FromStr,
    Utf8Error,
};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::iter::FusedIterator;

mod asserts;
mod features;
mod macros;
mod utility;

mod repr;
use repr::Repr;

mod traits;
pub use traits::{
    CompactStringExt,
    ToCompactString,
};

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
            repr: Repr::new_inline(text),
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

    /// Converts a vector of bytes to a [`CompactString`] without checking that the string contains
    /// valid UTF-8.
    ///
    /// See the safe version, [`CompactString::from_utf8`], for more details.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed to it are valid
    /// UTF-8. If this constraint is violated, it may cause memory unsafety issues with future users
    /// of the [`CompactString`], as the rest of the standard library assumes that
    /// [`CompactString`]s are valid UTF-8.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// // some bytes, in a vector
    /// let sparkle_heart = vec![240, 159, 146, 150];
    ///
    /// let sparkle_heart = unsafe {
    ///     CompactString::from_utf8_unchecked(sparkle_heart)
    /// };
    ///
    /// assert_eq!("💖", sparkle_heart);
    /// ```
    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked<B: AsRef<[u8]>>(buf: B) -> Self {
        let repr = Repr::from_utf8_unchecked(buf);
        CompactString { repr }
    }

    /// Decode a [`UTF-16`](https://en.wikipedia.org/wiki/UTF-16) slice of bytes into a
    /// [`CompactString`], returning an [`Err`] if the slice contains any invalid data.
    ///
    /// # Examples
    /// ### Valid UTF-16
    /// ```
    /// # use compact_str::CompactString;
    /// let buf: &[u16] = &[0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let compact = CompactString::from_utf16(buf).unwrap();
    ///
    /// assert_eq!(compact, "𝄞music");
    /// ```
    ///
    /// ### Invalid UTF-16
    /// ```
    /// # use compact_str::CompactString;
    /// let buf: &[u16] = &[0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// let res = CompactString::from_utf16(buf);
    ///
    /// assert!(res.is_err());
    /// ```
    #[inline]
    pub fn from_utf16<B: AsRef<[u16]>>(buf: B) -> Result<Self, Utf16Error> {
        // Note: we don't use collect::<Result<_, _>>() because that fails to pre-allocate a buffer,
        // even though the size of our iterator, `buf`, is known ahead of time.
        //
        // rustlang issue #48994 is tracking the fix

        let buf = buf.as_ref();
        let mut ret = CompactString::with_capacity(buf.len());
        for c in core::char::decode_utf16(buf.iter().copied()) {
            if let Ok(c) = c {
                ret.push(c);
            } else {
                return Err(Utf16Error(()));
            }
        }
        Ok(ret)
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

    /// Returns a mutable string slice containing the entire [`CompactString`].
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("hello");
    /// s.as_mut_str().make_ascii_uppercase();
    ///
    /// assert_eq!(s.as_str(), "HELLO");
    /// ```
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        let len = self.len();
        unsafe { std::str::from_utf8_unchecked_mut(&mut self.repr.as_mut_slice()[..len]) }
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
    pub fn push(&mut self, ch: char) {
        self.push_str(ch.encode_utf8(&mut [0; 4]));
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

    /// Removes a [`char`] from this [`CompactString`] at a byte position and returns it.
    ///
    /// This is an *O*(*n*) operation, as it requires copying every element in the
    /// buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than or equal to the [`CompactString`]'s length,
    /// or if it does not lie on a [`char`] boundary.
    ///
    /// # Examples
    ///
    /// ### Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut c = CompactString::from("hello world");
    ///
    /// assert_eq!(c.remove(0), 'h');
    /// assert_eq!(c, "ello world");
    ///
    /// assert_eq!(c.remove(5), 'w');
    /// assert_eq!(c, "ello orld");
    /// ```
    ///
    /// ### Past total length:
    ///
    /// ```should_panic
    /// # use compact_str::CompactString;
    /// let mut c = CompactString::from("hello there!");
    /// c.remove(100);
    /// ```
    ///
    /// ### Not on char boundary:
    ///
    /// ```should_panic
    /// # use compact_str::CompactString;
    /// let mut c = CompactString::from("🦄");
    /// c.remove(1);
    /// ```
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        let len = self.len();
        let substr = &mut self.as_mut_str()[idx..];

        // get the char we want to remove
        let ch = substr
            .chars()
            .next()
            .expect("cannot remove a char from the end of a string");
        let ch_len = ch.len_utf8();

        // shift everything back one character
        let num_bytes = substr.len() - ch_len;
        let ptr = substr.as_mut_ptr();

        // SAFETY: Both src and dest are valid for reads of `num_bytes` amount of bytes,
        // and are properly aligned
        unsafe {
            core::ptr::copy(ptr.add(ch_len) as *const u8, ptr, num_bytes);
            self.set_len(len - ch_len);
        }

        ch
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

    /// Ensure that the given range is inside the set data, and that no codepoints are split.
    ///
    /// Returns the range `start..end` as a tuple.
    #[inline]
    fn ensure_range(&self, range: impl RangeBounds<usize>) -> (usize, usize) {
        #[cold]
        #[inline(never)]
        fn illegal_range() -> ! {
            panic!("illegal range");
        }

        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => match n.checked_add(1) {
                Some(n) => n,
                None => illegal_range(),
            },
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => match n.checked_add(1) {
                Some(n) => n,
                None => illegal_range(),
            },
            Bound::Excluded(&n) => n,
            Bound::Unbounded => self.len(),
        };
        if end < start {
            illegal_range();
        }

        let s = self.as_str();
        if !s.is_char_boundary(start) || !s.is_char_boundary(end) {
            illegal_range();
        }

        (start, end)
    }

    /// Removes the specified range in the [`CompactString`],
    /// and replaces it with the given string.
    /// The given string doesn't need to be the same length as the range.
    ///
    /// # Panics
    ///
    /// Panics if the starting point or end point do not lie on a [`char`]
    /// boundary, or if they're out of bounds.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("Hello, world!");
    ///
    /// s.replace_range(7..12, "WORLD");
    /// assert_eq!(s, "Hello, WORLD!");
    ///
    /// s.replace_range(7..=11, "you");
    /// assert_eq!(s, "Hello, you!");
    ///
    /// s.replace_range(5.., "! Is it me you're looking for?");
    /// assert_eq!(s, "Hello! Is it me you're looking for?");
    /// ```
    #[inline]
    pub fn replace_range(&mut self, range: impl RangeBounds<usize>, replace_with: &str) {
        let (start, end) = self.ensure_range(range);
        let dest_len = end - start;
        match dest_len.cmp(&replace_with.len()) {
            Ordering::Equal => unsafe { self.replace_range_same_size(start, end, replace_with) },
            Ordering::Greater => unsafe { self.replace_range_shrink(start, end, replace_with) },
            Ordering::Less => unsafe { self.replace_range_grow(start, end, replace_with) },
        }
    }

    /// Replace into the same size.
    unsafe fn replace_range_same_size(&mut self, start: usize, end: usize, replace_with: &str) {
        core::ptr::copy_nonoverlapping(
            replace_with.as_ptr(),
            self.as_mut_ptr().add(start),
            end - start,
        );
    }

    /// Replace, so self.len() gets smaller.
    unsafe fn replace_range_shrink(&mut self, start: usize, end: usize, replace_with: &str) {
        let total_len = self.len();
        let dest_len = end - start;
        let new_len = total_len - (dest_len - replace_with.len());
        let amount = total_len - end;
        let data = self.as_mut_ptr();
        // first insert the replacement string, overwriting the current content
        core::ptr::copy_nonoverlapping(replace_with.as_ptr(), data.add(start), replace_with.len());
        // then move the tail of the CompactString forward to its new place, filling the gap
        core::ptr::copy(
            data.add(total_len - amount),
            data.add(new_len - amount),
            amount,
        );
        // and lastly we set the new length
        self.set_len(new_len);
    }

    /// Replace, so self.len() gets bigger.
    unsafe fn replace_range_grow(&mut self, start: usize, end: usize, replace_with: &str) {
        let dest_len = end - start;
        self.reserve(replace_with.len() - dest_len);
        let total_len = self.len();
        let new_len = total_len + (replace_with.len() - dest_len);
        let amount = total_len - end;
        // first grow the string, so MIRI knows that the full range is usable
        self.set_len(new_len);
        let data = self.as_mut_ptr();
        // then move the tail of the CompactString back to its new place
        core::ptr::copy(
            data.add(total_len - amount),
            data.add(new_len - amount),
            amount,
        );
        // and lastly insert the replacement string
        core::ptr::copy_nonoverlapping(replace_with.as_ptr(), data.add(start), replace_with.len());
    }

    /// Truncate the [`CompactString`] to a shorter length.
    ///
    /// If the length of the [`CompactString`] is less or equal to `new_len`, the call is a no-op.
    ///
    /// Calling this function does not change the capacity of the [`CompactString`].
    ///
    /// # Panics
    ///
    /// Panics if the new end of the string does not lie on a [`char`] boundary.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("Hello, world!");
    /// s.truncate(5);
    /// assert_eq!(s, "Hello");
    /// ```
    pub fn truncate(&mut self, new_len: usize) {
        let s = self.as_str();
        if new_len >= s.len() {
            return;
        }

        assert!(
            s.is_char_boundary(new_len),
            "new_len must lie on char boundary",
        );
        unsafe { self.set_len(new_len) };
    }

    /// Converts a [`CompactString`] to a raw pointer.
    #[inline]
    pub fn as_ptr(&mut self) -> *const u8 {
        self.repr.as_slice().as_ptr()
    }

    /// Converts a mutable [`CompactString`] to a raw pointer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        unsafe { self.repr.as_mut_slice().as_mut_ptr() }
    }

    /// Insert string character at an index.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("Hello!");
    /// s.insert_str(5, ", world");
    /// assert_eq!(s, "Hello, world!");
    /// ```
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        assert!(self.is_char_boundary(idx), "idx must lie on char boundary");

        let new_len = self.len() + string.len();
        self.reserve(string.len());

        // SAFETY: We just checked that we may split self at idx.
        //         We set the length only after reserving the memory.
        //         We fill the gap with valid UTF-8 data.
        unsafe {
            // first move the tail to the new back
            let data = self.as_mut_ptr();
            std::ptr::copy(
                data.add(idx),
                data.add(idx + string.len()),
                new_len - idx - string.len(),
            );

            // then insert the new bytes
            std::ptr::copy_nonoverlapping(string.as_ptr(), data.add(idx), string.len());

            // and lastly resize the string
            self.set_len(new_len);
        }
    }

    /// Insert a character at an index.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("Hello world!");
    /// s.insert(5, ',');
    /// assert_eq!(s, "Hello, world!");
    /// ```
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.insert_str(idx, ch.encode_utf8(&mut [0; 4]));
    }

    /// Reduces the length of the [`CompactString`] to zero.
    ///
    /// Calling this function does not change the capacity of the [`CompactString`].
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("Rust is the most loved language on Stackoverflow!");
    /// assert_eq!(s.capacity(), 49);
    ///
    /// s.clear();
    ///
    /// assert_eq!(s, "");
    /// assert_eq!(s.capacity(), 49);
    /// ```
    pub fn clear(&mut self) {
        unsafe { self.set_len(0) };
    }

    /// Split the [`CompactString`] into at the given byte index.
    ///
    /// Calling this function does not change the capacity of the [`CompactString`].
    ///
    /// # Panics
    ///
    /// Panics if `at` does not lie on a [`char`] boundary.
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("Hello, world!");
    /// assert_eq!(s.split_off(5), ", world!");
    /// assert_eq!(s, "Hello");
    /// ```
    pub fn split_off(&mut self, at: usize) -> Self {
        let result = self[at..].into();
        // SAFETY: the previous line `self[at...]` would have panicked if `at` was invalid
        unsafe { self.set_len(at) };
        result
    }

    /// Remove a range from the [`CompactString`], and return it as an iterator.
    ///
    /// Calling this function does not change the capacity of the [`CompactString`].
    ///
    /// # Panics
    ///
    /// Panics if the start or end of the range does not lie on a [`char`] boundary.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::new("Hello, world!");
    ///
    /// let mut d = s.drain(5..12);
    /// assert_eq!(d.next(), Some(','));   // iterate over the extracted data
    /// assert_eq!(d.as_str(), " world"); // or get the whole data as &str
    ///
    /// // The iterator keeps a reference to `s`, so you have to drop() the iterator,
    /// // before you can access `s` again.
    /// drop(d);
    /// assert_eq!(s, "Hello!");
    /// ```
    pub fn drain(&mut self, range: impl RangeBounds<usize>) -> Drain<'_> {
        let (start, end) = self.ensure_range(range);
        Drain {
            compact_string: self as *mut Self,
            start,
            end,
            chars: self[start..end].chars(),
        }
    }

    /// Shrinks the capacity of this [`CompactString`] with a lower bound.
    ///
    /// The resulting capactity is never less than the size of 3×[`usize`],
    /// i.e. the capacity than can be inlined.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::with_capacity(100);
    /// assert_eq!(s.capacity(), 100);
    ///
    /// // if the capacity was already bigger than the argument, the call is a no-op
    /// s.shrink_to(100);
    /// assert_eq!(s.capacity(), 100);
    ///
    /// s.shrink_to(50);
    /// assert_eq!(s.capacity(), 50);
    ///
    /// // if the string can be inlined, it is
    /// s.shrink_to(10);
    /// assert_eq!(s.capacity(), 3 * std::mem::size_of::<usize>());
    /// ```
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.repr.shrink_to(min_capacity);
    }

    /// Shrinks the capacity of this [`CompactString`] to match its length.
    ///
    /// The resulting capactity is never less than the size of 3×[`usize`],
    /// i.e. the capacity than can be inlined.
    ///
    /// This method is effectively the same as calling [`string.shrink_to(0)`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::from("This is a string with more than 24 characters.");
    ///
    /// s.reserve(100);
    /// assert!(s.capacity() >= 100);
    ///
    ///  s.shrink_to_fit();
    /// assert_eq!(s.len(), s.capacity());
    /// ```
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::from("short string");
    ///
    /// s.reserve(100);
    /// assert!(s.capacity() >= 100);
    ///
    /// s.shrink_to_fit();
    /// assert_eq!(s.capacity(), 3 * std::mem::size_of::<usize>());
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.repr.shrink_to(0);
    }

    /// Retains only the characters specified by the predicate.
    ///
    /// The method iterates over the characters in the string and calls the `predicate`.
    ///
    /// If the `predicate` returns `false`, then the character gets removed.
    /// If the `predicate` returns `true`, then the character is kept.
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::CompactString;
    /// let mut s = CompactString::from("äb𝄞d€");
    ///
    /// let keep = [false, true, true, false, true];
    /// let mut iter = keep.iter();
    /// s.retain(|_| *iter.next().unwrap());
    ///
    /// assert_eq!(s, "b𝄞€");
    /// ```
    pub fn retain(&mut self, mut predicate: impl FnMut(char) -> bool) {
        // We iterate over the string, and copy character by character.

        let s = self.as_mut_str();
        let mut dest_idx = 0;
        let mut src_idx = 0;
        while let Some(ch) = s[src_idx..].chars().next() {
            let ch_len = ch.len_utf8();
            if predicate(ch) {
                // SAFETY: We know that both indices are valid, and that we don't split a char.
                unsafe {
                    let p = s.as_mut_ptr();
                    core::ptr::copy(p.add(src_idx), p.add(dest_idx), ch_len);
                }
                dest_idx += ch_len;
            }
            src_idx += ch_len;
        }

        // SAFETY: We know that the index is a valid position to break the string.
        unsafe { self.set_len(dest_idx) };
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

impl DerefMut for CompactString {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut_str()
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

impl BorrowMut<str> for CompactString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut str {
        self.as_mut_str()
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

impl From<CompactString> for String {
    fn from(s: CompactString) -> Self {
        s.repr.into_string()
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

    fn write_fmt(mut self: &mut Self, args: fmt::Arguments<'_>) -> fmt::Result {
        match args.as_str() {
            Some(s) => {
                self.push_str(s);
                Ok(())
            }
            None => fmt::write(&mut self, args),
        }
    }
}

impl Add<&str> for CompactString {
    type Output = Self;
    fn add(mut self, rhs: &str) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl AddAssign<&str> for CompactString {
    fn add_assign(&mut self, rhs: &str) {
        self.push_str(rhs);
    }
}

/// A possible error value when converting a [`CompactString`] from a UTF-16 byte slice.
///
/// This type is the error type for the [`from_utf16`] method on [`CompactString`].
///
/// [`from_utf16`]: CompactString::from_utf16
/// # Examples
///
/// Basic usage:
///
/// ```
/// # use compact_str::CompactString;
/// // 𝄞mu<invalid>ic
/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
///           0xD800, 0x0069, 0x0063];
///
/// assert!(CompactString::from_utf16(v).is_err());
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Utf16Error(());

impl fmt::Display for Utf16Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("invalid utf-16: lone surrogate found", f)
    }
}

/// An iterator over the exacted data by [`CompactString::drain()`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Drain<'a> {
    compact_string: *mut CompactString,
    start: usize,
    end: usize,
    chars: std::str::Chars<'a>,
}

// SAFETY: Drain keeps the lifetime of the CompactString it belongs to.
unsafe impl Send for Drain<'_> {}
unsafe impl Sync for Drain<'_> {}

impl fmt::Debug for Drain<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Drain").field(&self.as_str()).finish()
    }
}

impl fmt::Display for Drain<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Drop for Drain<'_> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: Drain keeps a mutable reference to compact_string, so one one else can access
        //         the CompactString, but this function right now. CompactString::drain() ensured
        //         that the new extracted range does not split a UTF-8 character.
        unsafe { (*self.compact_string).replace_range_shrink(self.start, self.end, "") };
    }
}

impl Drain<'_> {
    /// The remaining, unconsumed characters of the extracted substring.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.chars.as_str()
    }
}

impl Deref for Drain<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Iterator for Drain<'_> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    #[inline]
    fn count(self) -> usize {
        // <Chars as Iterator>::count() is specialized, and cloning is trivial.
        self.chars.clone().count()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chars.size_hint()
    }

    #[inline]
    fn last(mut self) -> Option<char> {
        self.chars.next_back()
    }
}

impl DoubleEndedIterator for Drain<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<char> {
        self.chars.next_back()
    }
}

impl FusedIterator for Drain<'_> {}

crate::asserts::assert_size_eq!(CompactString, String);
