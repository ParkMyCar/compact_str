use crate::CompactString;
use alloc::string::String;
use musli::{Allocator, Decode, Decoder, Encode, Encoder};

#[cfg_attr(docsrs, doc(cfg(feature = "musli")))]
impl<M> Encode<M> for CompactString {
    type Encode = str;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        self.as_str().encode(encoder)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self.as_str()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "musli")))]
impl<'de, M, A> Decode<'de, M, A> for CompactString
where
    A: Allocator,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        let s = String::decode(decoder)?;
        Ok(CompactString::from(s))
    }
}

#[cfg(test)]
mod tests {
    use crate::CompactString;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use musli::{Decode, Encode};

    /* ---------------------------------- Storage ------------------------------------------------------------------------ */

    #[test]
    fn test_derive_storage() {
        #[derive(Encode, Decode, Clone, Debug, PartialEq)]
        pub struct TestStruct {
            compact: CompactString,
            compact_large: CompactString,
        }

        let str = TestStruct {
            compact: CompactString::new("this is a string"),
            compact_large: CompactString::new(
                "this is a longer string that exceeds inline capacity",
            ),
        };

        let bytes = musli::storage::to_vec(&str).unwrap();
        let decoded: TestStruct = musli::storage::from_slice(&bytes).unwrap();

        assert_eq!(str, decoded)
    }

    fn roundtrip<T>(value: &T) -> T
    where
        T: Encode<musli::mode::Binary>
            + for<'de> Decode<'de, musli::mode::Binary, musli::alloc::Global>,
    {
        let encoded = musli::packed::to_vec(value).expect("encode failed");
        musli::packed::from_slice(&encoded).expect("decode failed")
    }

    #[test]
    fn test_roundtrip_empty() {
        let s = CompactString::from("");
        let decoded: CompactString = roundtrip(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_roundtrip_short() {
        let s = CompactString::from("hello");
        let decoded: CompactString = roundtrip(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_roundtrip_long() {
        let s = CompactString::from("this is a longer string that exceeds inline capacity");
        let decoded: CompactString = roundtrip(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_interop_with_string() {
        let original = "Ferris the Crab";

        let std_encoded =
            musli::storage::to_vec(&original.to_string()).expect("encode String failed");
        let compact_encoded = musli::storage::to_vec(&CompactString::from(original))
            .expect("encode CompactString failed");

        // Both should produce identical encodings
        assert_eq!(std_encoded, compact_encoded);

        // Decode string-encoded bytes as CompactString
        let decoded_compact: CompactString =
            musli::storage::from_slice(&std_encoded).expect("decode as CompactString failed");
        assert_eq!(decoded_compact, original);

        // Decode compact-encoded bytes as String
        let decoded_string: String =
            musli::storage::from_slice(&compact_encoded).expect("decode as String failed");
        assert_eq!(decoded_string, original);
    }

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    struct PersonCompactString {
        name: CompactString,
        phones: Vec<CompactString>,
    }

    #[test]
    fn test_struct_roundtrip() {
        let person = PersonCompactString {
            name: CompactString::from("Alice"),
            phones: alloc::vec![
                CompactString::from("555-1234"),
                CompactString::from("555-5678"),
            ],
        };

        let encoded = musli::storage::to_vec(&person).expect("encode struct failed");
        let decoded: PersonCompactString =
            musli::storage::from_slice(&encoded).expect("decode struct failed");

        assert_eq!(person, decoded);
    }

    /* ---------------------------------- Packed ------------------------------------------------------------------------ */
    #[test]
    fn test_derive_packed() {
        #[derive(Encode, Decode, Clone, Debug, PartialEq)]
        pub struct TestStruct {
            compact: CompactString,
            compact_large: CompactString,
        }

        let str = TestStruct {
            compact: CompactString::new("this is a string"),
            compact_large: CompactString::new(
                "this is a longer string that exceeds inline capacity",
            ),
        };

        let bytes = musli::packed::to_vec(&str).unwrap();
        let decoded: TestStruct = musli::packed::from_slice(&bytes).unwrap();

        assert_eq!(str, decoded)
    }

    fn roundtrip_packed<T>(value: &T) -> T
    where
        T: Encode<musli::mode::Binary>
            + for<'de> Decode<'de, musli::mode::Binary, musli::alloc::Global>,
    {
        let encoded = musli::packed::to_vec(value).expect("encode failed");
        musli::packed::from_slice(&encoded).expect("decode failed")
    }

    #[test]
    fn test_roundtrip_empty_packed() {
        let s = CompactString::from("");
        let decoded: CompactString = roundtrip_packed(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_roundtrip_short_packed() {
        let s = CompactString::from("hello");
        let decoded: CompactString = roundtrip_packed(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_roundtrip_long_packed() {
        let s = CompactString::from("this is a longer string that exceeds inline capacity");
        let decoded: CompactString = roundtrip_packed(&s);
        assert_eq!(s, decoded);
    }

    /* ---------------------------------- Wire ------------------------------------------------------------------------ */

    #[test]
    fn test_derive_wire() {
        #[derive(Encode, Decode, Clone, Debug, PartialEq)]
        pub struct TestStruct {
            compact: CompactString,
            compact_large: CompactString,
        }

        let str = TestStruct {
            compact: CompactString::new("this is a string"),
            compact_large: CompactString::new(
                "this is a longer string that exceeds inline capacity",
            ),
        };

        let bytes = musli::wire::to_vec(&str).unwrap();
        let decoded: TestStruct = musli::wire::from_slice(&bytes).unwrap();

        assert_eq!(str, decoded)
    }

    fn roundtrip_wire<T>(value: &T) -> T
    where
        T: Encode<musli::mode::Binary>
            + for<'de> Decode<'de, musli::mode::Binary, musli::alloc::Global>,
    {
        let encoded = musli::wire::to_vec(value).expect("encode failed");
        musli::wire::from_slice(&encoded).expect("decode failed")
    }

    #[test]
    fn test_roundtrip_empty_wire() {
        let s = CompactString::from("");
        let decoded: CompactString = roundtrip_wire(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_roundtrip_short_wire() {
        let s = CompactString::from("hello");
        let decoded: CompactString = roundtrip_wire(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_roundtrip_long_wire() {
        let s = CompactString::from("this is a longer string that exceeds inline capacity");
        let decoded: CompactString = roundtrip_wire(&s);
        assert_eq!(s, decoded);
    }

    /* ---------------------------------- Descriptive ------------------------------------------------------------------------ */

    #[test]
    fn test_derive_descriptive() {
        #[derive(Encode, Decode, Clone, Debug, PartialEq)]
        pub struct TestStruct {
            compact: CompactString,
            compact_large: CompactString,
        }

        let str = TestStruct {
            compact: CompactString::new("this is a string"),
            compact_large: CompactString::new(
                "this is a longer string that exceeds inline capacity",
            ),
        };

        let bytes = musli::descriptive::to_vec(&str).unwrap();
        let decoded: TestStruct = musli::descriptive::from_slice(&bytes).unwrap();

        assert_eq!(str, decoded)
    }

    fn roundtrip_descriptive<T>(value: &T) -> T
    where
        T: Encode<musli::mode::Binary>
            + for<'de> Decode<'de, musli::mode::Binary, musli::alloc::Global>,
    {
        let encoded = musli::descriptive::to_vec(value).expect("encode failed");
        musli::descriptive::from_slice(&encoded).expect("decode failed")
    }

    #[test]
    fn test_roundtrip_empty_descriptive() {
        let s = CompactString::from("");
        let decoded: CompactString = roundtrip_descriptive(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_roundtrip_short_descriptive() {
        let s = CompactString::from("hello");
        let decoded: CompactString = roundtrip_descriptive(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_roundtrip_long_descriptive() {
        let s = CompactString::from("this is a longer string that exceeds inline capacity");
        let decoded: CompactString = roundtrip_descriptive(&s);
        assert_eq!(s, decoded);
    }

    /* ---------------------------------- Json ------------------------------------------------------------------------ */

    #[test]
    fn test_derive_json() {
        #[derive(Encode, Decode, Clone, Debug, PartialEq)]
        pub struct TestStruct {
            compact: CompactString,
            compact_large: CompactString,
        }

        let str = TestStruct {
            compact: CompactString::new("this is a string"),
            compact_large: CompactString::new(
                "this is a longer string that exceeds inline capacity",
            ),
        };

        let bytes = musli::json::to_vec(&str).unwrap();
        let decoded: TestStruct = musli::json::from_slice(&bytes).unwrap();

        assert_eq!(str, decoded)
    }

    fn roundtrip_json<T>(value: &T) -> T
    where
        T: Encode<musli::mode::Text>
            + for<'de> Decode<'de, musli::mode::Text, musli::alloc::Global>,
    {
        let encoded = musli::json::to_vec(value).expect("encode failed");
        musli::json::from_slice(&encoded).expect("decode failed")
    }

    #[test]
    fn test_roundtrip_empty_json() {
        let s = CompactString::from("");
        let decoded: CompactString = roundtrip_json(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_roundtrip_short_json() {
        let s = CompactString::from("hello");
        let decoded: CompactString = roundtrip_json(&s);
        assert_eq!(s, decoded);
    }

    #[test]
    fn test_roundtrip_long_json() {
        let s = CompactString::from("this is a longer string that exceeds inline capacity");
        let decoded: CompactString = roundtrip_json(&s);
        assert_eq!(s, decoded);
    }
}
