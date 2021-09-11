use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};
use smart_str::SmartStr;
use tracing::TracingAllocator;

#[global_allocator]
pub static ALLOCATOR: TracingAllocator = TracingAllocator::new();

#[cfg(target_pointer_width = "64")]
const INLINED_SIZE: usize = 23;
#[cfg(target_pointer_width = "32")]
const INLINED_SIZE: usize = 11;

#[test]
fn test_randomized_allocations() {
    // create an rng
    let seed: u64 = rand::thread_rng().gen();
    eprintln!("using seed: {}_u64", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    // generate a list of up to 10,000 words, with each word being up to 500 characters long
    let num_words = rng.gen_range(0..10_000);
    let words: Vec<String> = (0..num_words)
        .map(|_| {
            let len = rng.gen_range(0..500);
            rng.clone()
                .sample_iter(&Alphanumeric)
                .take(len)
                .map(char::from)
                .collect()
        })
        .collect();

    // count the number of allocations there should be
    let mut long_strs = 0;

    for w in words {
        if w.len() > INLINED_SIZE {
            long_strs += 1;
        }

        ALLOCATOR.enable_tracing();
        {
            SmartStr::new(&w);
        }
        ALLOCATOR.disable_tracing();
    }

    let events = ALLOCATOR.events();
    let actual_allocs = events
        .iter()
        .filter(|event| matches!(event, tracing::Event::Alloc { .. }))
        .count();
    // the number of alloc events should equal the number of strings > INLINED_SIZE characters long
    assert_eq!(long_strs, actual_allocs);

    // adding all of the Alloc and Freed events should result in 0, meaning we freed all the memory
    // we allocated
    let total_mem = events.iter().fold(0, |mem, event| mem + event.delta());
    assert_eq!(total_mem, 0);
}
