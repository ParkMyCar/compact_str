use compact_str::CompactStr;
use rand::{distributions, rngs::StdRng, Rng, SeedableRng};

#[test]
fn test_randomized_option() {
    // create an rng
    let seed: u64 = rand::thread_rng().gen();
    eprintln!("using seed: {}_u64", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let runs = option_env!("RANDOMIZED_RUNS")
        .map(|v| v.parse().expect("provided non-integer value?"))
        .unwrap_or(50_000);
    println!("Running with RANDOMIZED_RUNS: {}", runs);

    // generate random words up to 24 characters long
    for _ in 0..runs {
        let len = rng.gen_range(0..=30);
        let word: String = rng
            .clone()
            .sample_iter::<char, _>(&distributions::Standard)
            .take(len)
            .map(char::from)
            .collect();
        let compact = CompactStr::new(&word);

        let maybe_compact =
            unsafe { std::mem::transmute::<CompactStr, Option<CompactStr>>(compact) };

        // we should never mistake a valid CompactStr as None
        assert!(maybe_compact.is_some());
    }
}
