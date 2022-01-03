use core::str::Utf8Error;

use bytes::Buf;

use super::{
    Repr,
    MAX_SIZE,
};

#[cfg(target_pointer_width = "32")]
const DEFAULT_TEXT: str = "000000000000";
#[cfg(target_pointer_width = "64")]
const DEFAULT_TEXT: &str = "000000000000000000000000";

const DEFAULT_PACKED: Repr = Repr::new_const(DEFAULT_TEXT);

impl Repr {
    /// Converts a buffer of bytes to a `Repr`
    pub fn from_utf8_buf<B: Buf>(buf: &mut B) -> Result<Self, Utf8Error> {
        let size = buf.remaining();
        let chunk = buf.chunk();

        // Check to make sure we're not empty, so accessing the first byte below doesn't panic
        if chunk.is_empty() {
            // If the chunk is empty, then we should have 0 remaining bytes
            debug_assert_eq!(size, 0);
            return Ok(super::EMPTY);
        }
        let first_byte = buf.chunk()[0];

        // Get an "empty" Repr we can write into
        //
        // HACK: There currently isn't a way to provide an "empty" Packed repr, so we do this check
        // and return a "default" Packed repr if the buffer can fit
        let mut repr = if size == MAX_SIZE && first_byte <= 127 {
            // Note: No need to reserve additional bytes here, because we know we can fit all
            // remaining bytes of `buf` into a Packed repr
            DEFAULT_PACKED
        } else {
            let mut default = super::EMPTY;
            debug_assert_eq!(default.len(), 0);

            // Reserve enough bytes, possibly allocating on the heap, to store the text
            default.reserve(size);

            default
        };

        // SAFETY: Before returning this Repr we check to make sure the provided bytes are valid
        // UTF-8
        let slice = unsafe { repr.as_mut_slice() };
        // Copy the bytes from the buffer into our Repr!
        buf.copy_to_slice(&mut slice[..size]);

        // Set the length of the Repr
        // SAFETY: We just wrote `size` bytes into the Repr
        unsafe { repr.set_len(size) };

        // Check to make sure the provided bytes are valid UTF-8, return the Repr if they are!
        //
        // TODO: Add an `as_slice()` method to Repr and refactor this call
        match core::str::from_utf8(repr.as_str().as_bytes()) {
            Ok(_) => Ok(repr),
            Err(e) => Err(e),
        }
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
    #[should_panic(expected = "Utf8Error")]
    fn test_invalid_utf8() {
        let invalid = &[0, 159];
        let mut buf: Cursor<&[u8]> = Cursor::new(invalid);

        Repr::from_utf8_buf(&mut buf).unwrap();
    }
}
