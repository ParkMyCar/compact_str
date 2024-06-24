#![cfg_attr(docsrs, doc(cfg(feature = "borsh")))]

use borsh::io::{
    Error,
    ErrorKind,
    Read,
    Result,
    Write,
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use crate::CompactString;

impl BorshSerialize for CompactString {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_str().serialize(writer)
    }
}

impl BorshDeserialize for CompactString {
    fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let len = u32::deserialize_reader(&mut *reader)? as usize;

        // Do not call `Error::new` on OOM as it allocates.
        let mut s = CompactString::try_with_capacity(len).map_err(|_| ErrorKind::OutOfMemory)?;

        // SAFETY: The current length is zero so the bytes are uninterpreted
        // and don't have to form valid UTF-8
        let buf = unsafe { s.as_mut_bytes() };

        reader.read_exact(&mut buf[..len])?;
        core::str::from_utf8(&buf[..len]).map_err(|err| Error::new(ErrorKind::InvalidData, err))?;

        // SAFETY: The first `len` bytes are initialized and are valid UTF-8
        unsafe { s.set_len(len) };

        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use test_strategy::proptest;

    use crate::CompactString;

    #[test]
    fn test_roundtrip() {
        const VALUE: &str = "Hello, 🌍!";

        let bytes_compact = borsh::to_vec(&CompactString::from(VALUE)).unwrap();
        let bytes_control = borsh::to_vec(&String::from(VALUE)).unwrap();
        assert_eq!(&*bytes_compact, &*bytes_control);

        let compact: CompactString = borsh::from_slice(&bytes_compact).unwrap();
        let control: String = borsh::from_slice(&bytes_control).unwrap();
        assert_eq!(compact, VALUE);
        assert_eq!(control, VALUE);
    }

    #[cfg_attr(miri, ignore)]
    #[proptest]
    fn proptest_roundtrip(s: String) {
        let bytes_compact = borsh::to_vec(&CompactString::from(&s)).unwrap();
        let bytes_control = borsh::to_vec(&String::from(&s)).unwrap();
        assert_eq!(&*bytes_compact, &*bytes_control);

        let compact: CompactString = borsh::from_slice(&bytes_compact).unwrap();
        let control: String = borsh::from_slice(&bytes_control).unwrap();
        assert_eq!(compact, s);
        assert_eq!(control, s);
    }
}
