use compact_str::CompactStr;
use criterion::{
    criterion_group,
    criterion_main,
    BenchmarkId,
    Criterion,
};
use smartstring::alias::String as SmartString;
use smol_str::SmolStr;

fn creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("String Creation");

    let words: Vec<String> = vec![0, 11, 12, 22, 23, 24, 25, 50]
        .into_iter()
        .map(|len| (0..len).into_iter().map(|_| 'a').collect())
        .collect();

    for word in words {
        group.bench_with_input(
            BenchmarkId::new("CompactStr", word.len()),
            &word,
            |b, word| b.iter(|| CompactStr::new(word)),
        );
        group.bench_with_input(BenchmarkId::new("SmolStr", word.len()), &word, |b, word| {
            b.iter(|| SmolStr::new(word))
        });
        group.bench_with_input(
            BenchmarkId::new("SmartString", word.len()),
            &word,
            |b, word| b.iter(|| SmartString::from(word)),
        );
        group.bench_with_input(
            BenchmarkId::new("std::String", word.len()),
            &word,
            |b, word| b.iter(|| String::from(word)),
        );
    }
}
criterion_group!(string_creation, creation);

fn cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("String Cloning");

    let words: Vec<String> = vec![0, 11, 12, 22, 23, 24, 25, 50]
        .into_iter()
        .map(|len| (0..len).into_iter().map(|_| 'a').collect())
        .collect();

    for word in words {
        let compact = CompactStr::new(&word);
        group.bench_with_input(
            BenchmarkId::new("CompactStr", compact.len()),
            &compact,
            |b, compact| b.iter(|| compact.clone()),
        );

        let smol = SmolStr::new(&word);
        group.bench_with_input(BenchmarkId::new("SmolStr", smol.len()), &smol, |b, smol| {
            b.iter(|| smol.clone())
        });

        let smart = SmartString::from(&word);
        group.bench_with_input(
            BenchmarkId::new("SmartString", smart.len()),
            &smart,
            |b, smart| b.iter(|| smart.clone()),
        );

        let string = String::from(&word);
        group.bench_with_input(
            BenchmarkId::new("std::String", string.len()),
            &string,
            |b, string| b.iter(|| string.clone()),
        );
    }
}
criterion_group!(string_cloning, cloning);

fn access(c: &mut Criterion) {
    let mut group = c.benchmark_group("String Access");

    let words: Vec<String> = vec![0, 11, 12, 22, 23, 24, 25, 50]
        .into_iter()
        .map(|len| (0..len).into_iter().map(|_| 'a').collect())
        .collect();

    for word in words {
        let compact = CompactStr::new(&word);
        group.bench_with_input(
            BenchmarkId::new("CompactStr", compact.len()),
            &compact,
            |b, compact| b.iter(|| compact.as_str()),
        );

        let smol = SmolStr::new(&word);
        group.bench_with_input(BenchmarkId::new("SmolStr", smol.len()), &smol, |b, smol| {
            b.iter(|| smol.as_str())
        });

        let smart = SmartString::from(&word);
        group.bench_with_input(
            BenchmarkId::new("SmartString", smart.len()),
            &smart,
            |b, smart| b.iter(|| smart.as_str()),
        );

        let string = String::from(&word);
        group.bench_with_input(
            BenchmarkId::new("std::String", string.len()),
            &string,
            |b, string| b.iter(|| string.as_str()),
        );
    }
}
criterion_group!(string_access, access);

criterion_main!(string_creation, string_cloning, string_access);
