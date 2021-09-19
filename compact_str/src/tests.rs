use crate::CompactStr;
use proptest::{prelude::*, strategy::Strategy};

#[cfg(target_pointer_width = "64")]
const MAX_INLINED_SIZE: usize = 24;
#[cfg(target_pointer_width = "32")]
const MAX_INLINED_SIZE: usize = 12;

// generates random unicode strings, upto 80 chars long
fn rand_unicode() -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::char::any(), 0..80).prop_map(|v| v.into_iter().collect())
}

proptest! {
    #[test]
    fn test_strings_roundtrip(word in rand_unicode()) {
        let compact = CompactStr::new(&word);
        prop_assert_eq!(&word, &compact);
    }


    #[test]
    fn test_strings_allocated_properly(word in rand_unicode()) {
        let compact = CompactStr::new(&word);

        if compact.len() < MAX_INLINED_SIZE {
            prop_assert!(!compact.is_heap_allocated())
        } else if compact.len() == MAX_INLINED_SIZE && compact.as_bytes()[0] <= 127 {
            prop_assert!(!compact.is_heap_allocated())
        } else {
            prop_assert!(compact.is_heap_allocated())
        }
    }
}

#[test]
fn test_short_ascii() {
    // always inlined on all archs
    let strs = ["nyc", "statue", "liberty", "img_1234.png"];

    for s in strs {
        let compact = CompactStr::new(s);
        assert_eq!(compact, s);
        assert_eq!(compact.is_heap_allocated(), false);
    }
}

#[test]
fn test_short_unicode() {
    let strs = [
        ("🦀", false),
        ("🌧☀️", false),
        #[cfg(target_pointer_width = "64")]
        ("咬𓅈ꁈ:_", false),
        // str is 12 bytes long, and leading character is non-ASCII, so it gets heap allocated
        #[cfg(target_pointer_width = "32")]
        ("咬𓅈ꁈ:_", true),
    ];

    for (s, is_heap) in strs {
        let compact = CompactStr::new(s);
        assert_eq!(compact, s);
        assert_eq!(compact.is_heap_allocated(), is_heap);
    }
}

#[test]
fn test_medium_ascii() {
    let strs = [
        "rustconf 2021",
        "new york city",
        "nyc pizza is good",
        "test the 24 char limit!!",
    ];

    for s in strs {
        let compact = CompactStr::new(s);
        assert_eq!(compact, s);

        #[cfg(target_pointer_width = "64")]
        let is_heap = false;
        #[cfg(target_pointer_width = "32")]
        let is_heap = true;
        assert_eq!(compact.is_heap_allocated(), is_heap);
    }
}

#[test]
fn test_medium_unicode() {
    let strs = [
        ("☕️👀😁🎉", false),
        // str is 24 bytes long, and leading character is non-ASCII, so it gets heap allocated
        ("🦀😀😃😄😁🦀", true),
    ];

    #[allow(unused_variables)]
    for (s, is_heap) in strs {
        let compact = CompactStr::new(s);
        assert_eq!(compact, s);

        #[cfg(target_pointer_width = "64")]
        let is_heap = is_heap;
        #[cfg(target_pointer_width = "32")]
        let is_heap = true;

        assert_eq!(compact.is_heap_allocated(), is_heap);
    }
}
