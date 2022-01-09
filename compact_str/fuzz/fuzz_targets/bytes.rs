#![no_main]
use libfuzzer_sys::fuzz_target;
use compact_str::CompactStr;
use std::io::Cursor;

const MAX_INLINE_LENGTH: usize = std::mem::size_of::<String>();

fuzz_target!(|data: &[u8]| {
    let mut buffer = Cursor::new(data);

    let compact = CompactStr::from_utf8_buf(&mut buffer);
    let std_str = std::str::from_utf8(data);

    match (compact, std_str) {
        (Ok(c), Ok(s)) => {
            assert_eq!(c, s);

            // assert the CompactStr is properly allocated
            if s.len() < MAX_INLINE_LENGTH {
                assert!(!c.is_heap_allocated());
            } else if s.len() == MAX_INLINE_LENGTH && s.as_bytes()[0] <= 127 {
                assert!(!c.is_heap_allocated());
            } else {
                assert!(c.is_heap_allocated());
            }
        },
        (Err(c_err), Err(s_err)) => assert_eq!(c_err, s_err),
        _ => panic!("CompactStr and core::str read UTF-8 differently?"),
    }
});
