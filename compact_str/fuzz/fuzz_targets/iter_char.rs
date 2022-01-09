#![no_main]
use libfuzzer_sys::fuzz_target;
use compact_str::CompactStr;

const MAX_INLINE_LENGTH: usize = std::mem::size_of::<String>();

fuzz_target!(|chars: Vec<char>| {
    let compact_str: CompactStr = chars.iter().collect();
    let word: String = chars.iter().collect();

    // assert the word roundtrips
    assert_eq!(compact_str, word);

    // assert the CompactStr is properly allocated
    if word.len() < MAX_INLINE_LENGTH {
        assert!(!compact_str.is_heap_allocated());
    } else {
        assert!(compact_str.is_heap_allocated());
    }

    // TODO: Creating a CompactStr from an iterator, doesn't yet support the Packed representation
});
