use core::str::Utf8Error;

use bytes::Buf;

use crate::{
    CompactStr,
    Repr,
};

impl CompactStr {
    /// Converts a buffer of bytes to a `CompactStr`
    pub fn from_utf8_buf<B: Buf>(buf: &mut B) -> Result<Self, Utf8Error> {
        Repr::from_utf8_buf(buf).map(|repr| CompactStr { repr })
    }

    /// Converts a buffer of bytes to a `CompactStr`, without checking that the provided buffer is
    /// valid UTF-8.
    pub unsafe fn from_utf8_buf_unchecked<B: Buf>(buf: &mut B) -> Self {
        let repr = Repr::from_utf8_buf_unchecked(buf);
        CompactStr { repr }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use proptest::prelude::*;
    use proptest::strategy::Strategy;

    use crate::CompactStr;

    const MAX_INLINED_SIZE: usize = core::mem::size_of::<String>();

    // generates random unicode strings, upto 80 chars long
    fn rand_unicode() -> impl Strategy<Value = String> {
        proptest::collection::vec(proptest::char::any(), 0..80)
            .prop_map(|v| v.into_iter().collect())
    }

    proptest! {
        #[test]
        #[cfg_attr(miri, ignore)]
        fn test_buffers_roundtrip(word in rand_unicode()) {
            let mut buf = Cursor::new(word.as_bytes());
            let compact = CompactStr::from_utf8_buf(&mut buf).unwrap();

            prop_assert_eq!(&word, &compact);
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn test_allocated_properly(word in rand_unicode()) {
            let mut buf = Cursor::new(word.as_bytes());
            let compact = CompactStr::from_utf8_buf(&mut buf).unwrap();

            if word.len() < MAX_INLINED_SIZE {
                prop_assert!(!compact.is_heap_allocated())
            } else if word.len() == MAX_INLINED_SIZE && word.as_bytes()[0] <= 127 {
                prop_assert!(!compact.is_heap_allocated())
            } else {
                prop_assert!(compact.is_heap_allocated())
            }
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn test_only_accept_valid_utf8(bytes in proptest::collection::vec(any::<u8>(), 0..80)) {
            let mut buf = Cursor::new(bytes.as_slice());

            let compact_result = CompactStr::from_utf8_buf(&mut buf);
            let str_result = core::str::from_utf8(bytes.as_slice());

            match (compact_result, str_result) {
                (Ok(c), Ok(s)) => prop_assert_eq!(c, s),
                (Err(c_err), Err(s_err)) => prop_assert_eq!(c_err, s_err),
                _ => panic!("CompactStr and core::str read UTF-8 differently?"),
            }
        }
    }
}
