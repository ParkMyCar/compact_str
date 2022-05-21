use compact_str::{
    CompactStr,
    ToCompactStr,
};
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

fn bench_to_compact_str(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("u8::to_compact_str", "42"),
        &42_u8,
        |b, num| b.iter(|| num.to_compact_str()),
    );

    c.bench_with_input(
        BenchmarkId::new("u32::to_compact_str", "54321"),
        &54321_u32,
        |b, num| b.iter(|| num.to_compact_str()),
    );

    c.bench_with_input(
        BenchmarkId::new("isize::to_compact_str", "-9999999"),
        &-9999999_isize,
        |b, num| b.iter(|| num.to_compact_str()),
    );

    c.bench_with_input(
        BenchmarkId::new("u64::to_compact_str", "MAX"),
        &u64::MAX,
        |b, num| b.iter(|| num.to_compact_str()),
    );

    c.bench_with_input(
        BenchmarkId::new("u128::to_compact_str", "12345678909876543210123456789"),
        &12345678909876543210123456789_u128,
        |b, num| b.iter(|| num.to_compact_str()),
    );

    c.bench_with_input(
        BenchmarkId::new("bool::to_compact_str", "true"),
        &true,
        |b, flag| b.iter(|| flag.to_compact_str()),
    );

    c.bench_with_input(
        BenchmarkId::new("String::to_compact_str", "hello world!"),
        &String::from("hello world!"),
        |b, word| b.iter(|| word.to_compact_str()),
    );

    c.bench_with_input(
        BenchmarkId::new("char::to_compact_str", "a"),
        &'a',
        |b, c| b.iter(|| c.to_compact_str()),
    );
}

criterion_group!(compact_str, bench_new, bench_to_compact_str);
criterion_main!(compact_str);
