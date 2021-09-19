use rand::{distributions, rngs::StdRng, Rng, SeedableRng};
use compact_str::CompactStr;

#[cfg(target_pointer_width = "64")]
const MAX_INLINED_SIZE: usize = 24;
#[cfg(target_pointer_width = "32")]
const MAX_INLINED_SIZE: usize = 12;

#[test]
fn test_randomized_roundtrip() {
    // create an rng
    let seed: u64 = rand::thread_rng().gen();
    eprintln!("using seed: {}_u64", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let runs = option_env!("RANDOMIZED_RUNS")
        .map(|v| v.parse().expect("provided non-integer value?"))
        .unwrap_or(50_000);
    println!("Running with RANDOMIZED_RUNS: {}", runs);

    // generate random words up to 60 characters long
    for _ in 0..runs {
        let len = rng.gen_range(0..60);
        let word: String = rng
            .clone()
            .sample_iter::<char, _>(&distributions::Standard)
            .take(len)
            .map(char::from)
            .collect();

        let compact = CompactStr::new(&word);

        // assert the word roundtrips
        assert_eq!(compact, word);

        // assert it's properly allocated
        if compact.len() < MAX_INLINED_SIZE {
            assert!(!compact.is_heap_allocated())
        } else if compact.len() == MAX_INLINED_SIZE && compact.as_bytes()[0] <= 127 {
            assert!(!compact.is_heap_allocated())
        } else {
            assert!(compact.is_heap_allocated())
        }
    }
}
