use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use smart_str::SmartStr;
use smartstring::alias::String as SmartString;
use smol_str::SmolStr;

fn bench_string_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("String Creation");

    // generate a list of 40 strings, with each string being length i and composed of all 'a's
    let words: Vec<String> = (0..40)
        .into_iter()
        .map(|len| (0..len).into_iter().map(|_| 'a').collect())
        .collect();

    for word in words {
        group.bench_with_input(BenchmarkId::new("SmartStr", word.len()), &word, |b, word| {
            b.iter(|| SmartStr::new(word))
        });
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
criterion_group!(creation, bench_string_creation);

fn smart_str_inline(c: &mut Criterion) {
    let word = "im sixteen chars";
    c.bench_function("SmartStr_bench inline", |b| b.iter(|| SmartStr::new(word)));
}
fn smart_str_packed(c: &mut Criterion) {
    let word = "i am twenty four chars!!";
    c.bench_function("SmartStr_bench packed", |b| b.iter(|| SmartStr::new(word)));
}
criterion_group!(smart_str, smart_str_inline, smart_str_packed);

criterion_main!(creation, smart_str);

// fn smart_str_inline(c: &mut Criterion) {
//     let word = "im sixteen chars";
//     c.bench_function("SmartStr inline", |b| b.iter(|| SmartStr::new(word)));
// }

// fn smol_str_inline(c: &mut Criterion) {
//     let word = "im sixteen chars";
//     c.bench_function("SmolStr inline", |b| b.iter(|| SmolStr::new(word)));
// }

// fn smartstring_inline(c: &mut Criterion) {
//     let word = "im sixteen chars";
//     c.bench_function("SmartString inline", |b| b.iter(|| SmartString::from(word)));
// }

// fn smart_str_heap(c: &mut Criterion) {
//     let word = "i am a string that is 40 characters long";
//     c.bench_function("SmartStr heap", |b| b.iter(|| SmartStr::new(word)));
// }

// fn smol_str_heap(c: &mut Criterion) {
//     let word = "i am a string that is 40 characters long";
//     c.bench_function("SmolStr heap", |b| b.iter(|| SmolStr::new(word)));
// }

// fn smartstring_heap(c: &mut Criterion) {
//     let word = "i am a string that is 40 characters long";
//     c.bench_function("SmartString heap", |b| b.iter(|| SmartString::from(word)));
// }

// fn smart_str_cloning(c: &mut Criterion) {
//     let word = "i am a string that is 40 characters long";
//     let og = SmartStr::new(word);

//     c.bench_function("SmartStr cloning", |b| {
//         b.iter(|| {
//             let clone = og.clone();
//             assert_eq!(og.as_str(), clone.as_str());
//         })
//     });
// }

// fn smol_str_cloning(c: &mut Criterion) {
//     let word = "i am a string that is 40 characters long";
//     let og = SmolStr::new(word);

//     c.bench_function("SmolStr cloning", |b| {
//         b.iter(|| {
//             let clone = og.clone();
//             assert_eq!(og.as_str(), clone.as_str());
//         })
//     });
// }

// fn smartstring_cloning(c: &mut Criterion) {
//     let word = "i am a string that is 40 characters long";
//     let og = SmartString::from(word);
//     c.bench_function("SmartString cloning", |b| {
//         b.iter(|| {
//             let clone = og.clone();
//             assert_eq!(og.as_str(), clone.as_str());
//         })
//     });
// }

// criterion_group!(
//     inline,
//     smart_str_inline,
//     smol_str_inline,
//     smartstring_inline
// );
// criterion_group!(heap, smart_str_heap, smol_str_heap, smartstring_heap);
// criterion_group!(
//     cloning,
//     smart_str_cloning,
//     smol_str_cloning,
//     smartstring_cloning
// );
// criterion_main!(inline, heap, cloning);
