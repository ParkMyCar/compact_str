use core::mem;

use super::{
    HEAP_MASK,
    MAX_SIZE,
};

mod arc;
use arc::ArcString;

const PADDING_SIZE: usize = MAX_SIZE - mem::size_of::<ArcString>();
const PADDING: [u8; PADDING_SIZE] = [HEAP_MASK; PADDING_SIZE];

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HeapString {
    padding: [u8; PADDING_SIZE],
    pub string: ArcString,
}

impl HeapString {
    /// Creates a `HeapString` from the provided `text`.
    ///
    /// If you need to allocate a `HeapString` with additional capacity, see
    /// `HeapString::with_additional(...)`
    pub fn new(text: &str) -> Self {
        let padding = PADDING;
        let string = text.into();

        HeapString { padding, string }
    }

    /// Creates a `HeapString` from the provided `text` and allocates the underlying buffer with
    /// `additional` capacity
    pub fn with_additional(text: &str, additional: usize) -> Self {
        let padding = PADDING;
        let string = ArcString::new(text, additional);

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
    fn from(s: String) -> Self {
        let padding = PADDING;
        let string = s.as_str().into();

        HeapString { padding, string }
    }
}

static_assertions::assert_eq_size!(HeapString, String);
