use compact_str::CompactStr;
use criterion::{
    criterion_group,
    criterion_main,
    BenchmarkId,
    Criterion,
};

fn bench_new(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("CompactStr::new", "0 chars"),
        &"",
        |b, word| b.iter(|| CompactStr::new(word)),
    );

    c.bench_with_input(
        BenchmarkId::new("CompactStr::new", "16 chars"),
        &"im sixteen chars",
        |b, word| b.iter(|| CompactStr::new(word)),
    );

    c.bench_with_input(
        BenchmarkId::new("CompactStr::new", "24 chars"),
        &"i am twenty four chars!!",
        |b, word| b.iter(|| CompactStr::new(word)),
    );

    c.bench_with_input(
        BenchmarkId::new("CompactStr::new", "59 chars"),
        &"I am a very long string that will get allocated on the heap",
        |b, word| b.iter(|| CompactStr::new(word)),
    );

    c.bench_with_input(
        BenchmarkId::new("String::new", "59 chars"),
        &"I am a very long string that will get allocated on the heap",
        |b, &word| b.iter(|| String::from(word)),
    );
}

criterion_group!(compact_str, bench_new);
criterion_main!(compact_str);
