use compact_str_fuzz::Scenario;
use honggfuzz::fuzz;

fn main() {
    #[cfg(target_pointer_width = "64")]
    let pointer_width = 64;
    #[cfg(target_pointer_width = "32")]
    let pointer_width = 32;
    println!("Target pointer width: {}", pointer_width);

    loop {
        fuzz!(|scenario: Scenario<'_>| {
            // run our scenario!
            scenario.run();
        });
    }
}
