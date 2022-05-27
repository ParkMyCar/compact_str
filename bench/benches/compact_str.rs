use compact_str::{
    CompactString,
    ToCompactString,
};
use criterion::{
    criterion_group,
    criterion_main,
    BenchmarkId,
    Criterion,
};

fn bench_new(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("CompactString::new", "0 chars"),
        &"",
        |b, word| b.iter(|| CompactString::new(word)),
    );

    c.bench_with_input(
        BenchmarkId::new("CompactString::new", "16 chars"),
        &"im sixteen chars",
        |b, word| b.iter(|| CompactString::new(word)),
    );

    c.bench_with_input(
        BenchmarkId::new("CompactString::new", "24 chars"),
        &"i am twenty four chars!!",
        |b, word| b.iter(|| CompactString::new(word)),
    );

    c.bench_with_input(
        BenchmarkId::new("CompactString::new", "59 chars"),
        &"I am a very long string that will get allocated on the heap",
        |b, word| b.iter(|| CompactString::new(word)),
    );

    c.bench_with_input(
        BenchmarkId::new("String::new", "59 chars"),
        &"I am a very long string that will get allocated on the heap",
        |b, &word| b.iter(|| String::from(word)),
    );
}

fn bench_to_compact_string(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("u8::to_compact_string", "42"),
        &42_u8,
        |b, num| b.iter(|| num.to_compact_string()),
    );

    c.bench_with_input(
        BenchmarkId::new("u32::to_compact_string", "54321"),
        &54321_u32,
        |b, num| b.iter(|| num.to_compact_string()),
    );

    c.bench_with_input(
        BenchmarkId::new("isize::to_compact_string", "-9999999"),
        &-9999999_isize,
        |b, num| b.iter(|| num.to_compact_string()),
    );

    c.bench_with_input(
        BenchmarkId::new("u64::to_compact_string", "MAX"),
        &u64::MAX,
        |b, num| b.iter(|| num.to_compact_string()),
    );

    c.bench_with_input(
        BenchmarkId::new("u128::to_compact_string", "12345678909876543210123456789"),
        &12345678909876543210123456789_u128,
        |b, num| b.iter(|| num.to_compact_string()),
    );

    c.bench_with_input(
        BenchmarkId::new("bool::to_compact_string", "true"),
        &true,
        |b, flag| b.iter(|| flag.to_compact_string()),
    );

    c.bench_with_input(
        BenchmarkId::new("String::to_compact_string", "hello world!"),
        &String::from("hello world!"),
        |b, word| b.iter(|| word.to_compact_string()),
    );

    c.bench_with_input(
        BenchmarkId::new("char::to_compact_string", "a"),
        &'a',
        |b, c| b.iter(|| c.to_compact_string()),
    );
}

criterion_group!(compact_str, bench_new, bench_to_compact_string);
criterion_main!(compact_str);
