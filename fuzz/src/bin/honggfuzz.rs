use compact_str_fuzz::Scenario;
use honggfuzz::fuzz;

fn main() {
    loop {
        fuzz!(|scenario: Scenario<'_>| {
            // run our scenario!
            scenario.run();
        });
    }
}
