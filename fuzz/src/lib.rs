use std::collections::VecDeque;
use std::io::Cursor;

use arbitrary::Arbitrary;
use compact_str::CompactStr;

const MAX_INLINE_LENGTH: usize = std::mem::size_of::<String>();

/// A framework to generate a `CompactStr` and control `String`, and then run a series of actions
/// and assert equality
///
/// Used for fuzz testing
#[derive(Arbitrary, Debug)]
pub struct Scenario<'a> {
    pub creation: Creation<'a>,
    pub actions: Vec<Action<'a>>,
}

#[derive(Arbitrary, Debug)]
pub enum Creation<'a> {
    Bytes(&'a [u8]),
    Buf(&'a [u8]),
    IterChar(Vec<char>),
    IterString(Vec<String>),
    Word(String),
    NonContiguousBuf(&'a [u8]),
}

impl Creation<'_> {
    pub fn create(self) -> Option<(CompactStr, String)> {
        use Creation::*;

        match self {
            // Create a `CompactStr` from a `String`
            Word(word) => {
                let compact = CompactStr::new(&word);

                assert_eq!(compact, word);
                assert_properly_allocated(&compact, &word);

                Some((compact, word))
            }
            // Create a `CompactStr` from an iterator of `char`s
            IterChar(chars) => {
                let compact: CompactStr = chars.iter().collect();
                let std_str: String = chars.iter().collect();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            // Create a `CompactStr` from an iterator of `String`s
            IterString(strings) => {
                let compact: CompactStr = strings.iter().map::<&str, _>(|s| s.as_ref()).collect();
                let std_str: String = strings.iter().map::<&str, _>(|s| s.as_ref()).collect();

                assert_eq!(compact, std_str);
                assert_properly_allocated(&compact, &std_str);

                Some((compact, std_str))
            }
            // Create a `CompactStr` from a slice of bytes
            Bytes(data) => {
                let compact = CompactStr::from_utf8(&data);
                let std_str = std::str::from_utf8(&data);

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
                    _ => panic!("CompactStr and core::str read UTF-8 differently?"),
                }
            },
            // Create a `CompactStr` from a buffer of bytes
            Buf(data) => {
                let mut buffer = Cursor::new(data);

                let compact = CompactStr::from_utf8_buf(&mut buffer);
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
                    _ => panic!("CompactStr and core::str read UTF-8 differently?"),
                }
            }
            // Create a `CompactStr` from a non-contiguous buffer of bytes
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

                // create our CompactStr and control String
                let mut queue_clone = queue.clone();
                let compact = CompactStr::from_utf8_buf(&mut queue);
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
                    _ => panic!("CompactStr and core::str read UTF-8 differently?"),
                }
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
    pub fn perform(self, control: &mut String, compact: &mut CompactStr) {
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
                let c = a as f32 / u8::MAX as f32;
                let d = b as f32 / u8::MAX as f32;

                // scale c, b to be [0, compact.len()]
                let e = (c * compact.len() as f32) as usize;
                let f = (d * compact.len() as f32) as usize;

                let lower = core::cmp::min(e, f);
                let upper = core::cmp::max(e, f);

                let control_slice = &control.as_bytes()[lower..upper];
                let compact_slice = &compact.as_bytes()[lower..upper];

                assert_eq!(control_slice, compact_slice);
            }
        }
    }
}

/// Asserts the provided CompactStr is allocated properly either on the stack or on the heap, using
/// a "control" `&str` for a reference length.
fn assert_properly_allocated(compact: &CompactStr, control: &str) {
    assert_eq!(compact.len(), control.len());
    if control.len() <= MAX_INLINE_LENGTH {
        assert!(!compact.is_heap_allocated());
    } else {
        assert!(compact.is_heap_allocated());
    }
}
