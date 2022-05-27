//! Benchmarks for various APIs to make sure `CompactString` is at least no slower than `String`

use std::time::Instant;

use compact_str::CompactString;
use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

static VERY_LONG_STR: &str = include_str!("../data/moby10b.txt");

fn compact_string_inline_length(c: &mut Criterion) {
    let word = "i am short";
    let compact_str = CompactString::new(word);
    c.bench_function("inline length", |b| {
        b.iter(|| {
            let len = black_box(compact_str.len());
            assert_eq!(len, word.len());
        })
    });
}

fn compact_string_heap_length(c: &mut Criterion) {
    let word = "I am a very long string that will get allocated on the heap";
    let compact_str = CompactString::new(word);
    c.bench_function("heap length", |b| {
        b.iter(|| {
            let len = black_box(compact_str.len());
            assert_eq!(len, word.len());
        })
    });
}

fn compact_string_very_big_heap_length(c: &mut Criterion) {
    let compact_str = CompactString::new(VERY_LONG_STR);
    c.bench_function("very long heap length", |b| {
        b.iter(|| {
            let len = black_box(compact_str.len());
            assert_eq!(len, VERY_LONG_STR.len());
        })
    });
}

fn compact_string_reserve_small(c: &mut Criterion) {
    c.bench_function("reserve small", |b| {
        b.iter(|| {
            let mut compact_str = CompactString::default();
            black_box(compact_str.reserve(10));
        })
    });
}

fn compact_string_reserve_large(c: &mut Criterion) {
    c.bench_function("reserve large", |b| {
        b.iter(|| {
            let mut compact_str = CompactString::default();
            black_box(compact_str.reserve(100));
        })
    });
}

fn compact_string_clone_small(c: &mut Criterion) {
    let compact = CompactString::new("i am short");
    c.bench_function("clone small", |b| b.iter(|| compact.clone()));
}

fn compact_string_clone_large_and_modify(c: &mut Criterion) {
    let compact = CompactString::new("I am a very long string that will get allocated on the heap");
    c.bench_function("clone large", |b| {
        b.iter(|| {
            let mut clone = compact.clone();
            clone.push('!');
            clone.push(' ');
            clone.push_str("And that is quite cool~");
        })
    });
}

fn compact_string_extend_chars_empty(c: &mut Criterion) {
    c.bench_function("extend chars empty", |b| {
        b.iter(|| {
            let mut compact =
                CompactString::new("I am a very long string that will get allocated on the heap");
            compact.extend("".chars());
        })
    });
}

fn compact_string_extend_chars_short(c: &mut Criterion) {
    c.bench_function("extend chars short", |b| {
        b.iter(|| {
            let mut compact = CompactString::new("hello");
            compact.extend((0..10).map(|_| '!'));
        })
    });
}

fn compact_string_extend_chars_inline_to_heap_20(c: &mut Criterion) {
    c.bench_function("extend chars inline to heap, 20", |b| {
        b.iter(|| {
            let mut compact = CompactString::new("hello world");
            compact.extend((0..20).map(|_| '!'));
        })
    });
}

fn compact_string_extend_chars_heap_20(c: &mut Criterion) {
    c.bench_function("extend chars heap, 20", |b| {
        b.iter(|| {
            let mut compact =
                CompactString::new("this is a long string that will start on the heap");
            compact.extend((0..20).map(|_| '!'));
        })
    });
}

fn compact_string_from_string_inline(c: &mut Criterion) {
    c.bench_function("compact_str_from_string_inline", |b| {
        b.iter_custom(|iters| {
            let mut durations = vec![];
            for _ in 0..iters {
                let word = String::from("I am short");

                // only time how long it takes to go from String -> CompactString
                let start = Instant::now();
                let c = CompactString::from(word);
                let duration = start.elapsed();

                // explicitly drop _after_ we've finished timing
                drop(c);

                durations.push(duration);
            }
            durations.into_iter().sum()
        });
    });
}

fn compact_string_from_string_heap(c: &mut Criterion) {
    c.bench_function("compact_str_from_string_heap", |b| {
        b.iter_custom(|iters| {
            let mut durations = vec![];
            for _ in 0..iters {
                let word = String::from("I am a long string, look at me!");

                // only time how long it takes to go from String -> CompactString
                let start = Instant::now();
                let c = CompactString::from(word);
                let duration = start.elapsed();

                // explicitly drop _after_ we've finished timing
                drop(c);

                durations.push(duration);
            }
            durations.into_iter().sum()
        });
    });
}

fn compact_string_from_string_heap_long(c: &mut Criterion) {
    c.bench_function("compact_str_from_string_heap_long", |b| {
        b.iter_custom(|iters| {
            let mut durations = vec![];
            for _ in 0..iters {
                let word = String::from(VERY_LONG_STR);

                // only time how long it takes to go from String -> CompactString
                let start = Instant::now();
                let c = CompactString::from(word);
                let duration = start.elapsed();

                // explicitly drop _after_ we've finished timing
                drop(c);

                durations.push(duration);
            }
            durations.into_iter().sum()
        });
    });
}

fn std_string_short_length(c: &mut Criterion) {
    let word = "i am short";
    let string = String::from(word);
    c.bench_function("std string short length", |b| {
        b.iter(|| {
            let len = black_box(string.len());
            assert_eq!(len, word.len());
        })
    });
}

fn std_string_long_length(c: &mut Criterion) {
    let word = "I am a very long string that will get allocated on the heap";
    let string = String::from(word);
    c.bench_function("std string long length", |b| {
        b.iter(|| {
            let len = black_box(string.len());
            assert_eq!(len, word.len());
        })
    });
}

fn std_string_very_long_length(c: &mut Criterion) {
    let string = String::from(VERY_LONG_STR);
    c.bench_function("std string very long length", |b| {
        b.iter(|| {
            let len = black_box(string.len());
            assert_eq!(len, VERY_LONG_STR.len());
        })
    });
}

fn std_str_reserve_small(c: &mut Criterion) {
    c.bench_function("std str reserve small", |b| {
        b.iter(|| {
            let mut std_str = String::default();
            black_box(std_str.reserve(10));
        })
    });
}

fn std_str_reserve_large(c: &mut Criterion) {
    c.bench_function("std str reserve large", |b| {
        b.iter(|| {
            let mut std_str = String::default();
            black_box(std_str.reserve(100));
        })
    });
}

fn std_str_clone_small(c: &mut Criterion) {
    let std_str = String::from("i am short");
    c.bench_function("std str clone small", |b| b.iter(|| std_str.clone()));
}

fn std_str_clone_large_and_modify(c: &mut Criterion) {
    let std_str = String::from("I am a very long string that will get allocated on the heap");
    c.bench_function("std str clone large", |b| {
        b.iter(|| {
            let mut clone = std_str.clone();
            clone.push('!');
            clone.push(' ');
            clone.push_str("And that is quite cool~");
        })
    });
}

fn std_str_extend_chars_empty(c: &mut Criterion) {
    c.bench_function("std str extend chars empty", |b| {
        b.iter(|| {
            let mut std_str = String::from("hello");
            std_str.extend("".chars());
        })
    });
}

fn std_str_extend_chars_short(c: &mut Criterion) {
    c.bench_function("std str extend chars short", |b| {
        b.iter(|| {
            let mut std_str = String::from("hello");
            std_str.extend((0..10).map(|_| '!'));
        })
    });
}

fn std_str_str_extend_chars_20(c: &mut Criterion) {
    c.bench_function("std str extend chars 20", |b| {
        b.iter(|| {
            let mut std_str = String::from("hello");
            std_str.extend((0..20).map(|_| '!'));
        })
    });
}

criterion_group!(
    compact_str,
    compact_string_inline_length,
    compact_string_heap_length,
    compact_string_very_big_heap_length,
    compact_string_reserve_small,
    compact_string_reserve_large,
    compact_string_clone_small,
    compact_string_clone_large_and_modify,
    compact_string_extend_chars_empty,
    compact_string_extend_chars_short,
    compact_string_extend_chars_inline_to_heap_20,
    compact_string_extend_chars_heap_20,
    compact_string_from_string_inline,
    compact_string_from_string_heap,
    compact_string_from_string_heap_long
);
criterion_group!(
    std_string,
    std_string_short_length,
    std_string_long_length,
    std_string_very_long_length,
    std_str_reserve_small,
    std_str_reserve_large,
    std_str_clone_small,
    std_str_clone_large_and_modify,
    std_str_extend_chars_empty,
    std_str_extend_chars_short,
    std_str_str_extend_chars_20,
);

criterion_main!(compact_str, std_string);
