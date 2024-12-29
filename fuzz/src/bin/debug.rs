//! Binary that allows you to pass an input file, it creates a [`compact_str_fuzz::Scenario`] and
//! then runs it.
//!
//! This is helpful when AFL finds failures and we need to reproduce them.

use std::path::PathBuf;

use arbitrary::{Arbitrary, Unstructured};
use compact_str_fuzz::Scenario;

pub fn main() {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("no path provided!");
    let data = std::fs::read(path).expect("failed to read input file");
    let mut unstructured = Unstructured::new(&data);
    let scenario = Scenario::arbitrary(&mut unstructured).expect("failed to create Scenario");

    println!("Scenario: {:?}", scenario);

    scenario.run();
}
