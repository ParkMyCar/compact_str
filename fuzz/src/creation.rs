//! Different ways in which we can create a [`CompactString`] and a "control" [`String`]. If we can
//! successfully generate this pair, we then run varios actions on them, which are defined in the
//! [`super::actions`] module.

use std::borrow::Cow;
use std::collections::VecDeque;
use std::io::Cursor;
use std::num;
use std::str::FromStr;

use arbitrary::Arbitrary;
use compact_str::{
    CompactString,
    CompactStringExt,
    ToCompactString,
};

use super::assert_properly_allocated;
use crate::MAX_INLINE_LENGTH;

#[derive(Arbitrary, Debug)]
pub enum Creation<'a> {
    /// Create using [`CompactString::from_utf8`]
    Bytes(&'a [u8]),
    /// Create using [`CompactString::from_utf16`]
    BytesUtf16(Vec<u16>),
    /// Create using [`CompactString::from_utf8_buf`]
    Buf(&'a [u8]),
    /// Create using [`CompactString::from_utf8_buf_unchecked`]
    BufUnchecked(&'a [u8]),
    /// Create using an iterator of chars (i.e. the `FromIterator` trait)
    IterChar(Vec<char>),
    /// Create using an iterator of strings (i.e. the `FromIterator` trait)
    IterString(Vec<String>),
    /// Create using [`CompactString::new`]
    Word(String),
    /// Encode the provided `String` as UTF-16, and the create using [`CompactString::from_utf16`]
    WordUtf16(String),
    /// Create using [`CompactString::from_utf8_buf`] when the buffer is non-contiguous
    NonContiguousBuf(&'a [u8]),
    /// Create using `From<&str>`, which copies the data from the string slice
    FromStr(&'a str),
    /// Create using `FromStr` trait
    FromStrTrait(&'a str),
    /// Create using `From<String>`, which consumes the `String` for `O(1)` runtime
    FromString(String),
    /// Create using `From<Box<str>>`, which consumes the `Box<str>` for `O(1)` runtime
    FromBoxStr(Box<str>),
    /// Create using `From<Cow<'a, str>>`, which possibly consumes an owned string in `O(1)`
    FromCowStr(CowStrArg<'a>),
    /// Create from a type that implements [`ToCompactString`]
    ToCompactString(ToCompactStringArg),
    /// Create by joining a collection of strings with seperator, using [`CompactStringExt`]
    Join(Vec<&'a str>, &'a str),
    /// Create by concatenating a collection of strings, using [`CompactStringExt`]
    Concat(Vec<&'a str>),
    /// Create using [`CompactString::with_capacity`], note: the max size we create is 24MB
    WithCapacity(u32),
    /// Create by `.collect()`-ing chars
    CollectChar(Vec<char>),
    /// Create by `.collect()`-ing a collection of Strings
    CollectString(Vec<String>),
    /// Create by `.collect()`-ing a collection of Box<str>
    CollectBoxStr(Vec<Box<str>>),
    /// Create using [`std::default::Default`]
    Default,
}

/// Types that we're able to convert to a [`CompactString`]
///
/// Note: number types, bool, and char all have a special implementation for performance
#[derive(Arbitrary, Debug)]
pub enum ToCompactStringArg {
    /// Create from a number type using [`ToCompactString`]
    Num(NumType),
    /// Create from a non-zero number type using [`ToCompactString`]
    NonZeroNum(NonZeroNumType),
    /// Create from a `bool` using [`ToCompactString`]
    Bool(bool),
    /// Create from a `char` using [`ToCompactString`]
    Char(char),
    /// Create  from a string using [`ToCompactString`]
    String(String),
}

#[derive(Arbitrary, Debug)]
pub enum NumType {
    /// Create from an `u8`
    U8(u8),
    /// Create from an `i8`
    I8(i8),
    /// Create from an `u16`
    U16(u16),
    /// Create from an `i16`
    I16(i16),
    /// Create from an `u32`
    U32(u32),
    /// Create from an `i32`
    I32(i32),
    /// Create from an `u64`
    U64(u64),
    /// Create from an `i64`
    I64(i64),
    /// Create from an `u128`
    U128(u128),
    /// Create from an `i128`
    I128(i128),
    /// Create from an `usize`
    Usize(usize),
    /// Create from an `isize`
    Isize(isize),
    /// Create from an `f32`,
    F32(f32),
    /// Create from an `f64`
    F64(f64),
}

#[derive(Arbitrary, Debug)]
pub enum NonZeroNumType {
    /// Create from a `NonZeroU8`
    U8(num::NonZeroU8),
    /// Create from a `NonZeroI8`
    I8(num::NonZeroI8),
    /// Create from a `NonZeroU16`
    U16(num::NonZeroU16),
    /// Create from a `NonZeroI16`
    I16(num::NonZeroI16),
    /// Create from a `NonZeroU32`
    U32(num::NonZeroU32),
    /// Create from a `NonZeroI32`
    I32(num::NonZeroI32),
    /// Create from a `NonZeroU64`
    U64(num::NonZeroU64),
    /// Create from a `NonZeroI64`
    I64(num::NonZeroI64),
    /// Create from a `NonZeroU128`
    U128(num::NonZeroU128),
    /// Create from a `NonZeroI128`
    I128(num::NonZeroI128),
    /// Create from a `NonZeroUsize`
    Usize(num::NonZeroUsize),
    /// Create from a `NonZeroIsize`
    Isize(num::NonZeroIsize),
}

#[derive(Arbitrary, Clone, Debug)]
pub enum CowStrArg<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl Creation<'_> {
    pub fn create(self) -> Option<(CompactString, String)> {
        use Creation::*;

        match self {
            Word(word) => {
                let compact = CompactString::new(&word);

                assert_eq!(compact, word);
                assert_properly_allocated(&compact, &word);

                Some((compact, word))
            }
            WordUtf16(word) => {
                let utf16_buf: Vec<u16> = word.encode_utf16().collect();
                let compact =
                    CompactString::from_utf16(utf16_buf).expect("UTF-16 failed to roundtrip!");

                assert_eq!(compact, word);
                assert_properly_allocated(&compact, &word);

                Some((compact, word))
            }
            FromStr(s) => {
                let compact = CompactString::from(s);
                let std_str = s.to_string();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            FromStrTrait(s) => {
                let compact = CompactString::from_str(s).expect("FromStr was fallible!");
                let std_str = s.to_string();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            FromString(s) => {
                let compact = CompactString::from(s.clone());

                assert_eq!(compact, s);

                // Note: converting From<String> will always be heap allocated because we use the
                // underlying buffer from the source String
                if s.capacity() == 0 {
                    assert!(!compact.is_heap_allocated());
                } else {
                    assert!(compact.is_heap_allocated());
                }

                Some((compact, s))
            }
            FromBoxStr(b) => {
                let compact = CompactString::from(b.clone());

                assert_eq!(compact, b);

                // Note: converting From<Box<str>> will always be heap allocated because we use the
                // underlying buffer from the source String
                if b.len() == 0 {
                    assert!(!compact.is_heap_allocated())
                } else {
                    assert!(compact.is_heap_allocated())
                }

                let string = String::from(b);
                Some((compact, string))
            }
            FromCowStr(cow_arg) => {
                let (cow, std_str) = match cow_arg {
                    CowStrArg::Borrowed(borrow) => {
                        let cow = Cow::Borrowed(borrow);
                        let std_str = borrow.to_string();

                        (cow, std_str)
                    }
                    CowStrArg::Owned(owned) => {
                        let std_str = owned.clone();
                        let cow = Cow::Owned(owned);

                        (cow, std_str)
                    }
                };

                let compact = CompactString::from(cow);
                assert_eq!(compact, std_str);

                // Note: we don't assert properly allocated here because we might do an O(1)
                // conversion from String, if the Cow is owned, and thus could end up with a small
                // string on the heap

                Some((compact, std_str))
            }
            IterChar(chars) => {
                let compact: CompactString = chars.iter().collect();
                let std_str: String = chars.iter().collect();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            IterString(strings) => {
                let compact: CompactString =
                    strings.iter().map::<&str, _>(|s| s.as_ref()).collect();
                let std_str: String = strings.iter().map::<&str, _>(|s| s.as_ref()).collect();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            Bytes(data) => {
                let compact = CompactString::from_utf8(data);
                let std_str = std::str::from_utf8(data);

                match (compact, std_str) {
                    // valid UTF-8
                    (Ok(c), Ok(s)) => {
                        assert_eq!(c, s);
                        assert_properly_allocated(&c, s);

                        Some((c, s.to_string()))
                    }
                    // non-valid UTF-8
                    (Err(c_err), Err(s_err)) => {
                        assert_eq!(c_err, s_err);
                        None
                    }
                    _ => panic!("CompactString and core::str read UTF-8 differently?"),
                }
            }
            BytesUtf16(data) => {
                let compact = CompactString::from_utf16(&data);
                let std_str = String::from_utf16(&data);

                match (compact, std_str) {
                    // valid UTF-16
                    (Ok(c), Ok(s)) => {
                        assert_eq!(c, s);
                        assert_properly_allocated(&c, &s);

                        Some((c, s))
                    }
                    // non-valid UTF-16
                    (Err(_), Err(_)) => None,
                    _ => panic!("CompactString and String read UTF-16 differently?"),
                }
            }
            Buf(data) => {
                let mut buffer = Cursor::new(data);

                let compact = CompactString::from_utf8_buf(&mut buffer);
                let std_str = std::str::from_utf8(data);

                match (compact, std_str) {
                    // valid UTF-8
                    (Ok(c), Ok(s)) => {
                        assert_eq!(c, s);
                        assert_properly_allocated(&c, s);

                        Some((c, s.to_string()))
                    }
                    // non-valid UTF-8
                    (Err(c_err), Err(s_err)) => {
                        assert_eq!(c_err, s_err);
                        None
                    }
                    _ => panic!("CompactString and core::str read UTF-8 differently?"),
                }
            }
            BufUnchecked(data) => {
                let mut buffer = Cursor::new(data);

                // The data provided might not be valid UTF-8. We mainly want to make sure we don't
                // panic, and the data is written correctly. Before returning either of these types
                // we'll make sure they contain valid data
                let compact = unsafe { CompactString::from_utf8_buf_unchecked(&mut buffer) };
                let std_str = unsafe { String::from_utf8_unchecked(data.to_vec()) };

                // make sure our data didn't somehow get longer
                assert_eq!(data.len(), compact.len());
                assert_eq!(compact.len(), std_str.len());

                // make sure the data written is the same
                assert_eq!(compact.as_bytes(), std_str.as_bytes());

                let data_is_valid = std::str::from_utf8(data);
                let compact_is_valid = std::str::from_utf8(compact.as_bytes());
                let std_str_is_valid = std::str::from_utf8(std_str.as_bytes());

                match (data_is_valid, compact_is_valid, std_str_is_valid) {
                    (Ok(d), Ok(c), Ok(s)) => {
                        // if we get &str's back, make sure they're all equal
                        assert_eq!(d, c);
                        assert_eq!(c, s);

                        // we have valid UTF-8 data! we can return a pair
                        Some((compact, std_str))
                    }
                    (Err(d), Err(c), Err(s)) => {
                        // if we get errors back, the errors should be the same
                        assert_eq!(d, c);
                        assert_eq!(c, s);

                        // we don't have valid UTF-8 data, so we can't return anything
                        None
                    }
                    _ => panic!("data, CompactString, and String disagreed?"),
                }
            }
            NonContiguousBuf(data) => {
                let mut queue = if data.len() > 3 {
                    // if our data is long, make it non-contiguous
                    let (front, back) = data.split_at(data.len() / 2 + 1);
                    let mut queue = VecDeque::with_capacity(data.len());

                    // create a non-contiguous slice of memory in queue
                    front.iter().copied().for_each(|x| queue.push_back(x));
                    back.iter().copied().for_each(|x| queue.push_front(x));

                    // make sure it's non-contiguous
                    let (a, b) = queue.as_slices();
                    assert!(data.is_empty() || !a.is_empty());
                    assert!(data.is_empty() || !b.is_empty());

                    queue
                } else {
                    data.iter().copied().collect::<VecDeque<u8>>()
                };

                // create our CompactString and control String
                let mut queue_clone = queue.clone();
                let compact = CompactString::from_utf8_buf(&mut queue);
                let std_str = std::str::from_utf8(queue_clone.make_contiguous());

                match (compact, std_str) {
                    // valid UTF-8
                    (Ok(c), Ok(s)) => {
                        assert_eq!(c, s);
                        assert_properly_allocated(&c, s);
                        Some((c, s.to_string()))
                    }
                    // non-valid UTF-8
                    (Err(c_err), Err(s_err)) => {
                        assert_eq!(c_err, s_err);
                        None
                    }
                    _ => panic!("CompactString and core::str read UTF-8 differently?"),
                }
            }
            ToCompactString(arg) => {
                let (compact, word) = match arg {
                    ToCompactStringArg::Num(num_type) => match num_type {
                        NumType::U8(val) => (val.to_compact_string(), val.to_string()),
                        NumType::I8(val) => (val.to_compact_string(), val.to_string()),
                        NumType::U16(val) => (val.to_compact_string(), val.to_string()),
                        NumType::I16(val) => (val.to_compact_string(), val.to_string()),
                        NumType::U32(val) => (val.to_compact_string(), val.to_string()),
                        NumType::I32(val) => (val.to_compact_string(), val.to_string()),
                        NumType::U64(val) => (val.to_compact_string(), val.to_string()),
                        NumType::I64(val) => (val.to_compact_string(), val.to_string()),
                        NumType::U128(val) => (val.to_compact_string(), val.to_string()),
                        NumType::I128(val) => (val.to_compact_string(), val.to_string()),
                        NumType::Usize(val) => (val.to_compact_string(), val.to_string()),
                        NumType::Isize(val) => (val.to_compact_string(), val.to_string()),
                        // Note: The formatting of floats by `ryu` sometimes differs from that of
                        // `std`, so instead of asserting equality with `std` we just make sure the
                        // value roundtrips
                        NumType::F32(val) => {
                            let compact = val.to_compact_string();
                            let roundtrip = compact.parse::<f32>().unwrap();

                            if val.is_nan() {
                                assert!(roundtrip.is_nan())
                            } else {
                                assert_eq!(val, roundtrip);
                            }

                            return None;
                        }
                        NumType::F64(val) => {
                            let compact = val.to_compact_string();
                            let roundtrip = compact.parse::<f64>().unwrap();

                            if val.is_nan() {
                                assert!(roundtrip.is_nan())
                            } else {
                                assert_eq!(val, roundtrip);
                            }

                            return None;
                        }
                    },
                    ToCompactStringArg::NonZeroNum(non_zero_type) => match non_zero_type {
                        NonZeroNumType::U8(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::I8(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::U16(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::I16(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::U32(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::I32(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::U64(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::I64(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::U128(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::I128(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::Usize(val) => (val.to_compact_string(), val.to_string()),
                        NonZeroNumType::Isize(val) => (val.to_compact_string(), val.to_string()),
                    },
                    ToCompactStringArg::Bool(bool) => (bool.to_compact_string(), bool.to_string()),
                    ToCompactStringArg::Char(c) => (c.to_compact_string(), c.to_string()),
                    ToCompactStringArg::String(word) => (word.to_compact_string(), word),
                };

                assert_eq!(compact, word);
                assert_properly_allocated(&compact, &word);

                Some((compact, word))
            }
            Join(collection, seperator) => {
                let compact: CompactString = collection.join_compact(seperator);
                let std_str: String = collection.join(seperator);

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            Concat(collection) => {
                let compact: CompactString = collection.concat_compact();
                let std_str: String = collection.concat();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            WithCapacity(val) => {
                // pick some value between [0, 24MB]
                let ratio: f32 = (val as f32) / (u32::MAX as f32);
                let num_bytes = ((super::TWENTY_FOUR_MB_AS_BYTES as f32) * ratio) as u32;

                let compact = CompactString::with_capacity(num_bytes as usize);
                let std_str = String::with_capacity(num_bytes as usize);

                if compact.is_heap_allocated() {
                    // if we're heap allocated, then we should have the same capacity
                    assert_eq!(compact.capacity(), std_str.capacity());
                } else {
                    // if we're inline then a CompactString will have capacity MAX_INLINE_LENGTH
                    assert!(num_bytes as usize <= super::MAX_INLINE_LENGTH);
                    assert_eq!(compact.capacity(), MAX_INLINE_LENGTH);
                }

                // they both should be empty
                assert_eq!(compact, "");
                assert_eq!(compact, std_str);

                Some((compact, std_str))
            }
            CollectChar(chars) => {
                let compact: CompactString = chars.clone().into_iter().collect();
                let std_str: String = chars.into_iter().collect();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            CollectString(strings) => {
                let compact: CompactString = strings.clone().into_iter().collect();
                let std_str: String = strings.into_iter().collect();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            CollectBoxStr(strings) => {
                let compact: CompactString = strings.clone().into_iter().collect();
                let std_str: String = strings.into_iter().collect();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            Default => {
                let compact = CompactString::default();
                let std_str = String::default();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
        }
    }
}
