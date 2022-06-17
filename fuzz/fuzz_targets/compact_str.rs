#![no_main]

use compact_str_fuzz::Scenario;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|scenario: Scenario<'_>| {
    // run our scenario!
    scenario.run()
});
