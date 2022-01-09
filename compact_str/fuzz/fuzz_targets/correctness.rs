#![no_main]
use libfuzzer_sys::fuzz_target;
use compact_str::CompactStr;

const MAX_INLINE_LENGTH: usize = std::mem::size_of::<String>();

fuzz_target!(|word: String| {
    let compact_str = CompactStr::new(&word);

    // assert the word roundtrips
    assert_eq!(compact_str, word);

    // assert the CompactStr is properly allocated
    if word.len() < MAX_INLINE_LENGTH {
        assert!(!compact_str.is_heap_allocated());
    } else if word.len() == MAX_INLINE_LENGTH && word.as_bytes()[0] <= 127 {
        assert!(!compact_str.is_heap_allocated());
    } else {
        assert!(compact_str.is_heap_allocated());
    }
});
