//! Various actions we take on a [`CompactString`] and "control" [`String`], asserting invariants
//! along the way.

use arbitrary::Arbitrary;
use compact_str::CompactString;

#[derive(Arbitrary, Debug)]
pub enum Action<'a> {
    /// Push a character onto our strings
    Push(char),
    /// Pop a number of characters off the string
    Pop(u8),
    /// Push a &str onto our strings
    PushStr(&'a str),
    /// Extend our strings with a collection of characters
    ExtendChars(Vec<char>),
    /// Extend our strings with a collection of strings
    ExtendStr(Vec<&'a str>),
    /// Check to make sure a subslice of our strings are the same
    CheckSubslice(u8, u8),
    /// Make both of our strings uppercase
    MakeUppercase,
    /// Replace a range within both strings with the provided `&str`
    ReplaceRange(u8, u8, &'a str),
    /// Reserve space in our string, no-ops if the `CompactString` would have a capacity > 24MB
    Reserve(u16),
    /// Truncate a string to a new, shorter length
    Truncate(u8),
}

impl Action<'_> {
    pub fn perform(self, control: &mut String, compact: &mut CompactString) {
        use Action::*;

        match self {
            Push(c) => {
                control.push(c);
                compact.push(c);

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
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
            PushStr(s) => {
                control.push_str(s);
                compact.push_str(s);

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
            ExtendChars(chs) => {
                control.extend(chs.iter());
                compact.extend(chs.iter());

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
            ExtendStr(strs) => {
                control.extend(strs.iter().copied());
                compact.extend(strs.iter().copied());

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
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
            MakeUppercase => {
                control.as_mut_str().make_ascii_uppercase();
                compact.as_mut_str().make_ascii_uppercase();

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
            ReplaceRange(start, end, replace_with) => {
                // turn the arbitrary numbers (start, end) into character indices
                let start = control
                    .char_indices()
                    .into_iter()
                    .cycle()
                    .nth(start as usize)
                    .unwrap_or_default()
                    .0;
                let end = control
                    .char_indices()
                    .into_iter()
                    .cycle()
                    .nth(end as usize)
                    .unwrap_or_default()
                    .0;
                let (start, end) = (start.min(end), start.max(end));

                // then apply the replacement
                control.replace_range(start..end, replace_with);
                compact.replace_range(start..end, replace_with);

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
            Reserve(num_bytes) => {
                // if this would make us larger then 24MB, then no-op
                if (compact.capacity() + num_bytes as usize) > super::TWENTY_FOUR_MB_AS_BYTES {
                    return;
                }

                compact.reserve(num_bytes as usize);
                control.reserve(num_bytes as usize);

                // note: CompactString and String grow at different rates, so we can't assert that
                // their capacities are the same, because they might not be

                assert_eq!(compact, control);
                assert_eq!(compact.len(), control.len());
            }
            Truncate(new_len) => {
                // turn the arbitrary number `new_len` into character indices
                let new_len = control
                    .char_indices()
                    .into_iter()
                    .cycle()
                    .nth(new_len as usize)
                    .unwrap_or_default()
                    .0;

                // then truncate the string
                control.truncate(new_len);
                compact.truncate(new_len);

                assert_eq!(control, compact);
                assert_eq!(control.len(), compact.len());
            }
        }
    }
}
