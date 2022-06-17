use std::collections::VecDeque;
use std::io::Cursor;
use std::num;

use arbitrary::Arbitrary;
use compact_str::{
    CompactString,
    ToCompactString,
};

const MAX_INLINE_LENGTH: usize = std::mem::size_of::<String>();

/// A framework to generate a `CompactString` and control `String`, and then run a series of actions
/// and assert equality
///
/// Used for fuzz testing
#[derive(Arbitrary, Debug)]
pub struct Scenario<'a> {
    pub creation: Creation<'a>,
    pub actions: Vec<Action<'a>>,
}

impl<'a> Scenario<'a> {
    /// Run the provided scenario, asserting for correct behavior
    pub fn run(self) {
        // Given random creation method, if we can create a string
        if let Some((compact, mut control)) = self.creation.create() {
            // assert we never misinterpret a valid CompactString as None when transmuted to
            // Option<...>
            let mut compact = assert_not_option(compact);

            // run some actions, asserting properties along the way
            self.actions
                .into_iter()
                .for_each(|a| a.perform(&mut control, &mut compact));

            // make sure our strings are the same
            assert_eq!(compact, control);

            // after making all of our modifications, assert again that we don't misinterpret
            assert_not_option(compact);
        }
    }
}

#[derive(Arbitrary, Debug)]
pub enum Creation<'a> {
    /// Create using [`CompactString::from_utf8`]
    Bytes(&'a [u8]),
    /// Create using [`CompactString::from_utf8_buf`]
    Buf(&'a [u8]),
    /// Create using an iterator of chars (i.e. the `FromIterator` trait)
    IterChar(Vec<char>),
    /// Create using an iterator of strings (i.e. the `FromIterator` trait)
    IterString(Vec<String>),
    /// Create using [`CompactString::new`]
    Word(String),
    /// Create using [`CompactString::from_utf8_buf`] when the buffer is non-contiguous
    NonContiguousBuf(&'a [u8]),
    /// Create using `From<String>`, which consumes the `String` for `O(1)` runtime
    FromString(String),
    /// Create using `From<Box<str>>`, which consumes the `Box<str>` for `O(1)` runtime
    FromBoxStr(Box<str>),
    /// Create from a type that implements [`ToCompactString`]
    ToCompactString(ToCompactStringArg),
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
        }
    }
}

#[derive(Arbitrary, Debug)]
pub enum Action<'a> {
    Push(char),
    // Note: We use a `u8` to limit the number of pops
    Pop(u8),
    PushStr(&'a str),
    ExtendChars(Vec<char>),
    ExtendStr(Vec<&'a str>),
    CheckSubslice(u8, u8),
}

impl Action<'_> {
    pub fn perform(self, control: &mut String, compact: &mut CompactString) {
        use Action::*;

        match self {
            // push a character
            Push(c) => {
                control.push(c);
                compact.push(c);

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
            // pop `count` number of characters
            Pop(count) => {
                (0..count).for_each(|_| {
                    let a = control.pop();
                    let b = compact.pop();
                    assert_eq!(a, b);
                });
                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
                assert_eq!(control.is_empty(), compact.is_empty());
            }
            // push a `&str`
            PushStr(s) => {
                control.push_str(s);
                compact.push_str(s);

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
            // extend with a Iterator<Item = char>
            ExtendChars(chs) => {
                control.extend(chs.iter());
                compact.extend(chs.iter());

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
            // extend with a Iterator<Item = &str>
            ExtendStr(strs) => {
                control.extend(strs.iter().copied());
                compact.extend(strs.iter().copied());

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
            // check a subslice of bytes is equal
            CheckSubslice(a, b) => {
                assert_eq!(control.len(), compact.len());

                // scale a, b to be [0, 1]
                let c = a as f32 / (u8::MAX as f32);
                let d = b as f32 / (u8::MAX as f32);

                // scale c, b to be [0, compact.len()]
                let e = (c * compact.len() as f32) as usize;
                let f = (d * compact.len() as f32) as usize;

                let lower = core::cmp::min(e, f);
                let upper = core::cmp::max(e, f);

                // scale lower and upper by 1 so we're always comparing at least one character
                let lower = core::cmp::min(lower.wrapping_sub(1), lower);
                let upper = core::cmp::min(upper + 1, compact.len());

                let control_slice = &control.as_bytes()[lower..upper];
                let compact_slice = &compact.as_bytes()[lower..upper];

                assert_eq!(control_slice, compact_slice);
            }
        }
    }
}

/// Asserts the provided CompactString is allocated properly either on the stack or on the heap,
/// using a "control" `&str` for a reference length.
fn assert_properly_allocated(compact: &CompactString, control: &str) {
    assert_eq!(compact.len(), control.len());
    if control.len() <= MAX_INLINE_LENGTH {
        assert!(!compact.is_heap_allocated());
    } else {
        assert!(compact.is_heap_allocated());
    }
}

/// Asserts when the provided [`CompactString`] is `std::mem::transmute`-ed to
/// `Option<CompactString>` that it is never `None`, and when we unwrap the `Option<CompactString>`,
/// it equals the original `CompactString`.
///
/// We use a bit within the discriminant to store whether or not we're "None". We want to make sure
/// valid `CompactString`s never set this bit, and thus get misinterpreted as `None`
fn assert_not_option(compact: CompactString) -> CompactString {
    let clone = compact.clone();

    // transmute the CompactString to Option<CompactString>, they both the same size
    let maybe_compact: Option<CompactString> = unsafe { std::mem::transmute(clone) };

    // make sure we never consider a valid CompactString as None...
    assert!(maybe_compact.is_some());
    // ...and unwrapping the Option gives us the same original value
    assert_eq!(compact, maybe_compact.unwrap());

    compact
}
