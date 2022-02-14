use arbitrary::{
    Arbitrary,
    Unstructured,
};
use compact_str_fuzz::Scenario;

pub fn main() {
    let input_file = std::env::args().nth(1).expect("no input file given");
    let input_buffer = std::fs::read(&input_file).expect("failed to read input file");

    let mut unstructured = Unstructured::new(&input_buffer);
    let scenario: Scenario<'_> =
        Arbitrary::arbitrary(&mut unstructured).expect("failed to generate scenario");

    println!("{:#?}", scenario);

    if let Some((mut compact, mut control)) = scenario.creation.create() {
        // run some actions, asserting properties along the way
        scenario
            .actions
            .into_iter()
            .for_each(|a| a.perform(&mut control, &mut compact));

        // make sure our strings are the same
        assert_eq!(compact, control);
    }
}
