//! This is a fuzzing harness we use to test the [`compact_str`] crate, there are three components
//! of this harness:
//!
//! 1. [`Creation`] methods
//! 2. [`Action`]s that we can take
//! 3. A [`Scenario`] which is comprised of a [`Creation`] method, and a collection of [`Action`]s
//!
//! Publically we expose the [`Scenario`] struct, which implements the [`arbitrary::Arbitrary`]
//! trait. After generating a [`Scenario`] we `run()` it, which creates a [`CompactString`] and
//! "control" [`String`] via our [`Creation`] method, and then runs our collection of [`Action`]s,
//! and along the way asserts several invariants.

use arbitrary::Arbitrary;
use compact_str::CompactString;

const MAX_INLINE_LENGTH: usize = std::mem::size_of::<String>();
const MIN_HEAP_CAPACITY: usize = std::mem::size_of::<usize>() * 4;
const TWENTY_FOUR_MB_AS_BYTES: usize = 24 * 1024 * 1024;

mod actions;
mod creation;

use actions::Action;
use creation::Creation;

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

            // make sure the as_mut_bytes() APIs return the same value
            //
            // SAFETY: We're not actually modifying anything here, just checking equality
            unsafe { assert_eq!(compact.as_bytes_mut(), control.as_bytes_mut()) };

            // assert the fmt::Debug impls are the same
            let debug_compact = format!("{:?}", compact);
            let debug_std_str = format!("{:?}", control);
            assert_eq!(debug_compact, debug_std_str);

            // assert the fmt::Display impls are the same
            #[allow(clippy::useless_format)]
            let display_compact = format!("{}", compact);
            #[allow(clippy::useless_format)]
            let display_std_str = format!("{}", control);
            assert_eq!(display_compact, display_std_str);

            // after making all of our modifications, assert again that we don't misinterpret
            let compact = assert_not_option(compact);

            // Convert our CompactString into a String and assert they're equal. This covers our
            // From<CompactString> for String impl
            let compact_into_string = String::from(compact);
            assert_eq!(compact_into_string, control);
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
