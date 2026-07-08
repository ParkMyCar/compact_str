use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "garde")))]
impl garde::rules::AsStr for CompactString {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "garde")))]
impl garde::rules::length::HasBytes for CompactString {
    fn num_bytes(&self) -> usize {
        self.len()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "garde")))]
impl garde::rules::length::HasChars for CompactString {
    fn num_chars(&self) -> usize {
        self.chars().count()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "garde")))]
impl garde::rules::length::HasSimpleLength for CompactString {
    fn length(&self) -> usize {
        self.len()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "garde")))]
impl garde::rules::length::HasUtf16CodeUnits for CompactString {
    fn num_code_units(&self) -> usize {
        self.encode_utf16().count()
    }
}

#[cfg(test)]
mod tests {
    use garde::Validate;

    use crate::CompactString;

    // Validates that CompactString works as a field in a garde-validated struct
    // using the `ascii` rule (which relies on AsStr).
    #[derive(garde::Validate)]
    struct AsciiTest {
        #[garde(ascii)]
        value: CompactString,
    }

    #[test]
    fn as_str_ascii_valid() {
        let t = AsciiTest {
            value: CompactString::from("hello world!"),
        };
        assert!(t.validate().is_ok());
    }

    #[test]
    fn as_str_ascii_invalid() {
        let t = AsciiTest {
            value: CompactString::from("héllo"),
        };
        assert!(t.validate().is_err());
    }

    // Tests length(bytes, ...) which relies on HasBytes.
    // Multi-byte UTF-8 chars make byte count diverge from char count.
    #[derive(garde::Validate)]
    struct ByteLengthTest {
        #[garde(length(bytes, min = 1, max = 4))]
        value: CompactString,
    }

    #[test]
    fn has_bytes_within_limit() {
        // "hi" = 2 bytes
        let t = ByteLengthTest {
            value: CompactString::from("hi"),
        };
        assert!(t.validate().is_ok());
    }

    #[test]
    fn has_bytes_exceeds_limit() {
        // "héllo" = 6 bytes (é is 2 bytes) — exceeds max=4
        let t = ByteLengthTest {
            value: CompactString::from("héllo"),
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn has_bytes_empty_fails() {
        let t = ByteLengthTest {
            value: CompactString::new(""),
        };
        assert!(t.validate().is_err());
    }

    // Tests length(chars, ...) which relies on HasChars.
    // A multi-byte char is still one char.
    #[derive(garde::Validate)]
    struct CharLengthTest {
        #[garde(length(chars, min = 1, max = 3))]
        value: CompactString,
    }

    #[test]
    fn has_chars_within_limit() {
        // "héy" = 3 chars (even though é is 2 bytes)
        let t = CharLengthTest {
            value: CompactString::from("héy"),
        };
        assert!(t.validate().is_ok());
    }

    #[test]
    fn has_chars_exceeds_limit() {
        let t = CharLengthTest {
            value: CompactString::from("abcd"),
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn has_chars_empty_fails() {
        let t = CharLengthTest {
            value: CompactString::new(""),
        };
        assert!(t.validate().is_err());
    }

    // Tests length(...) with no mode, which uses HasSimpleLength (byte-based).
    #[derive(garde::Validate)]
    struct SimpleLengthTest {
        #[garde(length(min = 2, max = 5))]
        value: CompactString,
    }

    #[test]
    fn has_simple_length_valid() {
        let t = SimpleLengthTest {
            value: CompactString::from("abc"),
        };
        assert!(t.validate().is_ok());
    }

    #[test]
    fn has_simple_length_too_short() {
        let t = SimpleLengthTest {
            value: CompactString::from("a"),
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn has_simple_length_too_long() {
        let t = SimpleLengthTest {
            value: CompactString::from("abcdef"),
        };
        assert!(t.validate().is_err());
    }

    // Tests length(utf16, ...) which relies on HasUtf16CodeUnits.
    // Emoji outside the BMP require 2 UTF-16 code units (surrogate pair).
    #[derive(garde::Validate)]
    struct Utf16LengthTest {
        #[garde(length(utf16, min = 1, max = 2))]
        value: CompactString,
    }

    #[test]
    fn has_utf16_bmp_char_valid() {
        // "é" = 1 UTF-16 code unit
        let t = Utf16LengthTest {
            value: CompactString::from("é"),
        };
        assert!(t.validate().is_ok());
    }

    #[test]
    fn has_utf16_surrogate_pair_valid() {
        // "😀" = 2 UTF-16 code units
        let t = Utf16LengthTest {
            value: CompactString::from("😀"),
        };
        assert!(t.validate().is_ok());
    }

    #[test]
    fn has_utf16_exceeds_limit() {
        // "😀😀" = 4 UTF-16 code units — exceeds max=2
        let t = Utf16LengthTest {
            value: CompactString::from("😀😀"),
        };
        assert!(t.validate().is_err());
    }

    // Ensure CompactString and String produce identical results across all traits.
    #[derive(garde::Validate)]
    struct StringTest {
        #[garde(ascii)]
        #[garde(length(bytes, min = 1, max = 100))]
        #[garde(length(chars, min = 1, max = 100))]
        #[garde(length(min = 1, max = 100))]
        #[garde(length(utf16, min = 1, max = 100))]
        value: std::string::String,
    }

    #[derive(garde::Validate)]
    struct CompactTest {
        #[garde(ascii)]
        #[garde(length(bytes, min = 1, max = 100))]
        #[garde(length(chars, min = 1, max = 100))]
        #[garde(length(min = 1, max = 100))]
        #[garde(length(utf16, min = 1, max = 100))]
        value: CompactString,
    }

    #[test]
    fn matches_string_behavior() {
        for s in &["hello", "héllo", "😀", "a", ""] {
            let string_result = StringTest {
                value: std::string::String::from(*s),
            }
            .validate();
            let compact_result = CompactTest {
                value: CompactString::from(*s),
            }
            .validate();
            assert_eq!(
                string_result.is_ok(),
                compact_result.is_ok(),
                "mismatch for input {:?}",
                s
            );
        }
    }
}
