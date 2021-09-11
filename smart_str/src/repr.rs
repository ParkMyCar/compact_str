use static_assertions::*;
use std::{
    mem::{self, ManuallyDrop},
    sync::Arc,
};

use crate::metadata::{Discriminant, Metadata};

const MAX_SIZE: usize = mem::size_of::<String>();
const MAX_INLINE_SIZE: usize = MAX_SIZE - 1;

pub union Repr {
    heap: ManuallyDrop<HeapString>,
    inline: InlineString,
}

impl Repr {
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        let text = text.as_ref();

        if text.len() > MAX_INLINE_SIZE {
            let heap = ManuallyDrop::new(HeapString::new(text));
            Repr { heap }
        } else {
            let inline = InlineString::new(text);
            Repr { inline }
        }
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self.cast().into_str()
    }

    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        matches!(self.cast(), StrongRepr::Heap(..))
    }

    #[inline(always)]
    fn cast(&self) -> StrongRepr<'_> {
        let metadata = unsafe { self.inline.metadata };
        let discriminant = metadata.discriminant();

        // both `heap` and `inline` store the discriminant as their first field, so we should be able
        // to access it via either entry of the union
        debug_assert_eq!(discriminant, unsafe { self.heap.metadata.discriminant() });

        match discriminant {
            Discriminant::HEAP => {
                // SAFETY: We checked the discriminant to make sure the union is `heap`
                StrongRepr::Heap(unsafe { &self.heap.string })
            }
            Discriminant::INLINE => {
                let len = metadata.data() as usize;

                // SAFETY: We checked the discriminant to make sure the union is `inline`
                let slice = unsafe { &self.inline.string[..len] };
                StrongRepr::Inline(unsafe { ::std::str::from_utf8_unchecked(slice) })
            }
            _ => unreachable!("was another value added to discriminant?"),
        }
    }
}

impl Drop for Repr {
    fn drop(&mut self) {
        let metadata = unsafe { self.inline.metadata };
        let discriminant = metadata.discriminant();

        debug_assert_eq!(discriminant, unsafe { self.heap.metadata.discriminant() });

        match discriminant {
            Discriminant::HEAP => {
                // SAFETY: We checked the discriminant to make sure the union is `heap`
                unsafe { ManuallyDrop::drop(&mut self.heap) };
            }
            // No-op, the value is on the stack and doesn't need to be explicitly dropped
            Discriminant::INLINE => {}
            _ => unreachable!("was another value added to discriminant?"),
        }
    }
}

enum StrongRepr<'a> {
    Inline(&'a str),
    Heap(&'a Arc<str>),
}

impl<'a> StrongRepr<'a> {
    #[inline(always)]
    pub fn into_str(self) -> &'a str {
        use StrongRepr::*;

        match self {
            Inline(s) => s,
            Heap(s) => &*s,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct InlineString {
    metadata: Metadata,
    string: [u8; MAX_INLINE_SIZE],
}

impl InlineString {
    pub fn new(text: &str) -> Self {
        debug_assert!(text.len() <= MAX_INLINE_SIZE);

        let metadata = Metadata::new_inline(text);
        let mut string = [0u8; MAX_INLINE_SIZE];

        string[..text.len()].copy_from_slice(text.as_bytes());

        InlineString { metadata, string }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
struct HeapString {
    metadata: Metadata,
    padding: [u8; 7],
    string: Arc<str>,
}

impl HeapString {
    pub fn new(text: &str) -> Self {
        let metadata = Metadata::new_heap();
        let padding = [0; 7];
        let string = text.into();

        HeapString {
            metadata,
            padding,
            string,
        }
    }
}

assert_eq_size!(InlineString, HeapString);

#[cfg(target_pointer_width = "64")]
const_assert_eq!(mem::size_of::<Repr>(), 24);

#[cfg(target_pointer_width = "32")]
const_assert_eq!(mem::size_of::<Repr>(), 16);
