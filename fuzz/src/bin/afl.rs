use afl::fuzz;
use compact_str_fuzz::Scenario;

pub fn main() {
    fuzz!(|scenario: Scenario<'_>| {
        // run our scenario!
        scenario.run();
    });
}
