use super::SmartStr;
use serde::de::{Deserializer, Error, Unexpected, Visitor};
use std::fmt;

fn smart_str<'de: 'a, 'a, D: Deserializer<'de>>(deserializer: D) -> Result<SmartStr, D::Error> {
    struct SmartStrVisitor;

    impl<'a> Visitor<'a> for SmartStrVisitor {
        type Value = SmartStr;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(SmartStr::from(v))
        }

        fn visit_borrowed_str<E: Error>(self, v: &'a str) -> Result<Self::Value, E> {
            Ok(SmartStr::from(v))
        }

        fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(SmartStr::from(v))
        }

        fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
            match std::str::from_utf8(v) {
                Ok(s) => Ok(SmartStr::from(s)),
                Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self))
            }
        }

        fn visit_borrowed_bytes<E: Error>(self, v: &'a [u8]) -> Result<Self::Value, E> {
            match std::str::from_utf8(v) {
                Ok(s) => Ok(SmartStr::from(s)),
                Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self))
            }
        }

        fn visit_byte_buf<E:  Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
            match String::from_utf8(v) {
                Ok(s) => Ok(SmartStr::from(s)),
                Err(e) => Err(Error::invalid_value(Unexpected::Bytes(&e.into_bytes()), &self))
            }
        }
    }

    deserializer.deserialize_str(SmartStrVisitor)
}

impl serde::Serialize for SmartStr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for SmartStr {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        smart_str(deserializer)
    }
}
