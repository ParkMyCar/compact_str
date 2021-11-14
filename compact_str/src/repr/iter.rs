//! Implementations of the `FromIterator` trait to make building `CompactStr`s easier

use core::{
    iter::FromIterator,
    mem::ManuallyDrop,
};
use super::{
    inline::MAX_INLINE_SIZE,
    Repr,
    HeapString,
    InlineString,
};

impl FromIterator<char> for Repr {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut iter = iter.into_iter();

        // If the size hint indicates we can't store this inline, then create a heap string
        let (size_hint, _) = iter.size_hint();
        if size_hint > MAX_INLINE_SIZE {
            let heap = HeapString::from(iter.collect::<String>());
            return Repr { heap: ManuallyDrop::new(heap) };
        }

        // Otherwise, continuously pull chars from the iterator
        let mut curr_len = 0;
        let mut inline_buf = [0u8; MAX_INLINE_SIZE];
        while let Some(c) = iter.next() {
            let char_len = c.len_utf8();

            // If this new character is too large to fit into the inline buffer, then create a heap string
            if char_len + curr_len > inline_buf.len() {
                let (min_remaining, _) = iter.size_hint();
                let mut heap_buf = String::with_capacity(char_len + curr_len + min_remaining);

                // push existing characters onto the heap
                // SAFETY: `inline_buf` has been filled with `char`s which are valid UTF-8
                heap_buf.push_str(unsafe { core::str::from_utf8_unchecked(&inline_buf) });
                // push current char onto the heap
                heap_buf.push(c);
                // extend heap with remaining characters
                heap_buf.extend(iter);

                let heap = HeapString::from(heap_buf);
                return Repr { heap: ManuallyDrop::new(heap) };
            }

            // write the current char into a slice of the unoccupied space
            c.encode_utf8(&mut inline_buf[curr_len..]);
            curr_len += char_len;
        }

        // SAFETY: We know `inline_buf` is valid UTF-8 because it consists entriely of `char`s
        let inline = unsafe { InlineString::from_parts(curr_len, inline_buf) };
        Repr { inline }
    }
}

impl<'a> FromIterator<&'a char> for Repr {
    fn from_iter<T: IntoIterator<Item = &'a char>>(iter: T) -> Self {
        iter.into_iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Repr;

    #[test]
    fn short_char_iter() {
        let chars = ['a', 'b', 'c'];
        let repr: Repr = chars.into_iter().collect();

        assert_eq!(repr.as_str(), "abc");
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    fn short_char_ref_iter() {
        let chars = ['a', 'b', 'c'];
        let repr: Repr = chars.iter().collect();

        assert_eq!(repr.as_str(), "abc");
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    fn long_char_iter() {
        let long = "This is supposed to be a really long string";
        let repr: Repr = long.chars().collect();

        assert_eq!(repr.as_str(), "This is supposed to be a really long string");
        assert!(repr.is_heap_allocated());
    }
}
