//! Focused benches for the perf-audit optimizations.
use compact_str::CompactString;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_new(c: &mut Criterion) {
    let mut g = c.benchmark_group("new");
    for (name, s) in [
        ("0", ""),
        ("11", "hello world"),
        ("24", "abcdefghijklmnopqrstuvwx"),
        ("59", "the quick brown fox jumps over the lazy dog and then some.."),
    ] {
        g.bench_with_input(BenchmarkId::from_parameter(name), s, |b, s| {
            b.iter(|| CompactString::new(black_box(s)))
        });
    }
}

fn bench_access(c: &mut Criterion) {
    let inline = CompactString::new("hello world");
    let heap = CompactString::new("the quick brown fox jumps over the lazy dog");
    let mut g = c.benchmark_group("access");
    for (name, s) in [("inline", &inline), ("heap", &heap)] {
        g.bench_with_input(BenchmarkId::new("len", name), s, |b, s| {
            b.iter(|| black_box(s).len())
        });
        g.bench_with_input(BenchmarkId::new("as_str", name), s, |b, s| {
            b.iter(|| black_box(s).as_str())
        });
    }
}

fn bench_mutation(c: &mut Criterion) {
    let mut g = c.benchmark_group("mutation");
    g.bench_function("push_str/short_onto_short", |b| {
        b.iter_batched(
            || CompactString::new("hello "),
            |mut s| {
                s.push_str(black_box("world"));
                s
            },
            criterion::BatchSize::SmallInput,
        )
    });
    g.bench_function("push_str/short_onto_long", |b| {
        b.iter_batched(
            || CompactString::new("the quick brown fox jumps over the lazy dog"),
            |mut s| {
                s.push_str(black_box(" extra"));
                s
            },
            criterion::BatchSize::SmallInput,
        )
    });
    g.bench_function("with_capacity/small", |b| {
        b.iter(|| CompactString::with_capacity(black_box(10)))
    });
    g.bench_function("with_capacity/large", |b| {
        b.iter(|| CompactString::with_capacity(black_box(100)))
    });
    g.bench_function("clone/inline", |b| {
        let s = CompactString::new("hello world");
        b.iter(|| black_box(&s).clone())
    });
    g.bench_function("from_string_buffer", |b| {
        b.iter_batched(
            || String::from("the quick brown fox jumps over the lazy dog"),
            |s| CompactString::from_string_buffer(s),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_new, bench_access, bench_mutation);
criterion_main!(benches);
