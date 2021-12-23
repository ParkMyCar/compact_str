//! Benchmarks for various APIs to make sure `CompactStr` is at least no slower than `String`

use compact_str::CompactStr;
use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

static VERY_LONG_STR: &str = include_str!("moby10b.txt");

fn compact_str_inline_length(c: &mut Criterion) {
    let word = "i am short";
    let compact_str = CompactStr::new(word);
    c.bench_function("inline length", |b| {
        b.iter(|| {
            let len = black_box(compact_str.len());
            assert_eq!(len, word.len());
        })
    });
}

fn compact_str_heap_length(c: &mut Criterion) {
    let word = "I am a very long string that will get allocated on the heap";
    let compact_str = CompactStr::new(word);
    c.bench_function("heap length", |b| {
        b.iter(|| {
            let len = black_box(compact_str.len());
            assert_eq!(len, word.len());
        })
    });
}

fn compact_str_very_big_heap_length(c: &mut Criterion) {
    let compact_str = CompactStr::new(VERY_LONG_STR);
    c.bench_function("very long heap length", |b| {
        b.iter(|| {
            let len = black_box(compact_str.len());
            assert_eq!(len, VERY_LONG_STR.len());
        })
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

criterion_group!(
    compact_str,
    compact_str_inline_length,
    compact_str_heap_length,
    compact_str_very_big_heap_length
);
criterion_group!(
    std_string,
    std_string_short_length,
    std_string_long_length,
    std_string_very_long_length
);

criterion_main!(compact_str, std_string);
