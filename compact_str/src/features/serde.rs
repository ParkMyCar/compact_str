use std::fmt;

use serde::de::{
    Deserializer,
    Error,
    Unexpected,
    Visitor,
};

use crate::CompactString;

fn compact_string<'de: 'a, 'a, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<CompactString, D::Error> {
    struct CompactStringVisitor;

    impl<'a> Visitor<'a> for CompactStringVisitor {
        type Value = CompactString;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(CompactString::from(v))
        }

        fn visit_borrowed_str<E: Error>(self, v: &'a str) -> Result<Self::Value, E> {
            Ok(CompactString::from(v))
        }

        fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(CompactString::from(v))
        }

        fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
            match std::str::from_utf8(v) {
                Ok(s) => Ok(CompactString::from(s)),
                Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
            }
        }

        fn visit_borrowed_bytes<E: Error>(self, v: &'a [u8]) -> Result<Self::Value, E> {
            match std::str::from_utf8(v) {
                Ok(s) => Ok(CompactString::from(s)),
                Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
            }
        }

        fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
            match String::from_utf8(v) {
                Ok(s) => Ok(CompactString::from(s)),
                Err(e) => Err(Error::invalid_value(
                    Unexpected::Bytes(&e.into_bytes()),
                    &self,
                )),
            }
        }
    }

    deserializer.deserialize_str(CompactStringVisitor)
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl serde::Serialize for CompactString {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> serde::Deserialize<'de> for CompactString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        compact_string(deserializer)
    }
}
