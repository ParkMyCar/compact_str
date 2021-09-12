use static_assertions::*;
use std::mem::ManuallyDrop;

use crate::metadata::Discriminant;

mod heap;
mod inline;

use heap::HeapString;
use inline::InlineString;

const MAX_SIZE: usize = std::mem::size_of::<String>();

pub union Repr {
    heap: ManuallyDrop<HeapString>,
    inline: InlineString,
}

impl Repr {
    #[inline(always)]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        let text = text.as_ref();

        if text.len() > inline::MAX_INLINE_SIZE {
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
    fn discriminant(&self) -> Discriminant {
        debug_assert_eq!(unsafe { self.inline.metadata.discriminant() }, unsafe {
            self.heap.metadata.discriminant()
        });

        // SAFETY: Both heap and inline store the discriminant as their first field
        unsafe { self.inline.metadata.discriminant() }
    }

    #[inline(always)]
    fn cast(&self) -> StrongRepr<'_> {
        match self.discriminant() {
            Discriminant::HEAP => {
                // SAFETY: We checked the discriminant to make sure the union is `heap`
                StrongRepr::Heap(unsafe { &self.heap })
            }
            Discriminant::INLINE => {
                // SAFETY: We checked the discriminant to make sure the union is `inline`
                StrongRepr::Inline(unsafe { &self.inline })
            }
            _ => unreachable!("was another value added to discriminant?"),
        }
    }
}

impl Clone for Repr {
    fn clone(&self) -> Self {
        match self.cast() {
            StrongRepr::Heap(heap) => Repr {
                heap: ManuallyDrop::new(heap.clone()),
            },
            StrongRepr::Inline(inline) => Repr { inline: *inline },
        }
    }
}

impl Drop for Repr {
    fn drop(&mut self) {
        match self.discriminant() {
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
    Inline(&'a InlineString),
    Heap(&'a HeapString),
}

impl<'a> StrongRepr<'a> {
    #[inline(always)]
    pub fn into_str(self) -> &'a str {
        match self {
            Self::Inline(inline) => {
                let len = inline.metadata.data() as usize;
                let slice = &inline.buffer[..len];

                // SAFETY: You can only construct an InlineString via a &str
                unsafe { ::std::str::from_utf8_unchecked(slice) }
            }
            Self::Heap(heap) => &*heap.string,
        }
    }
}

assert_eq_size!(Repr, String);

#[cfg(target_pointer_width = "64")]
const_assert_eq!(std::mem::size_of::<Repr>(), 24);

#[cfg(target_pointer_width = "32")]
const_assert_eq!(std::mem::size_of::<Repr>(), 12);
