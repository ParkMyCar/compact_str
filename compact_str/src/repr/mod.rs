use std::mem::ManuallyDrop;

use static_assertions::{
    assert_eq_size,
    const_assert_eq,
};

mod iter;

mod discriminant;
mod heap;
mod inline;
mod packed;

use discriminant::{
    Discriminant,
    DiscriminantMask,
};
use heap::HeapString;
use inline::InlineString;
use packed::PackedString;

const MAX_SIZE: usize = std::mem::size_of::<String>();
const EMPTY: Repr = Repr {
    inline: InlineString::new_const(""),
};

// Used as a discriminant to identify different variants
pub const HEAP_MASK: u8 = 0b11111111;
pub const LEADING_BIT_MASK: u8 = 0b10000000;

pub union Repr {
    mask: DiscriminantMask,
    heap: ManuallyDrop<HeapString>,
    inline: InlineString,
    packed: PackedString,
}

impl Repr {
    #[inline]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        let text = text.as_ref();
        let len = text.len();

        if len == 0 {
            EMPTY
        } else if len <= inline::MAX_INLINE_SIZE {
            let inline = InlineString::new(text);
            Repr { inline }
        } else if len == MAX_SIZE && text.as_bytes()[0] <= 127 {
            let packed = PackedString::new(text);
            Repr { packed }
        } else {
            let heap = ManuallyDrop::new(HeapString::new(text));
            Repr { heap }
        }
    }

    #[inline]
    pub const fn new_const(text: &str) -> Self {
        let len = text.len();

        if len <= inline::MAX_INLINE_SIZE {
            let inline = InlineString::new_const(text);
            Repr { inline }
        } else if len == MAX_SIZE && text.as_bytes()[0] <= 127 {
            let packed = PackedString::new_const(text);
            Repr { packed }
        } else {
            // HACK: This allows us to make assertions within a `const fn` without requiring
            // nightly, see unstable `const_panic` feature. This results in a build
            // failure, not a runtime panic
            #[allow(clippy::no_effect)]
            #[allow(unconditional_panic)]
            ["Trying to create a non-inline-able string at compile time!"][42];
            EMPTY
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.cast().len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cast().capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        // We want at least enough capacity to store length + additional
        let new_capacity = self.len() + additional;

        // We already have at least `additional` capacity, so we don't need to do anything
        if self.capacity() >= new_capacity {
            return;
        }

        // Note: Inlined strings (i.e. inline and packed) are already their maximum size. So if our
        // current capacity isn't large enough, then we always need to create a heap variant

        // Create a `HeapString` with `text.len() + additional` capacity
        let heap = HeapString::with_additional(self.as_str(), additional);

        // Set self to this new String
        let heap = ManuallyDrop::new(heap);
        *self = Repr { heap };
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.cast().into_str()
    }

    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        self.cast_mut().into_mut_slice()
    }

    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        self.cast_mut().set_len(length)
    }

    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        matches!(self.cast(), StrongRepr::Heap(..))
    }

    #[inline(always)]
    fn discriminant(&self) -> Discriminant {
        // SAFETY: `heap`, `inline`, and `packed` all store a discriminant in their first byte
        unsafe { self.mask.discriminant() }
    }

    #[inline(always)]
    fn cast(&self) -> StrongRepr<'_> {
        match self.discriminant() {
            Discriminant::Heap => {
                // SAFETY: We checked the discriminant to make sure the union is `heap`
                StrongRepr::Heap(unsafe { &self.heap })
            }
            Discriminant::Inline => {
                // SAFETY: We checked the discriminant to make sure the union is `inline`
                StrongRepr::Inline(unsafe { &self.inline })
            }
            Discriminant::Packed => {
                // SAFETY: We checked the discriminant to make sure the union is `packed`
                StrongRepr::Packed(unsafe { &self.packed })
            }
        }
    }

    #[inline(always)]
    fn cast_mut(&mut self) -> MutStrongRepr<'_> {
        match self.discriminant() {
            Discriminant::Heap => {
                // SAFETY: We checked the discriminant to make sure the union is `heap`
                MutStrongRepr::Heap(unsafe { &mut self.heap })
            }
            Discriminant::Inline => {
                // SAFETY: We checked the discriminant to make sure the union is `inline`
                MutStrongRepr::Inline(unsafe { &mut self.inline })
            }
            Discriminant::Packed => {
                // SAFETY: We checked the discriminant to make sure the union is `packed`
                MutStrongRepr::Packed(unsafe { &mut self.packed })
            }
        }
    }
}

impl Clone for Repr {
    fn clone(&self) -> Self {
        match self.cast() {
            StrongRepr::Heap(heap) => Repr { heap: heap.clone() },
            StrongRepr::Inline(inline) => Repr { inline: *inline },
            StrongRepr::Packed(packed) => Repr { packed: *packed },
        }
    }
}

impl Drop for Repr {
    fn drop(&mut self) {
        match self.discriminant() {
            Discriminant::Heap => {
                // SAFETY: We checked the discriminant to make sure the union is `heap`
                unsafe { ManuallyDrop::drop(&mut self.heap) };
            }
            // No-op, the value is on the stack and doesn't need to be explicitly dropped
            Discriminant::Inline | Discriminant::Packed => {}
        }
    }
}

#[derive(Debug)]
enum StrongRepr<'a> {
    Heap(&'a ManuallyDrop<HeapString>),
    Inline(&'a InlineString),
    Packed(&'a PackedString),
}

impl<'a> StrongRepr<'a> {
    #[inline]
    pub fn len(self) -> usize {
        match self {
            Self::Inline(inline) => inline.len(),
            Self::Packed(packed) => packed.len(),
            Self::Heap(heap) => heap.string.len(),
        }
    }

    #[inline]
    pub fn capacity(self) -> usize {
        match self {
            Self::Inline(inline) => inline.capacity(),
            Self::Packed(packed) => packed.capacity(),
            Self::Heap(heap) => heap.string.capacity(),
        }
    }

    #[inline]
    pub fn into_str(self) -> &'a str {
        match self {
            Self::Inline(inline) => inline.as_str(),
            Self::Packed(packed) => packed.as_str(),
            Self::Heap(heap) => heap.string.as_str(),
        }
    }
}

#[derive(Debug)]
enum MutStrongRepr<'a> {
    Heap(&'a mut ManuallyDrop<HeapString>),
    Inline(&'a mut InlineString),
    Packed(&'a mut PackedString),
}

impl<'a> MutStrongRepr<'a> {
    #[inline]
    pub unsafe fn into_mut_slice(self) -> &'a mut [u8] {
        match self {
            Self::Inline(inline) => inline.as_mut_slice(),
            Self::Packed(packed) => packed.as_mut_slice(),
            Self::Heap(heap) => heap.make_mut_slice(),
        }
    }

    #[inline]
    pub unsafe fn set_len(self, length: usize) {
        match self {
            Self::Inline(inline) => inline.set_len(length),
            Self::Packed(packed) => packed.set_len(length),
            Self::Heap(heap) => heap.set_len(length),
        }
    }
}

assert_eq_size!(Repr, String);

#[cfg(target_pointer_width = "64")]
const_assert_eq!(std::mem::size_of::<Repr>(), 24);
#[cfg(target_pointer_width = "32")]
const_assert_eq!(std::mem::size_of::<Repr>(), 12);

#[cfg(test)]
mod tests {
    use super::Repr;

    #[test]
    fn test_inline_str() {
        let short = "abc";
        let repr = Repr::new(&short);
        assert_eq!(repr.as_str(), short);
    }

    #[test]
    fn test_packed_str() {
        #[cfg(target_pointer_width = "64")]
        let packed = "this string is 24 chars!";
        #[cfg(target_pointer_width = "32")]
        let packed = "i am 12 char";

        let repr = Repr::new(&packed);
        assert_eq!(repr.as_str(), packed);
    }

    #[test]
    fn test_heap_str() {
        let long = "I am a long string that has very many characters";
        let repr = Repr::new(&long);
        assert_eq!(repr.as_str(), long);
    }

    #[test]
    fn test_reserve() {
        let word = std::mem::size_of::<usize>();

        let short = "abc";
        let mut repr = Repr::new(&short);

        assert_eq!(repr.capacity(), word * 3 - 1);
        assert_eq!(repr.as_str(), short);

        // Reserve < WORD * 3 - 1
        repr.reserve(word);
        // This shouldn't cause a resize
        assert_eq!(repr.capacity(), word * 3 - 1);
        // We should not be heap allocated
        assert!(!repr.is_heap_allocated());

        // Reserve a large amount of bytes
        repr.reserve(128);

        // We should get resized
        assert_eq!(repr.capacity(), 128 + short.len());
        // The string should still be the same
        assert_eq!(repr.as_str(), short);
        // We should be heap allocated
        assert!(repr.is_heap_allocated());
    }

    #[test]
    fn test_write_to_buffer() {
        let mut repr = Repr::new("");
        let slice = unsafe { repr.as_mut_slice() };

        let word = "abc";
        let new_len = word.len();

        // write bytes into the `CompactStr`
        slice[..new_len].copy_from_slice(word.as_bytes());
        // set the length
        unsafe { repr.set_len(new_len) }

        assert_eq!(repr.as_str(), word);
    }

    #[test]
    fn test_write_to_resized_buffer() {
        let mut repr = Repr::new("");

        // reserve additional bytes
        repr.reserve(100);
        assert!(repr.is_heap_allocated());

        let slice = unsafe { repr.as_mut_slice() };

        let long_word = "hello, I am a very long string that should be allocated on the heap";
        let new_len = long_word.len();

        slice[..new_len].copy_from_slice(long_word.as_bytes());
        unsafe { repr.set_len(new_len) }

        assert_eq!(repr.as_str(), long_word);
    }
}
