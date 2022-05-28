// TODO: Use this module
#![allow(dead_code)]

use std::iter::Extend;
use std::sync::atomic::Ordering;
use std::{
    fmt,
    mem,
    slice,
    str,
};

mod inner;
use inner::{
    ArcStringHeader,
    ArcStringPtr,
};
mod writer;
use writer::ArcStringWriter;

/// A soft limit on the amount of references that may be made to an `Arc`.
///
/// Going above this limit will abort your program (although not
/// necessarily) at _exactly_ `MAX_REFCOUNT + 1` references.
const MAX_REFCOUNT: usize = (isize::MAX) as usize;

#[repr(C)]
pub struct ArcString {
    /// The string buffer behind the pointer must be initialized for at least self.len bytes
    /// and contain valid UTF-8.
    ptr: ArcStringPtr,
    len: usize,
}

// SAFETY: Mutation only happens through `&mut self`
unsafe impl Sync for ArcString {}
unsafe impl Send for ArcString {}

impl ArcString {
    #[inline]
    pub fn new(text: &str, additional: usize) -> Self {
        let len = text.len();

        let required = len + additional;
        let amortized = 3 * len / 2;
        let new_capacity = core::cmp::max(amortized, required);

        // TODO: Handle overflows in the case of __very__ large Strings
        debug_assert!(new_capacity >= len);

        let mut ptr = ArcStringPtr::with_capacity(new_capacity);

        let buffer_ptr = ptr.str_buf_ptr_mut();
        // SAFETY: We know both `src` and `dest` are valid for respectively reads and writes of
        // length `len` because `len` comes from `src`, and `dest` was allocated to be at least that
        // length. We also know they're non-overlapping because `dest` is newly allocated
        unsafe { buffer_ptr.copy_from_nonoverlapping(text.as_ptr(), len) };

        ArcString { len, ptr }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        // We should never be able to programatically create an `ArcString` with a capacity less
        // than our max inline size, since then the string should be inlined
        debug_assert!(capacity >= super::MAX_SIZE);

        let len = 0;
        let ptr = ArcStringPtr::with_capacity(capacity);

        ArcString { len, ptr }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.header().capacity
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        self.writer().push(ch);
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: The only way you can construct an `ArcString` is via a `&str` so it must be valid
        // UTF-8, or the caller has manually made those guarantees
        unsafe { str::from_utf8_unchecked(self.as_slice()) }
    }

    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        self.len = length;
    }

    #[inline]
    fn writer(&mut self) -> ArcStringWriter<'_> {
        ArcStringWriter::new(self)
    }

    /// Returns a shared reference to the heap allocated `ArcStringInner`
    #[inline]
    fn header(&self) -> &ArcStringHeader {
        self.ptr.header()
    }

    #[inline(never)]
    unsafe fn drop_inner(&mut self) {
        ArcStringPtr::dealloc(&mut self.ptr)
    }

    /// Returns a mutable reference to the initialized parts of the underlying buffer of bytes
    ///
    /// # Invariants
    /// * The caller must assert that the underlying buffer is still valid UTF-8
    #[inline]
    pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        // SAFETY:
        // We track the length in self.len and assert that it is valid.
        //
        // Note: In terms of mutability, it's up to the caller to assert the provided bytes are
        // value UTF-8
        slice::from_raw_parts_mut(self.ptr.str_buf_ptr_mut(), self.len)
    }

    /// Returns a shared reference to the underlying buffer of bytes
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY:
        // We track the length in self.len and assert that it is valid.
        //
        // Note: In terms of mutability, it's up to the caller to assert the provided bytes are
        // value UTF-8
        unsafe { slice::from_raw_parts(self.ptr.str_buf_ptr(), self.len) }
    }
}

impl Clone for ArcString {
    fn clone(&self) -> Self {
        let old_count = self.header().ref_count.fetch_add(1, Ordering::Relaxed);
        assert!(
            old_count < MAX_REFCOUNT,
            "Program has gone wild, ref count > {}",
            MAX_REFCOUNT
        );

        ArcString {
            len: self.len,
            ptr: self.ptr,
        }
    }
}

impl Drop for ArcString {
    fn drop(&mut self) {
        // This was copied from the implementation of `std::sync::Arc`
        // TODO: Better document the safety invariants here
        if self.header().ref_count.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }
        std::sync::atomic::fence(Ordering::Acquire);
        unsafe { self.drop_inner() }
    }
}

impl fmt::Debug for ArcString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl From<&str> for ArcString {
    #[inline]
    fn from(text: &str) -> Self {
        ArcString::new(text, 0)
    }
}

impl Extend<char> for ArcString {
    #[inline]
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        self.writer().extend(iter);
    }
}

impl<'c> Extend<&'c char> for ArcString {
    #[inline]
    fn extend<T: IntoIterator<Item = &'c char>>(&mut self, iter: T) {
        self.writer().extend(iter);
    }
}

impl<'s> Extend<&'s str> for ArcString {
    #[inline]
    fn extend<T: IntoIterator<Item = &'s str>>(&mut self, iter: T) {
        self.writer().extend(iter);
    }
}

impl Extend<Box<str>> for ArcString {
    #[inline]
    fn extend<T: IntoIterator<Item = Box<str>>>(&mut self, iter: T) {
        self.writer().extend(iter);
    }
}

impl Extend<String> for ArcString {
    #[inline]
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.writer().extend(iter);
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::ArcString;
    use crate::tests::rand_unicode;

    #[test]
    fn test_empty() {
        let empty = "";
        let arc_str = ArcString::from(empty);

        assert_eq!(arc_str.as_str(), empty);
        assert_eq!(arc_str.len, empty.len());
    }

    #[test]
    fn test_long() {
        let long = "aaabbbcccdddeeefff\n
                    ggghhhiiijjjkkklll\n
                    mmmnnnooopppqqqrrr\n
                    ssstttuuuvvvwwwxxx\n
                    yyyzzz000111222333\n
                    444555666777888999000";
        let arc_str = ArcString::from(long);

        assert_eq!(arc_str.as_str(), long);
        assert_eq!(arc_str.len, long.len());
    }

    #[test]
    fn test_clone_and_drop() {
        let example = "hello world!";
        let arc_str_1 = ArcString::from(example);
        let arc_str_2 = arc_str_1.clone();

        drop(arc_str_1);

        assert_eq!(arc_str_2.as_str(), example);
        assert_eq!(arc_str_2.len, example.len());
    }

    #[test]
    fn test_extend_chars() {
        let example = "hello";
        let mut arc = ArcString::from(example);

        arc.extend(" world!".chars());

        assert_eq!(arc.as_str(), "hello world!");
        assert_eq!(arc.len(), 12);
    }

    #[test]
    fn test_extend_strs() {
        let example = "hello";
        let mut arc = ArcString::from(example);

        let words = vec![" ", "world!", "my name is", " compact", "_str"];
        arc.extend(words);

        assert_eq!(arc.as_str(), "hello world!my name is compact_str");
        assert_eq!(arc.len(), 34);
    }

    #[test]
    fn test_sanity() {
        let example = "hello world!";
        let arc_str = ArcString::from(example);

        assert_eq!(arc_str.as_str(), example);
        assert_eq!(arc_str.len, example.len());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_strings_roundtrip(#[strategy(rand_unicode())] word: String) {
        let arc_str = ArcString::from(word.as_str());
        prop_assert_eq!(&word, arc_str.as_str());
    }
}

crate::asserts::assert_size!(ArcString, 2 * mem::size_of::<usize>());
