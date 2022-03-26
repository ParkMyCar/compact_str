// use super::arc::ArcString;
use super::boxed::BoxString;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HeapString {
    pub string: BoxString,
}

impl HeapString {
    /// Creates a `HeapString` from the provided `text`.
    ///
    /// If you need to allocate a `HeapString` with additional capacity, see
    /// `HeapString::with_additional(...)`
    #[inline]
    pub fn new(text: &str) -> Self {
        let string = BoxString::new(text);
        HeapString { string }
    }

    /// Creates a `HeapString` from the provided `text` and allocates the underlying buffer with
    /// `additional` capacity
    #[inline]
    pub fn with_additional(text: &str, additional: usize) -> Self {
        let string = BoxString::with_additional(text, additional);
        HeapString { string }
    }

    /// Creates a `HeapString` with the provided capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let string = BoxString::with_capacity(capacity);
        HeapString { string }
    }

    #[inline]
    pub fn from_string(s: String) -> Self {
        let string = BoxString::from_string(s);
        HeapString { string }
    }

    /// Makes a mutable reference to the underlying buffer.
    ///
    /// # Invariants
    /// * Please see `super::Repr` for all invariants
    #[inline]
    pub unsafe fn make_mut_slice(&mut self) -> &mut [u8] {
        self.string.as_mut_slice()
    }

    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        self.string.set_len(length)
    }
}

crate::asserts::assert_size_eq!(HeapString, String);
