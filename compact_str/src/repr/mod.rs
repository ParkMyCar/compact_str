use std::iter::Extend;
use std::mem::ManuallyDrop;

use static_assertions::{
    assert_eq_size,
    const_assert_eq,
};

#[cfg(feature = "bytes")]
mod bytes;

mod iter;

mod arc;
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

        // Replace `self` with the new Repr
        let heap = ManuallyDrop::new(heap);
        *self = Repr { heap };
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.cast().into_str()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.cast().into_slice()
    }

    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        self.cast_mut().into_mut_slice()
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        let len = self.len();
        let char_len = ch.len_utf8();

        // Reserve at least enough space for our char, possibly causing a heap allocation
        self.reserve(char_len);

        // Get a mutable reference to the underlying memory buffer
        let slice = unsafe { self.as_mut_slice() };

        // Write our character into the underlying buffer
        ch.encode_utf8(&mut slice[len..]);
        // Incrament our length
        unsafe { self.set_len(len + char_len) };
    }

    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.as_str().chars().rev().next()?;

        match self.cast() {
            StrongRepr::Packed(packed) => {
                let mut inline_buffer = [0; inline::MAX_INLINE_SIZE];

                let new_len = packed.len() - ch.len_utf8();
                let buffer: &mut [u8] = &mut inline_buffer[..new_len];

                buffer.copy_from_slice(&packed.as_slice()[..new_len]);

                let inline = unsafe { InlineString::from_parts(new_len, inline_buffer) };
                *self = Repr { inline }
            }
            StrongRepr::Inline(_) | StrongRepr::Heap(_) => {
                unsafe { self.set_len(self.len() - ch.len_utf8()) };
            }
        }

        Some(ch)
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        let len = self.len();
        let str_len = s.len();

        // Reserve at least enough space for our str, possibly causing a heap allocation
        self.reserve(str_len);

        let slice = unsafe { self.as_mut_slice() };
        let buffer = &mut slice[len..len + str_len];

        debug_assert_eq!(buffer.len(), s.as_bytes().len());

        // Copy the string into our buffer
        buffer.copy_from_slice(s.as_bytes());
        // Incrament the length of our string
        unsafe { self.set_len(len + str_len) };
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

impl Extend<char> for Repr {
    #[inline]
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        let mut iterator = iter.into_iter();

        // If we're the Heap variant pass-through to HeapString's Extend impl since it's optimized
        if let MutStrongRepr::Heap(heap) = self.cast_mut() {
            heap.string.extend(iterator);
            return;
        } else {
            let (lower_bound, _) = iterator.size_hint();
            // If there is no possibility of being able to inline this string, then create a Heap
            // variant
            if self.len() + lower_bound > MAX_SIZE {
                let mut heap = HeapString::with_additional(self.as_str(), lower_bound);
                heap.string.extend(iterator);

                // Replace `self` with the new Repr
                let heap = ManuallyDrop::new(heap);
                *self = Repr { heap };
                return;
            }

            // Otherwise, keep appending elements, heap allocating if need be
            while let Some(ch) = iterator.next() {
                let len = self.len();
                let char_len = ch.len_utf8();

                // Check if we can transform into the Packed repr
                if len + char_len == MAX_SIZE && !self.is_heap_allocated() && self.as_slice()[0] <= 127 {
                    debug_assert!(len != MAX_SIZE);

                    // SAFETY: We know we're the inline variant because our current length does
                    // not equal MAX_SIZE and we're not heap allocated
                    let inline = unsafe { &mut self.inline };
                    let inline_len = inline.len();

                    let mut buffer = [0; MAX_SIZE];

                    // Copy the contents of the InlineString, into a buffer which will be a PackedString
                    buffer[..inline_len].copy_from_slice(inline.as_slice());
                    ch.encode_utf8(&mut buffer[inline_len..]);

                    // SAFETY: We created `buffer` from an InlineString which is valid UTF-8, and
                    // appended a char, which is also valid UTF-8
                    let packed = unsafe { PackedString::from_parts(buffer) };
                    *self = Repr { packed };
                } else if self.len() + char_len > self.capacity() {
                    let mut heap = HeapString::with_additional(self.as_str(), char_len);
                    heap.string.push(ch);
                    heap.string.extend(iterator);

                    // Replace `self` with the new Repr
                    let heap = ManuallyDrop::new(heap);
                    *self = Repr { heap };

                    // All done
                    return;
                } else {
                    // SAFETY: At this point we know we have the Inline variant because we would have returned
                    // already if we were the Heap variant, and the above check for capacity always returns
                    // true for the Packed variant, so we would have fallen into the case above
                    let inline = unsafe { &mut self.inline };
                    let inline_len = inline.len();

                    // SAFTEY: We're writing a `char` into the buffer, which we know is valid UTF-8
                    let buffer = unsafe { inline.as_mut_slice() };
                    // Write the character into our buffer
                    ch.encode_utf8(&mut buffer[inline_len..]);
                    // SAFETY: We just wrote `char_len` bytes into the buffer, so we know this is a
                    // valid length
                    unsafe { inline.set_len(inline_len + char_len) };
                }
            }
        }
    }
}

impl<'a> Extend<&'a char> for Repr {
    fn extend<T: IntoIterator<Item = &'a char>>(&mut self, iter: T) {
        self.extend(iter.into_iter().copied());
    }
}

impl<'a> Extend<&'a str> for Repr {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push_str(s));
    }
}

impl Extend<Box<str>> for Repr {
    fn extend<T: IntoIterator<Item = Box<str>>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl Extend<String> for Repr {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
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

    #[inline]
    pub fn into_slice(self) -> &'a [u8] {
        match self {
            Self::Inline(inline) => inline.as_slice(),
            Self::Packed(packed) => packed.as_slice(),
            Self::Heap(heap) => heap.string.as_slice(),
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

    #[test]
    fn test_push() {
        let example = "hello world";
        let mut repr = Repr::new(example);
        repr.push('!');

        assert_eq!(repr.as_str(), "hello world!");
        assert_eq!(repr.len(), 12);
    }

    #[test]
    fn test_pop() {
        let example = "hello";
        let mut repr = Repr::new(example);

        assert_eq!(repr.pop(), Some('o'));
        assert_eq!(repr.pop(), Some('l'));
        assert_eq!(repr.pop(), Some('l'));

        assert_eq!(repr.as_str(), "he");
        assert_eq!(repr.len(), 2);
    }

    #[test]
    fn test_push_str() {
        let example = "hello";
        let mut repr = Repr::new(example);

        repr.push_str(" world!");

        assert_eq!(repr.as_str(), "hello world!");
        assert_eq!(repr.len(), 12);
    }

    #[test]
    fn test_extend_chars() {
        let example = "hello";
        let mut repr = Repr::new(example);

        repr.extend(" world".chars());

        assert_eq!(repr.len(), 11);
        assert_eq!(repr.as_str(), "hello world");
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    fn test_extend_chars_can_heap_allocate() {
        let start = "hello";
        let mut repr = Repr::new(start);

        let chars = (0..100).map(|_| '!');
        repr.extend(chars);

        let mut control = String::from(start);
        let chars2 = (0..100).map(|_| '!');
        control.extend(chars2);

        assert_eq!(repr.as_str(), control.as_str());
        assert_eq!(repr.len(), 105);
    }

    #[test]
    fn test_extend_chars_can_make_packed_repr() {
        let start = "nyc";
        let mut repr = Repr::new(start);

        let num_chars = super::MAX_SIZE - start.len();
        let chars = (0..num_chars).map(|_| '!');

        repr.extend(chars);
        assert_eq!(repr.len(), super::MAX_SIZE);
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    fn test_extend_strs() {
        let example = "hello";
        let mut repr = Repr::new(example);

        let words = vec![" ", "world!", "my name is", " compact", "_str"];
        repr.extend(words);

        assert_eq!(repr.as_str(), "hello world!my name is compact_str");
        assert_eq!(repr.len(), 34);
    }

    #[test]
    fn test_pop_packed() {
        let mut repr = Repr::new("i am 24 characters long!");
        repr.pop();

        assert_eq!(repr.len(), 23);
        assert_eq!(repr.as_str(), "i am 24 characters long");
    }
}
