use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, RangeBounds};
use core::str::FromStr;
use core::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    str::Utf8Error,
};

use alloc::boxed::Box;
use alloc::fmt;
use alloc::{borrow::Cow, string::String};

use crate::Drain;
use crate::{repr::Repr, CompactString, ReserveError, UnwrapWithMsg, Utf16Error};

/// A [`CompactCowStr`] is a compact string type
/// that can be used as [`Cow<str>`] for [`CompactString`].
///
/// It can own a string as [`CompactString`] keeping the value on heap
/// or inline or static reference, or can borrow a str keeping the reference.  
#[repr(transparent)]
pub struct CompactCowStr<'a>(Repr, PhantomData<&'a ()>);

static_assertions::assert_eq_size!(CompactString, CompactCowStr);
static_assertions::assert_eq_align!(CompactString, CompactCowStr);

impl<'a> CompactCowStr<'a> {
    #[inline]
    const fn new_raw(repr: Repr) -> Self {
        CompactCowStr(repr, PhantomData)
    }

    /// Creates a new [`CompactCowStr`] from any type that implements `AsRef<str>`.
    /// If the string is short enough, then it will be inlined on the stack!
    /// Otherwise it will be stored as reference.
    ///
    /// In a `static` or `const` context you can use the method [`CompactCowStr::const_new()`].
    ///
    /// # Examples
    ///
    /// ### Inlined
    /// ```
    /// # use compact_str::CompactCowStr;
    /// // We can inline strings up to 12 characters long on 32-bit architectures...
    /// #[cfg(target_pointer_width = "32")]
    /// let s = "i'm 12 chars";
    /// // ...and up to 24 characters on 64-bit architectures!
    /// #[cfg(target_pointer_width = "64")]
    /// let s = "i am 24 characters long!";
    ///
    /// let compact = CompactCowStr::new(&s);
    ///
    /// assert_eq!(compact, s);
    /// // we are not allocated on the heap!
    /// assert!(!compact.is_heap_allocated());
    /// ```
    ///
    /// ### Reference
    /// ```
    /// # use compact_str::CompactCowStr;
    /// // For longer strings though, we preserve the reference.
    /// let long = "I am a longer string that will be preserved as a reference";
    /// let compact = CompactCowStr::new(long);
    ///
    /// assert_eq!(compact, long);
    /// // we keep the same reference!
    /// assert_eq!(compact.as_ptr(), long.as_ptr());
    /// ```
    #[inline]
    #[track_caller]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self::new_raw(Repr::new_ref(text.as_ref()))
    }

    /// Creates a new inline [`CompactCowStr`] from `&'static str` at compile time.
    /// Complexity: O(1). As an optimization, short strings get inlined.
    ///
    /// In a dynamic context you can use the method [`CompactCowStr::new()`].
    ///
    /// # Examples
    /// ```
    /// use compact_str::CompactCowStr;
    ///
    /// const DEFAULT_NAME: CompactCowStr = CompactCowStr::const_new("untitled");
    /// ```
    #[inline]
    pub const fn const_new(text: &'static str) -> Self {
        CompactCowStr::new_raw(Repr::const_new(text))
    }

    /// Get back the `&'a str` if it was stored as borrowed reference.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    /// // For longer strings though, we preserve the reference.
    /// let long = "I am a longer string that will be preserved as a reference";
    /// let compact = CompactCowStr::new(long);
    ///
    /// assert_eq!(compact, long);
    /// // we keep the same reference!
    /// assert_eq!(compact.as_ref_str().unwrap().as_ptr(), long.as_ptr());
    ///
    /// const DEFAULT_NAME: CompactCowStr =
    ///     CompactCowStr::const_new("That is not dead which can eternal lie.");
    /// assert_eq!(
    ///     DEFAULT_NAME.as_ref_str().unwrap(),
    ///     "That is not dead which can eternal lie.",
    /// );
    /// ```
    #[inline]
    #[rustversion::attr(since(1.64), const)]
    pub fn as_ref_str(&'a self) -> Option<&'a str> {
        self.0.as_ref_str()
    }

    /// Get back the `&'static str` constructed by [`CompactCowStr::const_new`].
    ///
    /// If the string was short enough that it could be inlined, then it was inline, and
    /// this method will return `None`.
    ///
    /// # Examples
    /// ```
    /// use compact_str::CompactCowStr;
    ///
    /// const DEFAULT_NAME: CompactCowStr =
    ///     CompactCowStr::const_new("That is not dead which can eternal lie.");
    /// assert_eq!(
    ///     DEFAULT_NAME.as_static_str().unwrap(),
    ///     "That is not dead which can eternal lie.",
    /// );
    /// ```
    #[inline]
    #[rustversion::attr(since(1.64), const)]
    pub fn as_static_str(&self) -> Option<&'static str> {
        self.0.as_static_str()
    }

    /// Creates a new empty [`CompactCowStr`] with the capacity to fit at least `capacity` bytes.
    /// This will
    ///
    /// This function behaves similarly to the [`CompactCowStr::with_capacity`] function.
    ///
    /// A `CompactCowStr` will inline strings on the stack, if they're small enough. Specifically,
    /// if the string has a length less than or equal to `std::mem::size_of::<String>` bytes
    /// then it will be inlined. This also means that `CompactCowStr`s have a minimum capacity
    /// of `std::mem::size_of::<String>`.
    ///
    /// # Panics
    ///
    /// This method panics if the system is out-of-memory.
    /// Use [`CompactCowStr::try_with_capacity()`] if you want to handle such a problem manually.
    ///
    /// # Examples
    ///
    /// ### "zero" Capacity
    /// ```
    /// # use compact_str::CompactCowStr;
    /// // Creating a CompactCowStr with a capacity of 0 will create
    /// // one with capacity of std::mem::size_of::<String>();
    /// let empty = CompactCowStr::with_capacity(0);
    /// let min_size = std::mem::size_of::<String>();
    ///
    /// assert_eq!(empty.capacity(), min_size);
    /// assert_ne!(0, min_size);
    /// assert!(!empty.is_heap_allocated());
    /// ```
    ///
    /// ### Max Inline Size
    /// ```
    /// # use compact_str::CompactCowStr;
    /// // Creating a CompactCowStr with a capacity of std::mem::size_of::<String>()
    /// // will not heap allocate.
    /// let str_size = std::mem::size_of::<String>();
    /// let empty = CompactCowStr::with_capacity(str_size);
    ///
    /// assert_eq!(empty.capacity(), str_size);
    /// assert!(!empty.is_heap_allocated());
    /// ```
    ///
    /// ### Heap Allocating
    /// ```
    /// # use compact_str::CompactCowStr;
    /// // If you create a `CompactCowStr` with a capacity greater than
    /// // `std::mem::size_of::<String>`, it will heap allocated. For heap
    /// // allocated strings we have a minimum capacity
    ///
    /// const MIN_HEAP_CAPACITY: usize = std::mem::size_of::<usize>() * 4;
    ///
    /// let heap_size = std::mem::size_of::<String>() + 1;
    /// let empty = CompactCowStr::with_capacity(heap_size);
    ///
    /// assert_eq!(empty.capacity(), MIN_HEAP_CAPACITY);
    /// assert!(empty.is_heap_allocated());
    /// ```
    #[inline]
    #[track_caller]
    pub fn with_capacity(capacity: usize) -> Self {
        CompactString::with_capacity(capacity).into()
    }

    /// Fallible version of [`CompactCowStr::with_capacity()`]
    ///
    /// This function behaves similarly to the [`CompactString::try_with_capacity`] function.    
    ///
    /// This method won't panic if the system is out-of-memory, but return an [`ReserveError`].
    /// Otherwise it behaves the same as [`CompactString::with_capacity()`].
    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<Self, ReserveError> {
        CompactString::try_with_capacity(capacity).map(Into::into)
    }

    /// Convert a slice of bytes into a [`CompactCowStr`].
    ///
    /// A [`CompactCowStr`] is a contiguous collection of bytes (`u8`s) that is valid [`UTF-8`](https://en.wikipedia.org/wiki/UTF-8).
    /// This method converts from an arbitrary contiguous collection of bytes into a
    /// [`CompactCowStr`], failing if the provided bytes are not `UTF-8`.
    ///
    /// # Examples
    /// ### Valid UTF-8
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let bytes = vec![240, 159, 166, 128, 240, 159, 146, 175];
    /// let compact = CompactCowStr::from_utf8(bytes).expect("valid UTF-8");
    ///
    /// assert_eq!(compact, "🦀💯");
    /// ```
    ///
    /// ### Invalid UTF-8
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let bytes = vec![255, 255, 255];
    /// let result = CompactCowStr::from_utf8(bytes);
    ///
    /// assert!(result.is_err());
    /// ```
    #[inline]
    pub fn from_utf8<B: AsRef<[u8]>>(buf: B) -> Result<Self, Utf8Error> {
        Repr::from_utf8_ref(buf).map(CompactCowStr::new_raw)
    }

    /// Converts a vector of bytes to a [`CompactString`] without checking that the string contains
    /// valid UTF-8.
    ///
    /// See the safe version, [`CompactCowStr::from_utf8`], for more details.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed to it are valid
    /// UTF-8. If this constraint is violated, it may cause memory unsafety issues with future users
    /// of the [`CompactCowStr`], as the rest of the standard library assumes that
    /// [`CompactCowStr`]s are valid UTF-8.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// // some bytes, in a vector
    /// let sparkle_heart = vec![240, 159, 146, 150];
    ///
    /// let sparkle_heart = unsafe {
    ///     CompactCowStr::from_utf8_unchecked(sparkle_heart)
    /// };
    ///
    /// assert_eq!("💖", sparkle_heart);
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn from_utf8_unchecked<B: AsRef<[u8]>>(buf: B) -> Self {
        Repr::from_utf8_unchecked_ref(buf)
            .map(CompactCowStr::new_raw)
            .unwrap_with_msg()
    }

    /// Decode a [`UTF-16`](https://en.wikipedia.org/wiki/UTF-16) slice of bytes into a
    /// [`CompactCowStr`], returning an [`Err`] if the slice contains any invalid data.
    ///
    ///
    /// # Examples
    /// ### Valid UTF-16
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let buf: &[u16] = &[0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let compact = CompactCowStr::from_utf16(buf).unwrap();
    ///
    /// assert_eq!(compact, "𝄞music");
    /// ```
    ///
    /// ### Invalid UTF-16
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let buf: &[u16] = &[0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// let res = CompactCowStr::from_utf16(buf);
    ///
    /// assert!(res.is_err());
    /// ```
    #[inline]
    pub fn from_utf16<B: AsRef<[u16]>>(buf: B) -> Result<Self, Utf16Error> {
        CompactString::from_utf16(buf).map(Into::into)
    }

    /// Decode a UTF-16–encoded slice `v` into a `CompactString`, replacing invalid data with
    /// the replacement character (`U+FFFD`), �.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// // 𝄞mus<invalid>ic<invalid>
    /// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
    ///           0x0073, 0xDD1E, 0x0069, 0x0063,
    ///           0xD834];
    ///
    /// assert_eq!(CompactCowStr::from("𝄞mus\u{FFFD}ic\u{FFFD}"),
    ///            CompactCowStr::from_utf16_lossy(v));
    /// ```
    #[inline]
    pub fn from_utf16_lossy<B: AsRef<[u16]>>(buf: B) -> Self {
        CompactString::from_utf16_lossy(buf).into()
    }

    /// Returns the length of the [`CompactCowStr`] in `bytes`, not [`char`]s or graphemes.
    ///
    /// When using `UTF-8` encoding (which all strings in Rust do) a single character will be 1 to 4
    /// bytes long, therefore the return value of this method might not be what a human considers
    /// the length of the string.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let ascii = CompactCowStr::new("hello world");
    /// assert_eq!(ascii.len(), 11);
    ///
    /// let emoji = CompactCowStr::new("👱");
    /// assert_eq!(emoji.len(), 4);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the [`CompactString`] has a length of 0, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut msg = CompactCowStr::new("");
    /// assert!(msg.is_empty());
    ///
    /// // add some characters
    /// msg.push_str("hello reader!");
    /// assert!(!msg.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the capacity of the [`CompactCowStr`], in bytes.
    ///
    /// # Note
    /// * A `CompactCowStr` will always have a capacity of at least `std::mem::size_of::<String>()`
    ///
    /// # Examples
    /// ### Minimum Size
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let min_size = std::mem::size_of::<String>();
    /// let compact = CompactCowStr::new("");
    ///
    /// assert!(compact.capacity() >= min_size);
    /// ```
    ///
    /// ### Heap Allocated
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let compact = CompactCowStr::with_capacity(128);
    /// assert_eq!(compact.capacity(), 128);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Ensures that this [`CompactCowStr`]'s capacity is at least `additional` bytes longer than
    /// its length. The capacity may be increased by more than `additional` bytes if it chooses,
    /// to prevent frequent reallocations.
    ///
    /// # Note
    /// * A `CompactCowStr` will always have at least a capacity of `std::mem::size_of::<String>()`
    /// * Reserving additional bytes may cause the `CompactCowStr` to become heap allocated
    ///
    /// # Panics
    /// This method panics if the new capacity overflows `usize` or if the system is out-of-memory.
    /// Use [`CompactCowStr::try_reserve()`] if you want to handle such a problem manually.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    ///
    /// const WORD: usize = std::mem::size_of::<usize>();
    /// let mut compact = CompactCowStr::default();
    /// assert!(compact.capacity() >= (WORD * 3) - 1);
    ///
    /// compact.reserve(200);
    /// assert!(compact.is_heap_allocated());
    /// assert!(compact.capacity() >= 200);
    /// ```
    #[inline]
    #[track_caller]
    pub fn reserve(&mut self, additional: usize) {
        self.try_reserve(additional).unwrap_with_msg()
    }

    /// Fallible version of [`CompactCowStr::reserve()`]
    ///
    /// This method won't panic if the system is out-of-memory, but return an [`ReserveError`]
    /// Otherwise it behaves the same as [`CompactCowStr::reserve()`].
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), ReserveError> {
        self.to_mut().try_reserve(additional)
    }

    /// Returns a string slice containing the entire [`CompactCowStr`].
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let s = CompactCowStr::new("hello");
    ///
    /// assert_eq!(s.as_str(), "hello");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns a mutable string slice containing the entire [`CompactCowStr`].
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("hello");
    /// s.as_mut_str().make_ascii_uppercase();
    ///
    /// assert_eq!(s.as_str(), "HELLO");
    /// ```
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        self.to_mut().as_mut_str()
    }

    /// Returns a byte slice of the [`CompactCowStr`]'s contents.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let s = CompactCowStr::new("hello");
    ///
    /// assert_eq!(&[104, 101, 108, 108, 111], s.as_bytes());
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.to_ref().as_bytes()
    }

    // TODO: Implement a `try_as_mut_slice(...)` that will fail if it results in cloning?
    //
    /// Provides a mutable reference to the underlying buffer of bytes.
    ///
    /// # Safety
    /// * All Rust strings, including `CompactCowStr`, must be valid UTF-8. The caller must
    ///   guarantee
    /// that any modifications made to the underlying buffer are valid UTF-8.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("hello");
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
        self.to_mut().as_mut_bytes()
    }

    /// Appends the given [`char`] to the end of this [`CompactCowStr`].
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("foo");
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

    /// Removes the last character from the [`CompactCowStr`] and returns it.
    /// Returns `None` if this [`CompactCowStr`] is empty.
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("abc");
    ///
    /// assert_eq!(s.pop(), Some('c'));
    /// assert_eq!(s.pop(), Some('b'));
    /// assert_eq!(s.pop(), Some('a'));
    ///
    /// assert_eq!(s.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        self.to_mut().pop()
    }

    /// Appends a given string slice onto the end of this [`CompactCowStr`]
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("abc");
    ///
    /// s.push_str("123");
    ///
    /// assert_eq!("abc123", s);
    /// ```
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.to_mut().push_str(s)
    }

    /// Removes a [`char`] from this [`CompactCowStr`] at a byte position and returns it.
    ///
    /// This is an *O*(*n*) operation, as it requires copying every element in the
    /// buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than or equal to the [`CompactCowStr`]'s length,
    /// or if it does not lie on a [`char`] boundary.
    ///
    /// # Examples
    ///
    /// ### Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut c = CompactCowStr::from("hello world");
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
    /// # use compact_str::CompactCowStr;
    /// let mut c = CompactCowStr::from("hello there!");
    /// c.remove(100);
    /// ```
    ///
    /// ### Not on char boundary:
    ///
    /// ```should_panic
    /// # use compact_str::CompactCowStr;
    /// let mut c = CompactCowStr::from("🦄");
    /// c.remove(1);
    /// ```
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        self.to_mut().remove(idx)
    }

    /// Forces the length of the [`CompactCowStr`] to `new_len`.
    ///
    /// This is a low-level operation that maintains none of the normal invariants for
    /// `CompactCowStr`. If you want to modify the `CompactCowStr` you should use methods like
    /// `push`, `push_str` or `pop`.
    ///
    /// This doesn't clone and mark as owned.
    ///
    /// # Safety
    /// * `new_len` must be less than or equal to `capacity()`
    /// * The elements at `old_len..new_len` must be initialized
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.0.set_len(new_len)
    }

    /// Returns whether or not the [`CompactCowStr`] is heap allocated.
    ///
    /// # Examples
    /// ### Inlined
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let hello = CompactCowStr::new("hello world");
    ///
    /// assert!(!hello.is_heap_allocated());
    /// ```
    ///
    /// ### Reference
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let msg = CompactCowStr::new("this message will self destruct in 5, 4, 3, 2, 1 💥");
    ///
    /// assert!(!msg.is_heap_allocated());
    /// ```
    ///
    /// ### Heap Allocated
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut msg = CompactCowStr::new("this message will self destruct in 5, 4, 3, 2, 1 💥");
    /// msg.replace_range(0..1, "T");
    /// assert!(msg.is_heap_allocated());
    /// ```
    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        self.0.is_heap_allocated()
    }

    /// Returns whether or not the [`CompactCowStr`] is borrowed.
    /// This means that resource is not owned, and mutating this will cause clone.
    ///
    /// # Examples
    /// ### Inlined
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let hello = CompactCowStr::new("hello world");
    ///
    /// assert!(!hello.is_borrowed());
    /// ```
    ///
    /// ### Static
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let msg = CompactCowStr::const_new("this message will self destruct in 5, 4, 3, 2, 1 💥");
    ///
    /// assert!(msg.is_borrowed());
    /// ```
    ///
    /// ### Reference
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let msg = CompactCowStr::new("this message will self destruct in 5, 4, 3, 2, 1 💥");
    ///
    /// assert!(msg.is_borrowed());
    /// ```
    ///
    /// ### Heap Allocated
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut msg = CompactCowStr::new("this message will self destruct in 5, 4, 3, 2, 1 💥");
    /// msg.to_mut();
    /// assert!(!msg.is_borrowed());
    /// ```
    #[inline]
    pub fn is_borrowed(&self) -> bool {
        self.0.is_ref_str()
    }

    /// Removes the specified range in the [`CompactCowStr`],
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
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("Hello, world!");
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
        self.to_mut().replace_range(range, replace_with)
    }

    /// Creates a new [`CompactCowStr`] by repeating a string `n` times.
    ///
    /// # Panics
    ///
    /// This function will panic if the capacity would overflow.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use compact_str::CompactCowStr;
    /// assert_eq!(CompactCowStr::new("abc").repeat(4), CompactCowStr::new("abcabcabcabc"));
    /// ```
    ///
    /// A panic upon overflow:
    ///
    /// ```should_panic
    /// use compact_str::CompactCowStr;
    ///
    /// // this will panic at runtime
    /// let huge = CompactCowStr::new("0123456789abcdef").repeat(usize::MAX);
    /// ```
    #[must_use]
    pub fn repeat(&self, n: usize) -> Self {
        self.to_ref().repeat(n).into()
    }

    /// Truncate the [`CompactCowStr`] to a shorter length.
    ///
    /// If the length of the [`CompactCowStr`] is less or equal to `new_len`, the call is a no-op.
    ///
    /// Calling this function does not change the capacity of the [`CompactCowStr`].
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
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("Hello, world!");
    /// s.truncate(5);
    /// assert_eq!(s, "Hello");
    /// ```
    pub fn truncate(&mut self, new_len: usize) {
        self.to_mut().truncate(new_len)
    }

    /// Converts a [`CompactCowStr`] to a raw pointer.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.to_ref().as_ptr()
    }

    /// Converts a mutable [`CompactCowStr`] to a raw pointer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.to_mut().as_mut_ptr()
    }

    /// Insert string character at an index.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("Hello!");
    /// s.insert_str(5, ", world");
    /// assert_eq!(s, "Hello, world!");
    /// ```
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        self.to_mut().insert_str(idx, string)
    }

    /// Insert a character at an index.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("Hello world!");
    /// s.insert(5, ',');
    /// assert_eq!(s, "Hello, world!");
    /// ```
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.insert_str(idx, ch.encode_utf8(&mut [0; 4]));
    }

    /// Reduces the length of the [`CompactCowStr`] to zero.
    ///
    /// Calling this function does not change the capacity of the [`CompactCowStr`].
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("Rust is the most loved language on Stackoverflow!");
    /// assert_eq!(s.capacity(), 49);
    ///
    /// s.clear();
    ///
    /// assert_eq!(s, "");
    /// assert_eq!(s.capacity(), 49);
    /// ```
    pub fn clear(&mut self) {
        self.to_mut().clear()
    }

    /// Split the [`CompactCowStr`] into at the given byte index.
    ///
    /// Calling this function does not change the capacity of the [`CompactCowStr`], unless the
    /// [`CompactCowStr`] is backed by a `&str`.
    ///
    /// # Panics
    ///
    /// Panics if `at` does not lie on a [`char`] boundary.
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::const_new("Hello, world!");
    /// let w = s.split_off(5);
    ///
    /// assert_eq!(w, ", world!");
    /// assert_eq!(s, "Hello");
    /// ```
    pub fn split_off(&mut self, at: usize) -> Self {
        if let Some(s) = self.as_static_str() {
            let result = Self::const_new(&s[at..]);
            // SAFETY: the previous line `self[at...]` would have panicked if `at` was invalid
            unsafe { self.set_len(at) };
            result
        } else {
            // This will make result as borrowed str.
            let result = self[at..].into();
            // SAFETY: the previous line `self[at...]` would have panicked if `at` was invalid
            unsafe { self.set_len(at) };
            result
        }
    }

    /// Remove a range from the [`CompactCowStr`], and return it as an iterator.
    ///
    /// Calling this function does not change the capacity of the [`CompactCowStr`].
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
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::new("Hello, world!");
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
        self.to_mut().drain(range)
    }

    /// Shrinks the capacity of this [`CompactCowStr`] with a lower bound.
    ///
    /// The resulting capactity is never less than the size of 3×[`usize`],
    /// i.e. the capacity than can be inlined.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::with_capacity(100);
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
        if self.is_heap_allocated() {
            self.to_mut().shrink_to(min_capacity)
        }
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
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::from("This is a string with more than 24 characters.");
    ///
    /// s.reserve(100);
    /// assert!(s.capacity() >= 100);
    ///
    ///  s.shrink_to_fit();
    /// assert_eq!(s.len(), s.capacity());
    /// ```
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::from("short string");
    ///
    /// s.reserve(100);
    /// assert!(s.capacity() >= 100);
    ///
    /// s.shrink_to_fit();
    /// assert_eq!(s.capacity(), 3 * std::mem::size_of::<usize>());
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.shrink_to(0)
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
    /// # use compact_str::CompactCowStr;
    /// let mut s = CompactCowStr::from("äb𝄞d€");
    ///
    /// let keep = [false, true, true, false, true];
    /// let mut iter = keep.iter();
    /// s.retain(|_| *iter.next().unwrap());
    ///
    /// assert_eq!(s, "b𝄞€");
    /// ```
    pub fn retain(&mut self, predicate: impl FnMut(char) -> bool) {
        self.to_mut().retain(predicate)
    }

    /// Decode a bytes slice as UTF-8 string, replacing any illegal codepoints
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let chess_knight = b"\xf0\x9f\xa8\x84";
    ///
    /// assert_eq!(
    ///     "🨄",
    ///     CompactCowStr::from_utf8_lossy(chess_knight),
    /// );
    ///
    /// // For valid UTF-8 slices, this is the same as:
    /// assert_eq!(
    ///     "🨄",
    ///     CompactCowStr::new(std::str::from_utf8(chess_knight).unwrap()),
    /// );
    /// ```
    ///
    /// Incorrect bytes:
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let broken = b"\xf0\x9f\xc8\x84";
    ///
    /// assert_eq!(
    ///     "�Ȅ",
    ///     CompactCowStr::from_utf8_lossy(broken),
    /// );
    ///
    /// // For invalid UTF-8 slices, this is an optimized implemented for:
    /// assert_eq!(
    ///     "�Ȅ",
    ///     CompactCowStr::from(String::from_utf8_lossy(broken)),
    /// );
    /// ```
    pub fn from_utf8_lossy(v: &[u8]) -> Self {
        // fixme: optimize
        String::from_utf8_lossy(v).into()
    }

    /// Convert the [`CompactCowStr`] into a [`String`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let s = CompactCowStr::new("Hello world");
    /// let s = s.into_string();
    /// assert_eq!(s, "Hello world");
    /// ```
    pub fn into_string(self) -> String {
        self.0.into_string()
    }

    /// Convert a [`String`] into a [`CompactCowStr`] _without inlining_.
    ///
    /// Note: You probably don't need to use this method, instead you should use `From<String>`
    /// which is implemented for [`CompactCowStr`].
    ///
    /// This method exists incase your code is very sensitive to memory allocations. Normally when
    /// converting a [`String`] to a [`CompactCowStr`] we'll inline short strings onto the stack.
    /// But this results in [`Drop`]-ing the original [`String`], which causes memory it owned on
    /// the heap to be deallocated. Instead when using this method, we always reuse the buffer that
    /// was previously owned by the [`String`], so no trips to the allocator are needed.
    ///
    /// # Examples
    ///
    /// ### Short Strings
    /// ```
    /// use compact_str::CompactCowStr;
    ///
    /// let short = "hello world".to_string();
    /// let c_heap = CompactCowStr::from_string_buffer(short);
    ///
    /// // using CompactCowStr::from_string_buffer, we'll re-use the String's underlying buffer
    /// assert!(c_heap.is_heap_allocated());
    ///
    /// // note: when Clone-ing a short heap allocated string, we'll eagerly inline at that point
    /// let c_inline = c_heap.clone();
    /// assert!(!c_inline.is_heap_allocated());
    ///
    /// assert_eq!(c_heap, c_inline);
    /// ```
    ///
    /// ### Longer Strings
    /// ```
    /// use compact_str::CompactCowStr;
    ///
    /// let x = "longer string that will be on the heap".to_string();
    /// let c1 = CompactCowStr::from(x);
    ///
    /// let y = "longer string that will be on the heap".to_string();
    /// let c2 = CompactCowStr::from_string_buffer(y);
    ///
    /// // for longer strings, we re-use the underlying String's buffer in both cases
    /// assert!(c1.is_heap_allocated());
    /// assert!(c2.is_heap_allocated());
    /// ```
    ///
    /// ### Buffer Re-use
    /// ```
    /// use compact_str::CompactCowStr;
    ///
    /// let og = "hello world".to_string();
    /// let og_addr = og.as_ptr();
    ///
    /// let mut c = CompactCowStr::from_string_buffer(og);
    /// let ex_addr = c.as_ptr();
    ///
    /// // When converting to/from String and CompactCowStr with from_string_buffer we always re-use
    /// // the same underlying allocated memory/buffer
    /// assert_eq!(og_addr, ex_addr);
    ///
    /// let long = "this is a long string that will be on the heap".to_string();
    /// let long_addr = long.as_ptr();
    ///
    /// let mut long_c = CompactCowStr::from(long);
    /// let long_ex_addr = long_c.as_ptr();
    ///
    /// // When converting to/from String and CompactCowStr with From<String>, we'll also re-use the
    /// // underlying buffer, if the string is long, otherwise when converting to CompactString we
    /// // eagerly inline
    /// assert_eq!(long_addr, long_ex_addr);
    /// ```
    #[inline]
    #[track_caller]
    pub fn from_string_buffer(s: String) -> Self {
        CompactString::from_string_buffer(s).into()
    }

    #[inline]
    fn into_compact_string(mut self) -> CompactString {
        self.0.make_owned();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    fn to_ref(&self) -> &CompactString {
        unsafe { std::mem::transmute(self) }
    }

    /// Acquires a mutable reference to the owned form of the data.
    /// Clones the data if it is not already owned.
    ///
    /// ```
    /// # use compact_str::CompactCowStr;
    /// let original = "This is a string with more than 24 characters.";
    /// let mut cow_str = CompactCowStr::new(original);
    /// assert_eq!(cow_str.as_ptr(), original.as_ptr());
    /// assert!(cow_str.is_borrowed());
    /// assert!(!cow_str.is_heap_allocated());
    /// cow_str.to_mut();
    /// assert_ne!(cow_str.as_ptr(), original.as_ptr());
    /// assert!(!cow_str.is_borrowed());
    /// assert!(cow_str.is_heap_allocated());
    /// ```
    ///
    #[inline]
    pub fn to_mut(&mut self) -> &mut CompactString {
        self.0.make_owned();
        unsafe { std::mem::transmute(self) }
    }
}

impl<'a> From<CompactString> for CompactCowStr<'a> {
    #[inline]
    fn from(value: CompactString) -> Self {
        // SAFETY:
        // * A `HeapBuffer` and `Repr` have the same size
        // * and all LastUtf8Char is valid for `CompactCowStr`
        unsafe { std::mem::transmute(value) }
    }
}

impl<'a> From<&'a CompactString> for CompactCowStr<'a> {
    #[inline]
    fn from(value: &'a CompactString) -> Self {
        if value.is_heap_allocated() {
            // Create a new cow str as borrowed from source value.
            Self::new(value.as_str())
        } else {
            // If the original CompactString is not heap allocated,
            // we need to preserve whether this repr is stacic or non-static refernce,
            // or is on the stack, so clone the inner repr.
            unsafe { CompactCowStr::new_raw(core::ptr::read(&value.0)) }
        }
    }
}

impl<'a, 'b> From<&'a CompactCowStr<'b>> for CompactCowStr<'a> {
    #[inline]
    fn from(value: &'a CompactCowStr<'b>) -> Self {
        if value.is_heap_allocated() {
            // Create a new cow str as borrowed from source value.
            Self::new(value.as_str())
        } else {
            // If the original CompactString is not heap allocated,
            // we need to preserve whether this repr is stacic or non-static refernce,
            // or is on the stack, so clone the inner repr.
            unsafe { CompactCowStr::new_raw(core::ptr::read(&value.0)) }
        }
    }
}

impl<'a> From<CompactCowStr<'a>> for CompactString {
    #[inline]
    fn from(value: CompactCowStr<'a>) -> Self {
        value.into_compact_string()
    }
}

impl Clone for CompactCowStr<'_> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new_raw(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0)
    }
}

impl Default for CompactCowStr<'_> {
    #[inline]
    fn default() -> Self {
        CompactCowStr::new("")
    }
}

impl Deref for CompactCowStr<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl DerefMut for CompactCowStr<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl AsRef<str> for CompactCowStr<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for CompactCowStr<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> Borrow<str> for CompactCowStr<'a> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'a> BorrowMut<str> for CompactCowStr<'a> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl Eq for CompactCowStr<'_> {}

impl<T: AsRef<str>> PartialEq<T> for CompactCowStr<'_> {
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<CompactCowStr<'_>> for String {
    fn eq(&self, other: &CompactCowStr<'_>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<CompactCowStr<'_>> for &str {
    fn eq(&self, other: &CompactCowStr<'_>) -> bool {
        *self == other.as_str()
    }
}

impl<'a> PartialEq<CompactCowStr<'_>> for Cow<'a, str> {
    fn eq(&self, other: &CompactCowStr<'_>) -> bool {
        *self == other.as_str()
    }
}

impl Ord for CompactCowStr<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for CompactCowStr<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for CompactCowStr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<'a> From<&'a str> for CompactCowStr<'_> {
    #[inline]
    #[track_caller]
    fn from(s: &'a str) -> Self {
        CompactCowStr::new(s)
    }
}

impl From<String> for CompactCowStr<'_> {
    #[inline]
    #[track_caller]
    fn from(s: String) -> Self {
        CompactString::from(s).into()
    }
}

impl<'a> From<&'a String> for CompactCowStr<'_> {
    #[inline]
    #[track_caller]
    fn from(s: &'a String) -> Self {
        CompactCowStr::new(s)
    }
}

impl<'a> From<Cow<'a, str>> for CompactCowStr<'_> {
    fn from(cow: Cow<'a, str>) -> Self {
        match cow {
            Cow::Borrowed(s) => s.into(),
            // we separate these two so we can re-use the underlying buffer in the owned case
            Cow::Owned(s) => s.into(),
        }
    }
}

impl From<Box<str>> for CompactCowStr<'_> {
    #[inline]
    #[track_caller]
    fn from(b: Box<str>) -> Self {
        CompactString::from(b).into()
    }
}

impl From<CompactCowStr<'_>> for String {
    #[inline]
    fn from(s: CompactCowStr<'_>) -> Self {
        s.into_string()
    }
}

impl<'a> From<CompactCowStr<'a>> for Cow<'a, str> {
    #[inline]
    fn from(s: CompactCowStr<'a>) -> Self {
        s.0.into_cow()
    }
}

impl<'a> From<&'a CompactCowStr<'_>> for Cow<'a, str> {
    #[inline]
    fn from(s: &'a CompactCowStr<'_>) -> Self {
        Self::Borrowed(s.as_str())
    }
}

#[rustversion::since(1.60)]
#[cfg(target_has_atomic = "ptr")]
impl From<CompactCowStr<'_>> for alloc::sync::Arc<str> {
    fn from(value: CompactCowStr<'_>) -> Self {
        Self::from(value.as_str())
    }
}

impl From<CompactCowStr<'_>> for alloc::rc::Rc<str> {
    fn from(value: CompactCowStr<'_>) -> Self {
        Self::from(value.as_str())
    }
}

#[cfg(feature = "std")]
impl From<CompactCowStr<'_>> for Box<dyn std::error::Error + Send + Sync> {
    fn from(value: CompactCowStr<'_>) -> Self {
        CompactString::from(value).into()
    }
}

#[cfg(feature = "std")]
impl From<CompactCowStr<'_>> for Box<dyn std::error::Error> {
    fn from(value: CompactCowStr<'_>) -> Self {
        CompactString::from(value).into()
    }
}

impl From<CompactCowStr<'_>> for Box<str> {
    fn from(value: CompactCowStr<'_>) -> Self {
        if value.is_heap_allocated() {
            value.into_string().into_boxed_str()
        } else {
            Box::from(value.as_str())
        }
    }
}

#[cfg(feature = "std")]
impl From<CompactCowStr<'_>> for std::ffi::OsString {
    fn from(value: CompactCowStr<'_>) -> Self {
        Self::from(value.into_string())
    }
}

#[cfg(feature = "std")]
impl From<CompactCowStr<'_>> for std::path::PathBuf {
    fn from(value: CompactCowStr<'_>) -> Self {
        Self::from(std::ffi::OsString::from(value))
    }
}

#[cfg(feature = "std")]
impl AsRef<std::path::Path> for CompactCowStr<'_> {
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(self.as_str())
    }
}

impl From<CompactCowStr<'_>> for alloc::vec::Vec<u8> {
    fn from(value: CompactCowStr<'_>) -> Self {
        if value.is_heap_allocated() {
            value.into_string().into_bytes()
        } else {
            value.as_bytes().to_vec()
        }
    }
}

impl<'a> FromStr for CompactCowStr<'a> {
    type Err = core::convert::Infallible;
    fn from_str(s: &str) -> Result<CompactCowStr<'a>, Self::Err> {
        Ok(CompactCowStr::from(s))
    }
}

impl fmt::Debug for CompactCowStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for CompactCowStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<S> FromIterator<S> for CompactCowStr<'_>
where
    CompactString: FromIterator<S>,
{
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        CompactString::from_iter(iter).into()
    }
}

impl<'a> FromIterator<CompactCowStr<'a>> for CompactString {
    fn from_iter<T: IntoIterator<Item = CompactCowStr<'a>>>(iter: T) -> Self {
        let repr = iter.into_iter().collect();
        CompactString(repr)
    }
}

impl<'a> FromIterator<CompactCowStr<'a>> for String {
    fn from_iter<T: IntoIterator<Item = CompactCowStr<'a>>>(iter: T) -> Self {
        let mut iterator = iter.into_iter();
        match iterator.next() {
            None => String::new(),
            Some(buf) => {
                let mut buf = buf.into_string();
                buf.extend(iterator);
                buf
            }
        }
    }
}

impl<'a> Extend<CompactCowStr<'a>> for String {
    fn extend<T: IntoIterator<Item = CompactCowStr<'a>>>(&mut self, iter: T) {
        for s in iter {
            self.push_str(&s);
        }
    }
}

impl<'a> Extend<CompactCowStr<'a>> for Cow<'_, str> {
    fn extend<T: IntoIterator<Item = CompactCowStr<'a>>>(&mut self, iter: T) {
        self.to_mut().extend(iter);
    }
}

impl<'a, S> Extend<S> for CompactCowStr<'a>
where
    CompactString: Extend<S>,
{
    fn extend<T: IntoIterator<Item = S>>(&mut self, iter: T) {
        self.to_mut().extend(iter);
    }
}

impl core::fmt::Write for CompactCowStr<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.to_mut().write_str(s)
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        self.to_mut().write_fmt(args)
    }
}

impl Add<&str> for CompactCowStr<'_> {
    type Output = Self;
    fn add(mut self, rhs: &str) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl AddAssign<&str> for CompactCowStr<'_> {
    fn add_assign(&mut self, rhs: &str) {
        self.push_str(rhs);
    }
}
