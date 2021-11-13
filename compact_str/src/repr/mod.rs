use static_assertions::{assert_eq_align, assert_eq_size, const_assert_eq};
use std::mem::ManuallyDrop;

mod discriminant;
mod heap;
mod inline;
mod non_max;
mod packed;

use discriminant::{Discriminant, DiscriminantMask};
use heap::HeapString;
use inline::InlineString;
use non_max::NonMaxU8;
use packed::PackedString;

const MAX_SIZE: usize = std::mem::size_of::<String>();
const EMPTY: ReprUnion = ReprUnion {
    inline: InlineString::new_const(""),
};

// Used as a discriminant to identify different variants
pub const HEAP_MASK: u8 = 0b11111110;
pub const LEADING_BIT_MASK: u8 = 0b10000000;

#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[repr(align(8))]
pub struct ReprWithNiche((NonMaxU8, [u8; MAX_SIZE - 1]));

#[cfg(target_pointer_width = "32")]
#[repr(C)]
#[repr(align(4))]
pub struct ReprWithNiche((NonMaxU8, [u8; MAX_SIZE - 1]));

#[repr(C)]
pub union ReprUnion {
    mask: DiscriminantMask,
    heap: ManuallyDrop<HeapString>,
    inline: InlineString,
    packed: PackedString,
}

impl ReprWithNiche {
    #[inline]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self::from_union(ReprUnion::new(text))
    }

    #[inline]
    pub const fn new_const(text: &str) -> Self {
        Self::from_union(ReprUnion::new_const(text))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.as_union().as_str()
    }

    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        self.as_union().is_heap_allocated()
    }

    #[inline]
    fn discriminant(&self) -> Discriminant {
        self.as_union().discriminant()
    }

    #[inline(always)]
    pub const fn from_union(repr: ReprUnion) -> Self {
        unsafe { std::mem::transmute::<ReprUnion, ReprWithNiche>(repr) }
    }

    #[inline(always)]
    pub fn as_union(&self) -> &ReprUnion {
        unsafe { &*(self as *const ReprWithNiche as *const ReprUnion) }
    }
}

impl Clone for ReprWithNiche {
    fn clone(&self) -> Self {
        Self::from_union(self.as_union().clone())
    }
}

impl Drop for ReprWithNiche {
    fn drop(&mut self) {
        match self.discriminant() {
            Discriminant::Heap => {
                // SAFETY: We checked the discriminant to make sure the union is `heap`
                unsafe {
                    let union = std::mem::transmute::<&mut ReprWithNiche, &mut ReprUnion>(self);
                    ManuallyDrop::drop(&mut union.heap)
                };
            }
            // No-op, the value is on the stack and doesn't need to be explicitly dropped
            Discriminant::Inline | Discriminant::Packed => {}
        }
    }
}

impl ReprUnion {
    #[inline]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        let text = text.as_ref();
        let len = text.len();

        if len == 0 {
            EMPTY
        } else if len <= inline::MAX_INLINE_SIZE {
            let inline = InlineString::new(text);
            ReprUnion { inline }
        } else if len == MAX_SIZE && text.as_bytes()[0] <= 127 {
            let packed = PackedString::new(text);
            ReprUnion { packed }
        } else {
            let heap = ManuallyDrop::new(HeapString::new(text));
            ReprUnion { heap }
        }
    }

    #[inline]
    pub const fn new_const(text: &str) -> Self {
        let len = text.len();

        if len <= inline::MAX_INLINE_SIZE {
            let inline = InlineString::new_const(text);
            ReprUnion { inline }
        } else if len == MAX_SIZE && text.as_bytes()[0] <= 127 {
            let packed = PackedString::new_const(text);
            ReprUnion { packed }
        } else {
            // HACK: This allows us to make assertions within a `const fn` without requiring nightly,
            // see unstable `const_panic` feature. This results in a build failure, not a runtime panic
            #[allow(clippy::no_effect)]
            #[allow(unconditional_panic)]
            ["Trying to create a non-inline-able string at compile time!"][42];
            EMPTY
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.cast().into_str()
    }

    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        matches!(self.cast(), StrongRepr::Heap(..))
    }

    #[inline]
    fn discriminant(&self) -> Discriminant {
        // SAFETY: `heap`, `inline`, and `packed` all store a discriminant in their first byte
        unsafe { self.mask.discriminant() }
    }

    #[inline]
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
}

impl Clone for ReprUnion {
    fn clone(&self) -> Self {
        match self.cast() {
            StrongRepr::Heap(heap) => ReprUnion { heap: heap.clone() },
            StrongRepr::Inline(inline) => ReprUnion { inline: *inline },
            StrongRepr::Packed(packed) => ReprUnion { packed: *packed },
        }
    }
}

impl Drop for ReprUnion {
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
    pub fn into_str(self) -> &'a str {
        match self {
            Self::Inline(inline) => inline.as_str(),
            Self::Packed(packed) => packed.as_str(),
            Self::Heap(heap) => &*heap.string,
        }
    }
}

assert_eq_size!(Option<ReprWithNiche>, ReprWithNiche);
assert_eq_size!(ReprWithNiche, ReprUnion);
assert_eq_size!(ReprUnion, String);

#[cfg(target_pointer_width = "64")]
const_assert_eq!(std::mem::align_of::<ReprUnion>(), 8);
#[cfg(target_pointer_width = "32")]
const_assert_eq!(std::mem::align_of::<ReprUnion>(), 4);
assert_eq_align!(ReprUnion, ReprWithNiche);

const_assert_eq!(std::mem::size_of::<ReprWithNiche>(), MAX_SIZE);
#[cfg(target_pointer_width = "64")]
const_assert_eq!(std::mem::size_of::<ReprUnion>(), 24);
#[cfg(target_pointer_width = "32")]
const_assert_eq!(std::mem::size_of::<ReprUnion>(), 12);

#[cfg(test)]
mod tests {
    use super::{ReprUnion, ReprWithNiche};

    #[test]
    fn test_inline_str() {
        let short = "abc";
        let repr = ReprUnion::new(&short);
        assert_eq!(repr.as_str(), short);
    }

    #[test]
    fn test_packed_str() {
        #[cfg(target_pointer_width = "64")]
        let packed = "this string is 24 chars!";
        #[cfg(target_pointer_width = "32")]
        let packed = "i am 12 char";

        let repr = ReprUnion::new(&packed);
        assert_eq!(repr.as_str(), packed);
    }

    #[test]
    fn test_heap_str() {
        let long = "I am a long string that has very many characters";
        let repr = ReprUnion::new(&long);
        assert_eq!(repr.as_str(), long);
    }

    // Test to assert that the `None` value generated by `Option<ReprWithNiche>` is truely an invalid
    // state for a `ReprUnion`
    #[test]
    #[should_panic(expected = "index out of bounds: the len is 1 but the index is 42")]
    fn test_transmute_none() {
        let none: Option<ReprWithNiche> = None;
        unsafe { std::mem::transmute::<Option<ReprWithNiche>, ReprUnion>(none) };
    }
}
