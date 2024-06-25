#![cfg_attr(docsrs, doc(cfg(feature = "borsh")))]

use alloc::string::String;
use alloc::vec::Vec;
use core::str;

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

use crate::repr::MAX_SIZE;
use crate::CompactString;

impl BorshSerialize for CompactString {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_str().serialize(writer)
    }
}

impl BorshDeserialize for CompactString {
    fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let len = u32::deserialize_reader(&mut *reader)? as usize;

        if len <= MAX_SIZE {
            let mut buf = [0u8; MAX_SIZE];
            reader.read_exact(&mut buf[..len])?;
            let s = str::from_utf8(&buf[..len])
                .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
            Ok(CompactString::from(s))
        } else {
            let mut buf = Vec::new();
            buf.try_reserve_exact(len)?;
            if reader.take(len as u64).read_to_end(&mut buf)? < len {
                return Err(ErrorKind::UnexpectedEof.into());
            }
            let s =
                String::from_utf8(buf).map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
            Ok(CompactString::from(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use test_strategy::proptest;

    use crate::repr::{
        HEAP_MASK,
        MAX_SIZE,
    };
    use crate::CompactString;

    fn assert_roundtrip(s: &str) {
        let bytes_compact = borsh::to_vec(&CompactString::from(s)).unwrap();
        let bytes_control = borsh::to_vec(&String::from(s)).unwrap();
        assert_eq!(&*bytes_compact, &*bytes_control);

        let compact: CompactString = borsh::from_slice(&bytes_compact).unwrap();
        let control: String = borsh::from_slice(&bytes_control).unwrap();
        assert_eq!(compact, s);
        assert_eq!(control, s);
    }

    #[test]
    fn test_deserialize_invalid_utf8() {
        let bytes = borsh::to_vec(&[HEAP_MASK; MAX_SIZE] as &[u8]).unwrap();
        borsh::from_slice::<CompactString>(&bytes).unwrap_err();
    }

    #[test]
    fn test_deserialize_unexpected_eof() {
        let s = core::str::from_utf8(&[b'a'; 55]).unwrap();
        let mut bytes = borsh::to_vec(s).unwrap();
        bytes.pop();
        borsh::from_slice::<CompactString>(&bytes).unwrap_err();
    }

    #[test]
    fn test_roundtrip() {
        assert_roundtrip("Hello, 🌍!");
    }

    #[cfg_attr(miri, ignore)]
    #[proptest]
    fn proptest_roundtrip(s: String) {
        assert_roundtrip(&s);
    }
}
