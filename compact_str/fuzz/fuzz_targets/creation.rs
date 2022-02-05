#![no_main]
#![allow(clippy::if_same_then_else)]

use std::io::Cursor;

use arbitrary::Arbitrary;
use compact_str::CompactStr;
use libfuzzer_sys::fuzz_target;

const MAX_INLINE_LENGTH: usize = std::mem::size_of::<String>();

#[derive(Arbitrary, Debug)]
enum CreationMethod<'a> {
    Bytes(&'a [u8]),
    IterChar(Vec<char>),
    IterString(Vec<String>),
    Word(String),
}

fuzz_target!(|method: CreationMethod<'_>| {
    use CreationMethod::*;

    match method {
        // Create a `CompactStr` from a `String`
        Word(word) => {
            let compact = CompactStr::new(&word);
            assert_eq!(compact, word);

            // assert the CompactStr is properly allocated
            if word.len() < MAX_INLINE_LENGTH {
                assert!(!compact.is_heap_allocated());
            } else if word.len() == MAX_INLINE_LENGTH && word.as_bytes()[0] != 0b11111111 && word.as_bytes()[0] >> 6 != 0b00000010 {
                assert!(!compact.is_heap_allocated());
            } else {
                assert!(compact.is_heap_allocated());
            }
        }
        // Create a `CompactStr` from an iterator of `char`s
        IterChar(chars) => {
            let compact: CompactStr = chars.iter().collect();
            let std_str: String = chars.iter().collect();

            assert_eq!(compact, std_str);

            // assert the CompactStr is properly allocated
            //
            // Note: Creating a CompactStr from an iterator doesn't yet support the Packed
            // representation, so we can only inline MAX_INLINE_LENGTH - 1
            if std_str.len() < MAX_INLINE_LENGTH {
                assert!(!compact.is_heap_allocated());
            } else {
                assert!(compact.is_heap_allocated());
            }
        }
        // Create a `CompactStr` from an iterator of `String`s
        IterString(strings) => {
            let compact: CompactStr = strings.iter().map::<&str, _>(|s| s.as_ref()).collect();
            let std_str: String = strings.iter().map::<&str, _>(|s| s.as_ref()).collect();

            assert_eq!(compact, std_str);

            // assert the CompactStr is properly allocated
            //
            // Note: Creating a CompactStr from an iterator doesn't yet support the Packed
            // representation, so we can only inline MAX_INLINE_LENGTH - 1
            if std_str.len() < MAX_INLINE_LENGTH {
                assert!(!compact.is_heap_allocated());
            } else {
                assert!(compact.is_heap_allocated());
            }
        }
        // Create a `CompactStr` from a buffer of bytes
        Bytes(data) => {
            let mut buffer = Cursor::new(data);

            let compact = CompactStr::from_utf8_buf(&mut buffer);
            let std_str = std::str::from_utf8(data);

            match (compact, std_str) {
                (Ok(c), Ok(s)) => {
                    assert_eq!(c, s);

                    // assert the CompactStr is properly allocated
                    if s.len() < MAX_INLINE_LENGTH {
                        assert!(!c.is_heap_allocated());
                    } else if s.len() == MAX_INLINE_LENGTH && s.as_bytes()[0] != 0b11111111 && s.as_bytes()[0] >> 6 != 0b00000010 {
                        assert!(!c.is_heap_allocated());
                    } else {
                        assert!(c.is_heap_allocated());
                    }
                }
                (Err(c_err), Err(s_err)) => assert_eq!(c_err, s_err),
                _ => panic!("CompactStr and core::str read UTF-8 differently?"),
            }
        }
    }
});
