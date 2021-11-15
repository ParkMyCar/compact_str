//! Implementations of the `FromIterator` trait to make building `CompactStr`s more ergonomic

use core::iter::FromIterator;
use core::mem::ManuallyDrop;

use super::inline::MAX_INLINE_SIZE;
use super::{
    HeapString,
    InlineString,
    Repr,
};

impl FromIterator<char> for Repr {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut iter = iter.into_iter();

        // If the size hint indicates we can't store this inline, then create a heap string
        let (size_hint, _) = iter.size_hint();
        if size_hint > MAX_INLINE_SIZE {
            let heap = HeapString::from(iter.collect::<String>());
            return Repr {
                heap: ManuallyDrop::new(heap),
            };
        }

        // Otherwise, continuously pull chars from the iterator
        let mut curr_len = 0;
        let mut inline_buf = [0u8; MAX_INLINE_SIZE];
        while let Some(c) = iter.next() {
            let char_len = c.len_utf8();

            // If this new character is too large to fit into the inline buffer, then create a heap
            // string
            if char_len + curr_len > MAX_INLINE_SIZE {
                let (min_remaining, _) = iter.size_hint();
                let mut heap_buf = String::with_capacity(char_len + curr_len + min_remaining);

                // push existing characters onto the heap
                // SAFETY: `inline_buf` has been filled with `char`s which are valid UTF-8
                heap_buf
                    .push_str(unsafe { core::str::from_utf8_unchecked(&inline_buf[..curr_len]) });
                // push current char onto the heap
                heap_buf.push(c);
                // extend heap with remaining characters
                heap_buf.extend(iter);

                let heap = HeapString::from(heap_buf);
                return Repr {
                    heap: ManuallyDrop::new(heap),
                };
            }

            // write the current char into a slice of the unoccupied space
            c.encode_utf8(&mut inline_buf[curr_len..]);
            curr_len += char_len;
        }

        // TODO: Support PackedString here in an efficient way

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

fn from_as_ref_str_iterator<S, I>(mut iter: I) -> Repr
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
    String: core::iter::Extend<S>,
    String: FromIterator<S>,
{
    // If there are more strings than we can fit bytes inline, then immediately make a `HeapString`
    let (size_hint, _) = iter.size_hint();
    if size_hint > MAX_INLINE_SIZE {
        let heap = HeapString::from(iter.collect::<String>());
        return Repr {
            heap: ManuallyDrop::new(heap),
        };
    }

    // Otherwise, continuously pull strings from the iterator
    let mut curr_len = 0;
    let mut inline_buf = [0u8; MAX_INLINE_SIZE];
    while let Some(s) = iter.next() {
        let str_slice = s.as_ref();
        let bytes_len = str_slice.len();

        // this new string is too large to fit into our inline buffer, so heap allocate the rest
        if bytes_len + curr_len > MAX_INLINE_SIZE {
            let (min_remaining, _) = iter.size_hint();
            let mut heap_buf = String::with_capacity(bytes_len + curr_len + min_remaining);

            // push existing strings onto the heap
            // SAFETY: `inline_buf` has been filled with `&str`s which are valid UTF-8
            heap_buf.push_str(unsafe { core::str::from_utf8_unchecked(&inline_buf[..curr_len]) });
            // push current string onto the heap
            heap_buf.push_str(str_slice);
            // extend heap with remaining strings
            heap_buf.extend(iter);

            let heap = HeapString::from(heap_buf);
            return Repr {
                heap: ManuallyDrop::new(heap),
            };
        }

        // write the current string into a slice of the unoccupied space
        (&mut inline_buf[curr_len..][..bytes_len]).copy_from_slice(str_slice.as_bytes());
        curr_len += bytes_len;
    }

    // TODO: Support PackedString here in an efficient way

    // SAFETY: We know `inline_buf` is valid UTF-8 because it consists entriely of `&str`s
    let inline = unsafe { InlineString::from_parts(curr_len, inline_buf) };
    Repr { inline }
}

impl<'a> FromIterator<&'a str> for Repr {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        from_as_ref_str_iterator(iter.into_iter())
    }
}

impl FromIterator<Box<str>> for Repr {
    fn from_iter<T: IntoIterator<Item = Box<str>>>(iter: T) -> Self {
        from_as_ref_str_iterator(iter.into_iter())
    }
}

impl FromIterator<String> for Repr {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        from_as_ref_str_iterator(iter.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::Repr;

    #[test]
    fn short_char_iter() {
        let chars = ['a', 'b', 'c'];
        let repr: Repr = chars.iter().collect();

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

    #[test]
    fn short_string_iter() {
        let strings = vec!["hello", "world"];
        let repr: Repr = strings.into_iter().collect();

        assert_eq!(repr.as_str(), "helloworld");
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    fn long_short_string_iter() {
        let strings = vec![
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
            "17", "18", "19", "20",
        ];
        let repr: Repr = strings.into_iter().collect();

        assert_eq!(repr.as_str(), "1234567891011121314151617181920");
        assert!(repr.is_heap_allocated());
    }
}
