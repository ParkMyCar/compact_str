use compact_str::CompactStr;
use criterion::{
    criterion_group,
    criterion_main,
    Criterion,
};

fn empty(c: &mut Criterion) {
    let word = "";
    c.bench_function("empty", |b| b.iter(|| CompactStr::new(word)));
}

fn inline(c: &mut Criterion) {
    let word = "im sixteen chars";
    c.bench_function("inline", |b| b.iter(|| CompactStr::new(word)));
}

fn packed(c: &mut Criterion) {
    let word = "i am twenty four chars!!";
    c.bench_function("packed", |b| b.iter(|| CompactStr::new(word)));
}

fn heap(c: &mut Criterion) {
    let word = "I am a very long string that will get allocated on the heap";
    c.bench_function("heap", |b| b.iter(|| CompactStr::new(word)));
}

fn std_string(c: &mut Criterion) {
    let word = "I am a very long string that will get allocated on the heap";
    c.bench_function("std_string", |b| b.iter(|| String::from(word)));
}

criterion_group!(compact_str, empty, inline, packed, heap, std_string);
criterion_main!(compact_str);
