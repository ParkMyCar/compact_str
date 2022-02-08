use core::str::Utf8Error;

use bytes::Buf;

use super::{
    Repr,
    MAX_SIZE,
};

#[cfg(target_pointer_width = "32")]
const DEFAULT_TEXT: &str = "000000000000";
#[cfg(target_pointer_width = "64")]
const DEFAULT_TEXT: &str = "000000000000000000000000";

const DEFAULT_PACKED: Repr = Repr::new_const(DEFAULT_TEXT);

impl Repr {
    /// Converts a buffer of bytes to a `Repr`,
    pub fn from_utf8_buf<B: Buf>(buf: &mut B) -> Result<Self, Utf8Error> {
        // SAFETY: We check below to make sure the provided buffer is valid UTF-8
        let (repr, bytes_written) = unsafe { Self::from_buf(buf) };

        // Check to make sure the provided bytes are valid UTF-8, return the Repr if they are!
        match core::str::from_utf8(&repr.as_slice()[..bytes_written]) {
            Ok(_) => Ok(repr),
            Err(e) => Err(e),
        }
    }

    /// Converts a buffer of bytes to a `Repr`, without checking for valid UTF-8
    ///
    /// # Safety
    /// The provided buffer must be valid UTF-8
    pub unsafe fn from_utf8_buf_unchecked<B: Buf>(buf: &mut B) -> Self {
        let (repr, _bytes_written) = Self::from_buf(buf);
        repr
    }

    unsafe fn from_buf<B: Buf>(buf: &mut B) -> (Self, usize) {
        // Get an empty Repr we can write into
        let mut repr = super::EMPTY;
        let mut bytes_written = 0;
        debug_assert_eq!(repr.len(), bytes_written);

        while buf.has_remaining() {
            let chunk = buf.chunk();
            let chunk_len = chunk.len();

            // reserve at least enough space to fit this chunk
            repr.reserve(chunk_len);

            // SAFETY: The caller is responsible for making sure the provided buffer is UTF-8. This
            // invariant is documented in the public API
            let slice = repr.as_mut_slice();
            // write the chunk into the Repr
            buf.copy_to_slice(&mut slice[bytes_written..bytes_written + chunk_len]);

            // Set the length of the Repr
            // SAFETY: We just wrote an additional `chunk_len` bytes into the Repr
            bytes_written += chunk_len;
            repr.set_len(bytes_written);
        }

        (repr, bytes_written)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::Repr;

    #[test]
    fn test_smoke() {
        let word = "hello world";
        let mut buf = Cursor::new(word.as_bytes());

        let repr = Repr::from_utf8_buf(&mut buf).unwrap();
        assert_eq!(repr.as_str(), word);
    }

    #[test]
    fn test_heap_allocated() {
        let word = "hello, this is a long string which should be heap allocated";
        let mut buf = Cursor::new(word.as_bytes());

        let repr = Repr::from_utf8_buf(&mut buf).unwrap();
        assert_eq!(repr.as_str(), word);
    }

    #[test]
    fn test_empty() {
        let mut buf: Cursor<&[u8]> = Cursor::new(&[]);

        let repr = Repr::from_utf8_buf(&mut buf).unwrap();
        assert_eq!(repr.len(), 0);
        assert_eq!(repr.as_str(), "");
    }

    #[test]
    fn test_packed() {
        #[cfg(target_pointer_width = "64")]
        let packed = "this string is 24 chars!";
        #[cfg(target_pointer_width = "32")]
        let packed = "i am 12 char";

        let mut buf = Cursor::new(packed.as_bytes());

        let repr = Repr::from_utf8_buf(&mut buf).unwrap();
        assert_eq!(repr.as_str(), packed);

        // This repr should __not__ be heap allocated
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    fn test_fuzz_panic() {
        let bytes = &[
            255, 255, 255, 255, 255, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 12, 0, 0, 96,
        ];
        let mut buf: Cursor<&[u8]> = Cursor::new(bytes);

        assert!(Repr::from_utf8_buf(&mut buf).is_err());
    }

    #[test]
    fn test_valid_repr_but_invalid_utf8() {
        let bytes = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192];
        let mut buf: Cursor<&[u8]> = Cursor::new(bytes);

        assert!(Repr::from_utf8_buf(&mut buf).is_err());
    }

    #[test]
    fn test_fake_heap_variant() {
        let bytes = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255];
        let mut buf: Cursor<&[u8]> = Cursor::new(bytes);

        assert!(Repr::from_utf8_buf(&mut buf).is_err());
    }

    #[test]
    #[should_panic(expected = "Utf8Error")]
    fn test_invalid_utf8() {
        let invalid = &[0, 159];
        let mut buf: Cursor<&[u8]> = Cursor::new(invalid);

        Repr::from_utf8_buf(&mut buf).unwrap();
    }
}
