//! Gungraun (Callgrind) instruction-count benchmarks for `CompactString` creation, conversion,
//! and the internal repr.

use std::hint::black_box;
use std::mem::size_of;

use compact_str::{CompactString, ToCompactString};
use compact_str_6::CompactString as CompactString6;
use gungraun::{library_benchmark, library_benchmark_group, main};

fn make_string(word: &str) -> String {
    String::from(word)
}

fn make_repeated(len: usize) -> String {
    (0..len).map(|_| 'a').collect()
}

// --- CompactString::new / String::from ------------------------------------------------------

#[library_benchmark]
#[bench::chars_0("")]
#[bench::chars_16("im sixteen chars")]
#[bench::chars_24("i am twenty four chars!!")]
fn compact_string_new(word: &str) -> CompactString {
    black_box(CompactString::new(black_box(word)))
}

#[library_benchmark]
#[bench::chars_59("I am a very long string that will get allocated on the heap")]
fn compact_string_from_str(word: &str) -> CompactString {
    black_box(CompactString::from(black_box(word)))
}

#[library_benchmark]
#[bench::chars_59("I am a very long string that will get allocated on the heap")]
fn std_string_from_str(word: &str) -> String {
    black_box(String::from(black_box(word)))
}

// --- CompactString::new(String) -------------------------------------------------------------

#[library_benchmark(setup = make_repeated)]
#[bench::inline_short(size_of::<String>() / 2)]
#[bench::inline_limit(size_of::<String>())]
#[bench::heap_boundary(size_of::<String>() + 1)]
#[bench::heap_medium(64)]
#[bench::heap_large(1024)]
fn compact_string_new_owned(value: String) -> CompactString {
    black_box(CompactString::new(black_box(value)))
}

// --- ToCompactString ------------------------------------------------------------------------

#[library_benchmark]
#[bench::u8(42_u8)]
fn u8_to_compact_string(num: u8) -> CompactString {
    black_box(black_box(num).to_compact_string())
}

#[library_benchmark]
#[bench::u32(54321_u32)]
fn u32_to_compact_string(num: u32) -> CompactString {
    black_box(black_box(num).to_compact_string())
}

#[library_benchmark]
#[bench::isize(-9999999_isize)]
fn isize_to_compact_string(num: isize) -> CompactString {
    black_box(black_box(num).to_compact_string())
}

#[library_benchmark]
#[bench::max(u64::MAX)]
fn u64_to_compact_string(num: u64) -> CompactString {
    black_box(black_box(num).to_compact_string())
}

#[library_benchmark]
#[bench::big(12345678909876543210123456789_u128)]
fn u128_to_compact_string(num: u128) -> CompactString {
    black_box(black_box(num).to_compact_string())
}

#[library_benchmark]
#[bench::yes(true)]
fn bool_to_compact_string(flag: bool) -> CompactString {
    black_box(black_box(flag).to_compact_string())
}

#[library_benchmark(setup = make_string)]
#[bench::hello_world("hello world!")]
fn string_to_compact_string(word: String) -> CompactString {
    black_box(black_box(&word).to_compact_string())
}

#[library_benchmark]
#[bench::a('a')]
fn char_to_compact_string(c: char) -> CompactString {
    black_box(black_box(c).to_compact_string())
}

#[library_benchmark]
#[bench::inline("module")]
#[bench::heap("package.submodule.long_name")]
fn str_to_compact_string(value: &str) -> CompactString {
    black_box(black_box(value).to_compact_string())
}

// --- Internal repr: creation ----------------------------------------------------------------

#[library_benchmark(setup = make_repeated)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn repr_creation_compact_str_6(word: String) -> CompactString6 {
    black_box(CompactString6::new(black_box(&word)))
}

#[library_benchmark(setup = make_repeated)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn repr_creation_compact_str(word: String) -> CompactString {
    black_box(CompactString::new(black_box(&word)))
}

#[library_benchmark(setup = make_repeated)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn repr_creation_std_string(word: String) -> String {
    black_box(String::from(black_box(&word)))
}

// --- Internal repr: access ------------------------------------------------------------------

fn setup_compact_str_6(len: usize) -> CompactString6 {
    CompactString6::new(make_repeated(len))
}

fn setup_compact_str(len: usize) -> CompactString {
    CompactString::new(make_repeated(len))
}

#[library_benchmark(setup = setup_compact_str_6)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_50(50)]
fn repr_access_compact_str_6(compact: CompactString6) {
    black_box(black_box(&compact).as_str());
}

#[library_benchmark(setup = setup_compact_str)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_50(50)]
fn repr_access_compact_str(compact: CompactString) {
    black_box(black_box(&compact).as_str());
}

#[library_benchmark(setup = make_repeated)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_50(50)]
fn repr_access_std_string(std_str: String) {
    black_box(black_box(&std_str).as_str());
}

library_benchmark_group!(
    name = compact_str_benches,
    benchmarks = [
        compact_string_new,
        compact_string_from_str,
        std_string_from_str,
        compact_string_new_owned,
        u8_to_compact_string,
        u32_to_compact_string,
        isize_to_compact_string,
        u64_to_compact_string,
        u128_to_compact_string,
        bool_to_compact_string,
        string_to_compact_string,
        char_to_compact_string,
        str_to_compact_string,
    ]
);

library_benchmark_group!(
    name = repr_benches,
    benchmarks = [
        repr_creation_compact_str_6,
        repr_creation_compact_str,
        repr_creation_std_string,
        repr_access_compact_str_6,
        repr_access_compact_str,
        repr_access_std_string,
    ]
);

main!(library_benchmark_groups = compact_str_benches, repr_benches);
