use std::borrow::Cow;
use std::fmt;
use std::iter::Extend;
use std::mem::ManuallyDrop;
use std::str::Utf8Error;

#[cfg(feature = "bytes")]
mod bytes;

mod iter;

mod arc;
mod boxed;
mod discriminant;
mod heap;
mod inline;
mod num;

mod traits;
use discriminant::{
    Discriminant,
    DiscriminantMask,
};
use heap::HeapString;
use inline::InlineString;
pub use traits::IntoRepr;

pub(crate) const MAX_SIZE: usize = std::mem::size_of::<String>();
const EMPTY: Repr = Repr {
    inline: InlineString::new_const(""),
};

/// Used as a discriminant to identify different variants
pub const HEAP_MASK: u8 = 0b11111111;

pub union Repr {
    mask: DiscriminantMask,
    heap: ManuallyDrop<HeapString>,
    inline: InlineString,
}

impl Repr {
    #[inline]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        let text = text.as_ref();
        let len = text.len();

        if len == 0 {
            EMPTY
        } else if len <= MAX_SIZE {
            let inline = InlineString::new(text);
            Repr { inline }
        } else {
            let heap = ManuallyDrop::new(HeapString::new(text));
            Repr { heap }
        }
    }

    #[inline]
    pub const fn new_const(text: &str) -> Self {
        let len = text.len();

        if len <= MAX_SIZE {
            let inline = InlineString::new_const(text);
            Repr { inline }
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
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= MAX_SIZE {
            EMPTY
        } else {
            let heap = ManuallyDrop::new(HeapString::with_capacity(capacity));
            Repr { heap }
        }
    }

    #[inline]
    pub fn from_utf8<B: AsRef<[u8]>>(buf: B) -> Result<Self, Utf8Error> {
        // Get a &str from the Vec, failing if it's not valid UTF-8
        let s = core::str::from_utf8(buf.as_ref())?;
        // Construct a Repr from the &str
        Ok(Self::new(s))
    }

    #[inline]
    pub fn from_string(s: String) -> Self {
        if s.capacity() == 0 {
            EMPTY
        } else {
            let heap = ManuallyDrop::new(HeapString::from_string(s));
            Repr { heap }
        }
    }

    #[inline]
    pub fn from_box_str(b: Box<str>) -> Self {
        if b.len() == 0 {
            EMPTY
        } else {
            let heap = ManuallyDrop::new(HeapString::from_box_str(b));
            Repr { heap }
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

        if new_capacity <= MAX_SIZE {
            // It's possible to have a `CompactStr` that is heap allocated with a capacity less than
            // MAX_SIZE, if that `CompactStr` was created From a String or Box<str>.
            let inline = InlineString::new(self.as_str());
            *self = Repr { inline }
        } else {
            // Create a `HeapString` with `text.len() + additional` capacity
            let heap = HeapString::with_additional(self.as_str(), additional);

            // Replace `self` with the new Repr
            let heap = ManuallyDrop::new(heap);
            *self = Repr { heap };
        }
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

        // SAFETY: We know this is is a valid length which falls on a char boundary
        unsafe { self.set_len(self.len() - ch.len_utf8()) };

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
        matches!(self.discriminant(), Discriminant::Heap)
    }

    #[inline(always)]
    fn discriminant(&self) -> Discriminant {
        // SAFETY: `heap` and `inline` all store a discriminant in their last byte
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
        }
    }
}

impl Clone for Repr {
    fn clone(&self) -> Self {
        match self.cast() {
            StrongRepr::Heap(heap) => Repr { heap: heap.clone() },
            StrongRepr::Inline(inline) => Repr { inline: *inline },
        }
    }
}

impl Drop for Repr {
    #[inline]
    fn drop(&mut self) {
        // By "outlining" the actual Drop code and only calling it if we're a heap variant, it
        // allows dropping an inline variant to be as cheap as possible.
        if self.is_heap_allocated() {
            outlined_drop(self)
        }

        #[inline(never)]
        fn outlined_drop(this: &mut Repr) {
            match this.discriminant() {
                Discriminant::Heap => {
                    // SAFETY: We checked the discriminant to make sure the union is `heap`
                    unsafe { ManuallyDrop::drop(&mut this.heap) };
                }
                // No-op, the value is on the stack and doesn't need to be explicitly dropped
                Discriminant::Inline => {}
            }
        }
    }
}

impl Extend<char> for Repr {
    #[inline]
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        let mut iterator = iter.into_iter().peekable();

        // if the iterator is empty, no work needs to be done!
        if iterator.peek().is_none() {
            return;
        }
        let (lower_bound, _) = iterator.size_hint();

        match self.cast_mut() {
            MutStrongRepr::Heap(heap) => heap.string.extend(iterator),
            MutStrongRepr::Inline(inline) => {
                // Check if the lower_bound of the iterator indicates we'll need to heap allocate
                if lower_bound + inline.len() > MAX_SIZE {
                    let mut heap = HeapString::with_additional(inline.as_str(), lower_bound);
                    heap.string.extend(iterator);

                    // Replace `self` with the new Repr
                    let heap = ManuallyDrop::new(heap);
                    *self = Repr { heap };
                    return;
                }

                // Keep pulling characters off the iterator, eventually heap allocating if we run
                // out of space inline!
                while let Some(ch) = iterator.next() {
                    let inline_len = inline.len();
                    let char_len = ch.len_utf8();

                    if inline_len + char_len <= MAX_SIZE {
                        // SAFTEY: We're writing a `char` into the buffer, which we know is valid
                        // UTF-8
                        let buffer = unsafe { inline.as_mut_slice() };
                        // Write the character into our buffer
                        ch.encode_utf8(&mut buffer[inline_len..]);
                        // SAFETY: We just wrote `char_len` bytes into the buffer, so we know this
                        // is a valid length
                        unsafe { inline.set_len(inline_len + char_len) };
                    } else {
                        // We can't fit the remainder of the iterator in an InlineString, so we
                        // either need to make a HeapString
                        let mut heap = HeapString::with_additional(inline.as_str(), lower_bound);

                        // push the char we just popped off, but couldn't fit inline
                        heap.string.push(ch);
                        // write in the rest of the iterator!
                        heap.string.extend(iterator);

                        // Replace `self` with the new Repr
                        let heap = ManuallyDrop::new(heap);
                        *self = Repr { heap };

                        // All done!
                        return;
                    }
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

impl<'a> Extend<Cow<'a, str>> for Repr {
    fn extend<T: IntoIterator<Item = Cow<'a, str>>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl Extend<String> for Repr {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl fmt::Write for Repr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

#[derive(Debug)]
enum StrongRepr<'a> {
    Inline(&'a InlineString),
    Heap(&'a ManuallyDrop<HeapString>),
}

impl<'a> StrongRepr<'a> {
    #[inline]
    pub fn len(self) -> usize {
        match self {
            Self::Inline(inline) => inline.len(),
            Self::Heap(heap) => heap.string.len(),
        }
    }

    #[inline]
    pub fn capacity(self) -> usize {
        match self {
            Self::Inline(inline) => inline.capacity(),
            Self::Heap(heap) => heap.string.capacity(),
        }
    }

    #[inline]
    pub fn into_str(self) -> &'a str {
        match self {
            Self::Inline(inline) => inline.as_str(),
            Self::Heap(heap) => heap.string.as_str(),
        }
    }

    #[inline]
    pub fn into_slice(self) -> &'a [u8] {
        match self {
            Self::Inline(inline) => inline.as_slice(),
            Self::Heap(heap) => heap.string.as_slice(),
        }
    }
}

#[derive(Debug)]
enum MutStrongRepr<'a> {
    Inline(&'a mut InlineString),
    Heap(&'a mut ManuallyDrop<HeapString>),
}

impl<'a> MutStrongRepr<'a> {
    #[inline]
    pub unsafe fn into_mut_slice(self) -> &'a mut [u8] {
        match self {
            Self::Inline(inline) => inline.as_mut_slice(),
            Self::Heap(heap) => heap.make_mut_slice(),
        }
    }

    #[inline]
    pub unsafe fn set_len(self, length: usize) {
        match self {
            Self::Inline(inline) => inline.set_len(length),
            Self::Heap(heap) => heap.set_len(length),
        }
    }
}

crate::asserts::assert_size_eq!(Repr, String);

#[cfg(target_pointer_width = "64")]
crate::asserts::assert_size!(Repr, 24);
#[cfg(target_pointer_width = "32")]
crate::asserts::assert_size!(Repr, 12);

#[cfg(test)]
mod tests {
    use super::{
        Repr,
        MAX_SIZE,
    };

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

        assert_eq!(repr.capacity(), word * 3);
        assert_eq!(repr.as_str(), short);

        // Reserve < WORD * 3
        repr.reserve(word);
        // This shouldn't cause a resize
        assert_eq!(repr.capacity(), word * 3);
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
    fn test_extend_short_heap_variant() {
        let start = "this is a long string that will be heap allocated";
        let mut repr = Repr::new(start);
        assert!(repr.is_heap_allocated());

        while repr.len() > 10 {
            assert!(repr.pop().is_some());
        }
        // we don't (yet?) re-inline when characters are popped
        assert!(repr.is_heap_allocated());

        // extend with one character
        repr.extend(vec!['!']);
        // should still be heap allocated
        assert!(repr.is_heap_allocated());

        // extend with a lot of characters
        repr.extend((0..100).map(|_| '😊'));
        // should still be heap allocated
        assert!(repr.is_heap_allocated());
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

    #[test]
    fn test_pop_does_not_reinline() {
        let mut repr = Repr::new("i am a long string that will be heap allocated");
        assert!(repr.is_heap_allocated());

        while repr.len() > 10 {
            repr.pop();
        }
        // should not have re-inlined the string
        assert!(repr.is_heap_allocated());
    }

    #[test]
    fn test_from_small_string_then_mutate() {
        let s = String::from("hello world");
        assert_eq!(s.capacity(), 11);

        let mut repr = Repr::from_string(s);

        // When converting from a String, we defer inlining the string until necessary to prevent
        // dropping the heap allocated buffer
        assert_eq!(repr.capacity(), 11);
        assert!(repr.is_heap_allocated());

        repr.push('!');

        // Once we push a character we'll need to resize, it's at this point we'll inline the string
        // since we need to drop the original buffer anyways
        assert_eq!(repr.capacity(), MAX_SIZE);
        assert!(!repr.is_heap_allocated());
        assert_eq!(repr.as_str(), "hello world!");
    }

    #[test]
    fn test_from_small_box_str_then_mutate() {
        let b = String::from("hello world").into_boxed_str();
        assert_eq!(b.len(), 11);

        let mut repr = Repr::from_box_str(b);

        // When converting from a Box<str>, we defer inlining the string until necessary to prevent
        // dropping the heap allocated buffer
        assert_eq!(repr.capacity(), 11);
        assert!(repr.is_heap_allocated());

        repr.push('!');

        // Once we push a character we'll need to resize, it's at this point we'll inline the string
        // since we need to drop the original buffer anyways
        assert_eq!(repr.capacity(), MAX_SIZE);
        assert!(!repr.is_heap_allocated());
        assert_eq!(repr.as_str(), "hello world!");
    }
}
