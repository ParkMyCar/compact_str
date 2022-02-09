use core::mem;

use super::arc::ArcString;
use super::{
    HEAP_MASK,
    MAX_SIZE,
};

const PADDING_SIZE: usize = MAX_SIZE - mem::size_of::<ArcString>();
const PADDING: [u8; PADDING_SIZE] = [HEAP_MASK; PADDING_SIZE];

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HeapString {
    pub string: ArcString,
    padding: [u8; PADDING_SIZE],
}

impl HeapString {
    /// Creates a `HeapString` from the provided `text`.
    ///
    /// If you need to allocate a `HeapString` with additional capacity, see
    /// `HeapString::with_additional(...)`
    #[inline]
    pub fn new(text: &str) -> Self {
        let padding = PADDING;
        let string = ArcString::new(text, 0);

        HeapString { padding, string }
    }

    /// Creates a `HeapString` from the provided `text` and allocates the underlying buffer with
    /// `additional` capacity
    #[inline]
    pub fn with_additional(text: &str, additional: usize) -> Self {
        let padding = PADDING;
        let string = ArcString::new(text, additional);

        HeapString { padding, string }
    }

    /// Creates a `HeapString` with the provided capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let padding = PADDING;
        let string = ArcString::with_capacity(capacity);

        HeapString { padding, string }
    }

    /// Makes a mutable reference to the underlying buffer, cloning if there is more than one out
    /// standing reference.
    ///
    /// # Invariants
    /// * Please see `super::Repr` for all invariants
    #[inline]
    pub unsafe fn make_mut_slice(&mut self) -> &mut [u8] {
        self.string.make_mut_slice()
    }

    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        self.string.set_len(length)
    }
}

impl From<String> for HeapString {
    #[inline]
    fn from(s: String) -> Self {
        let padding = PADDING;
        let string = ArcString::from(s.as_str());

        HeapString { padding, string }
    }
}

crate::asserts::assert_size_eq!(HeapString, String);
