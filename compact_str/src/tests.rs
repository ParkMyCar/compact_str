use std::str::FromStr;

use proptest::prelude::*;
use proptest::strategy::Strategy;

use crate::CompactStr;

#[cfg(target_pointer_width = "64")]
const MAX_SIZE: usize = 24;
#[cfg(target_pointer_width = "32")]
const MAX_SIZE: usize = 12;

// generates random unicode strings, upto 80 chars long
fn rand_unicode() -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::char::any(), 0..80).prop_map(|v| v.into_iter().collect())
}

/// `proptest::Strategy` that generates `String`s with up to `len` bytes
pub fn rand_unicode_bytes(len: usize) -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::char::any(), 0..len).prop_map(move |chars| {
        let mut len_utf8 = 0;
        chars
            .into_iter()
            .take_while(|c| {
                len_utf8 += c.len_utf8();
                len_utf8 <= len
            })
            .collect::<String>()
    })
}

// generates groups upto 40 strings long of random unicode strings, upto 80 chars long
fn rand_unicode_collection() -> impl Strategy<Value = Vec<String>> {
    proptest::collection::vec(rand_unicode(), 0..40)
}

/// Asserts a CompactStr is allocated properly
fn assert_allocated_properly(compact: &CompactStr) {
    if compact.len() <= MAX_SIZE {
        assert!(!compact.is_heap_allocated())
    } else {
        assert!(compact.is_heap_allocated())
    }
}

proptest! {
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_strings_roundtrip(word in rand_unicode()) {
        let compact = CompactStr::new(&word);
        prop_assert_eq!(&word, &compact);
    }


    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_strings_allocated_properly(word in rand_unicode()) {
        let compact = CompactStr::new(&word);
        assert_allocated_properly(&compact);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_char_iterator_roundtrips(word in rand_unicode()) {
        let compact: CompactStr = word.clone().chars().collect();
        prop_assert_eq!(&word, &compact)
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_string_iterator_roundtrips(collection in rand_unicode_collection()) {
        let compact: CompactStr = collection.clone().into_iter().collect();
        let word: String = collection.into_iter().collect();
        prop_assert_eq!(&word, &compact);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_reserve_and_write_bytes(word in rand_unicode()) {
        let mut compact = CompactStr::default();
        prop_assert!(compact.is_empty());

        // reserve enough space to write our bytes
        compact.reserve(word.len());

        // SAFETY: We're writing a String which we know is UTF-8
        let slice = unsafe { compact.as_mut_bytes() };
        slice[..word.len()].copy_from_slice(word.as_bytes());

        // SAFTEY: We know this is the length of our string, since `compact` started with 0 bytes
        // and we just wrote `word.len()` bytes
        unsafe { compact.set_len(word.len()) }

        prop_assert_eq!(&word, &compact);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_reserve_and_write_bytes_allocated_properly(word in rand_unicode()) {
        let mut compact = CompactStr::default();
        prop_assert!(compact.is_empty());

        // reserve enough space to write our bytes
        compact.reserve(word.len());

        // SAFETY: We're writing a String which we know is UTF-8
        let slice = unsafe { compact.as_mut_bytes() };
        slice[..word.len()].copy_from_slice(word.as_bytes());

        // SAFTEY: We know this is the length of our string, since `compact` started with 0 bytes
        // and we just wrote `word.len()` bytes
        unsafe { compact.set_len(word.len()) }

        prop_assert_eq!(compact.len(), word.len());

        // The string should be heap allocated if `word` was > MAX_SIZE
        //
        // NOTE: The reserve and write API's don't currently support the Packed representation
        prop_assert_eq!(compact.is_heap_allocated(), word.len() > MAX_SIZE);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_extend_chars_allocated_properly((start, extend) in (rand_unicode(), rand_unicode())) {
        let mut compact = CompactStr::new(&start);
        compact.extend(extend.chars());

        let mut control = start.clone();
        control.extend(extend.chars());

        prop_assert_eq!(&compact, &control);
        assert_allocated_properly(&compact);
    }
}

#[test]
fn test_const_creation() {
    const EMPTY: CompactStr = CompactStr::new_inline("");
    const SHORT: CompactStr = CompactStr::new_inline("rust");

    #[cfg(target_pointer_width = "64")]
    const PACKED: CompactStr = CompactStr::new_inline("i am 24 characters long!");
    #[cfg(target_pointer_width = "32")]
    const PACKED: CompactStr = CompactStr::new_inline("i am 12 char");

    assert_eq!(EMPTY, CompactStr::new(""));
    assert_eq!(SHORT, CompactStr::new("rust"));

    #[cfg(target_pointer_width = "64")]
    assert_eq!(PACKED, CompactStr::new("i am 24 characters long!"));
    #[cfg(target_pointer_width = "32")]
    assert_eq!(PACKED, CompactStr::new("i am 12 char"));
}

#[test]
fn test_short_ascii() {
    // always inlined on all archs
    let strs = vec!["nyc", "statue", "liberty", "img_1234.png"];

    for s in strs {
        let compact = CompactStr::new(s);
        assert_eq!(compact, s);
        assert_eq!(s, compact);
        assert_eq!(compact.is_heap_allocated(), false);
    }
}

#[test]
fn test_short_unicode() {
    let strs = vec![
        ("🦀", false),
        ("🌧☀️", false),
        // str is 12 bytes long, and leading character is non-ASCII
        ("咬𓅈ꁈ:_", false),
    ];

    for (s, is_heap) in strs {
        let compact = CompactStr::new(s);
        assert_eq!(compact, s);
        assert_eq!(s, compact);
        assert_eq!(compact.is_heap_allocated(), is_heap);
    }
}

#[test]
fn test_medium_ascii() {
    let strs = vec![
        "rustconf 2021",
        "new york city",
        "nyc pizza is good",
        "test the 24 char limit!!",
    ];

    for s in strs {
        let compact = CompactStr::new(s);
        assert_eq!(compact, s);
        assert_eq!(s, compact);

        #[cfg(target_pointer_width = "64")]
        let is_heap = false;
        #[cfg(target_pointer_width = "32")]
        let is_heap = true;
        assert_eq!(compact.is_heap_allocated(), is_heap);
    }
}

#[test]
fn test_medium_unicode() {
    let strs = vec![
        ("☕️👀😁🎉", false),
        // str is 24 bytes long, and leading character is non-ASCII
        ("🦀😀😃😄😁🦀", false),
    ];

    #[allow(unused_variables)]
    for (s, is_heap) in strs {
        let compact = CompactStr::new(s);
        assert_eq!(compact, s);
        assert_eq!(s, compact);

        #[cfg(target_pointer_width = "64")]
        let is_heap = is_heap;
        #[cfg(target_pointer_width = "32")]
        let is_heap = true;

        assert_eq!(compact.is_heap_allocated(), is_heap);
    }
}

#[test]
fn test_from_str_trait() {
    let s = "hello_world";

    // Until the never type `!` is stabilized, we have to unwrap here
    let c = CompactStr::from_str(s).unwrap();

    assert_eq!(s, c);
}

#[test]
#[cfg_attr(target_pointer_width = "32", ignore)]
fn test_from_char_iter() {
    let s = "\u{0} 0 \u{0}a𐀀𐀀 𐀀a𐀀";
    println!("{}", s.len());
    let compact: CompactStr = s.chars().into_iter().collect();

    assert!(!compact.is_heap_allocated());
    assert_eq!(s, compact);
}

#[test]
#[cfg_attr(target_pointer_width = "32", ignore)]
fn test_extend_packed_from_empty() {
    let s = "  0\u{80}A\u{0}𐀀 𐀀¡a𐀀0";

    let mut compact = CompactStr::new(s);
    assert!(!compact.is_heap_allocated());

    // extend from an empty iterator
    compact.extend("".chars());

    // we should still be heap allocated
    assert!(!compact.is_heap_allocated());
}

#[test]
fn test_pop_empty() {
    let num_pops = 256;
    let mut compact = CompactStr::from("");

    (0..num_pops).for_each(|_| {
        let ch = compact.pop();
        assert!(ch.is_none());
    });
    assert!(compact.is_empty());
    assert_eq!(compact, "");
}

#[test]
fn test_compact_str_is_send_and_sync() {
    fn is_send_and_sync<T: Send + Sync>() {}
    is_send_and_sync::<CompactStr>();
}
