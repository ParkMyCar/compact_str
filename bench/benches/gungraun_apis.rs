//! Gungraun (Callgrind) instruction-count benchmarks for various APIs, making sure
//! `CompactString` is at least no slower than `String`.

use std::hint::black_box;

use compact_str::CompactString;
use gungraun::{library_benchmark, library_benchmark_group, main};

static VERY_LONG_STR: &str = include_str!("../data/moby10b.txt");

fn make_compact(word: &str) -> CompactString {
    CompactString::new(word)
}

fn make_string(word: &str) -> String {
    String::from(word)
}

// --- CompactString --------------------------------------------------------------------------

#[library_benchmark(setup = make_compact)]
#[bench::inline("i am short")]
#[bench::heap("I am a very long string that will get allocated on the heap")]
#[bench::very_long(VERY_LONG_STR)]
fn compact_string_length(compact_str: CompactString) -> usize {
    black_box(black_box(&compact_str).len())
}

#[library_benchmark]
#[bench::small(10)]
#[bench::large(100)]
fn compact_string_reserve(additional: usize) -> CompactString {
    let mut compact_str = CompactString::default();
    compact_str.reserve(black_box(additional));
    black_box(compact_str)
}

#[library_benchmark(setup = make_compact)]
#[bench::small("i am short")]
fn compact_string_clone(compact: CompactString) -> CompactString {
    black_box(black_box(&compact).clone())
}

#[library_benchmark(setup = make_compact)]
#[bench::large("I am a very long string that will get allocated on the heap")]
fn compact_string_clone_and_modify(compact: CompactString) -> CompactString {
    let mut clone = black_box(&compact).clone();
    clone.push('!');
    clone.push(' ');
    clone.push_str("And that is quite cool~");
    black_box(clone)
}

#[library_benchmark]
#[bench::empty("I am a very long string that will get allocated on the heap", 0)]
#[bench::short("hello", 10)]
#[bench::inline_to_heap_20("hello world", 20)]
#[bench::heap_20("this is a long string that will start on the heap", 20)]
fn compact_string_extend_chars(base: &str, count: usize) -> CompactString {
    let mut compact = CompactString::new(black_box(base));
    compact.extend((0..count).map(|_| '!'));
    black_box(compact)
}

#[library_benchmark(setup = make_string)]
#[bench::inline("I am short")]
#[bench::heap("I am a long string, look at me!")]
#[bench::heap_long(VERY_LONG_STR)]
fn compact_string_from_string(word: String) -> CompactString {
    black_box(CompactString::from(black_box(word)))
}

// --- std::String ----------------------------------------------------------------------------

#[library_benchmark(setup = make_string)]
#[bench::short("i am short")]
#[bench::long("I am a very long string that will get allocated on the heap")]
#[bench::very_long(VERY_LONG_STR)]
fn std_string_length(string: String) -> usize {
    black_box(black_box(&string).len())
}

#[library_benchmark]
#[bench::small(10)]
#[bench::large(100)]
fn std_str_reserve(additional: usize) -> String {
    let mut std_str = String::default();
    std_str.reserve(black_box(additional));
    black_box(std_str)
}

#[library_benchmark(setup = make_string)]
#[bench::small("i am short")]
fn std_str_clone(std_str: String) -> String {
    black_box(black_box(&std_str).clone())
}

#[library_benchmark(setup = make_string)]
#[bench::large("I am a very long string that will get allocated on the heap")]
fn std_str_clone_and_modify(std_str: String) -> String {
    let mut clone = black_box(&std_str).clone();
    clone.push('!');
    clone.push(' ');
    clone.push_str("And that is quite cool~");
    black_box(clone)
}

#[library_benchmark]
#[bench::empty("hello", 0)]
#[bench::short("hello", 10)]
#[bench::chars_20("hello", 20)]
fn std_str_extend_chars(base: &str, count: usize) -> String {
    let mut std_str = String::from(black_box(base));
    std_str.extend((0..count).map(|_| '!'));
    black_box(std_str)
}

library_benchmark_group!(
    name = compact_str_benches,
    benchmarks = [
        compact_string_length,
        compact_string_reserve,
        compact_string_clone,
        compact_string_clone_and_modify,
        compact_string_extend_chars,
        compact_string_from_string,
    ]
);

library_benchmark_group!(
    name = std_string,
    benchmarks = [
        std_string_length,
        std_str_reserve,
        std_str_clone,
        std_str_clone_and_modify,
        std_str_extend_chars,
    ]
);

main!(library_benchmark_groups = compact_str_benches, std_string);
