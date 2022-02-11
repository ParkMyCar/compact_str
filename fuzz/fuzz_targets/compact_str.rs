#![no_main]

use compact_str_fuzz::Scenario;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|scenario: Scenario<'_>| {
    // Given random creation method, if we can create a string
    if let Some((mut compact, mut control)) = scenario.creation.create() {
        // run some actions, asserting properties along the way
        scenario
            .actions
            .into_iter()
            .for_each(|a| a.perform(&mut control, &mut compact));

        // make sure our strings are the same
        assert_eq!(compact, control);
    }
});
