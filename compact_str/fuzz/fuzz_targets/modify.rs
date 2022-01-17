#![no_main]

use arbitrary::Arbitrary;
use compact_str::CompactStr;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Scenario<'a> {
    word: String,
    actions: Vec<Modification<'a>>,
}

fuzz_target!(|scenario: Scenario| {
    let actions = scenario.actions;

    let mut word = scenario.word;
    let mut compact = CompactStr::new(&word);

    actions
        .into_iter()
        .for_each(|a| a.perform(&mut word, &mut compact));
});

#[derive(Arbitrary, Debug)]
enum Modification<'a> {
    Push(char),
    Pop(u32),
    PushStr(&'a str),
    ExtendChars(Vec<char>),
    ExtendStr(Vec<&'a str>),
}

impl Modification<'_> {
    pub fn perform(self, control: &mut String, compact: &mut CompactStr) {
        use Modification::*;

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
        }
    }
}
