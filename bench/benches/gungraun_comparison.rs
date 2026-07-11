//! Gungraun (Callgrind) instruction-count comparison of `CompactString`, `SmolStr`,
//! `SmartString`, and `std::String` across a range of lengths. Each group uses `compare_by_id`
//! so the same length is compared across types.

use std::hint::black_box;

use compact_str::CompactString;
use gungraun::{library_benchmark, library_benchmark_group, main};
use smartstring::alias::String as SmartString;
use smol_str::SmolStr;

fn make_word(len: usize) -> String {
    (0..len).map(|_| 'a').collect()
}

fn setup_compact(len: usize) -> CompactString {
    CompactString::new(make_word(len))
}

fn setup_smol(len: usize) -> SmolStr {
    SmolStr::new(make_word(len))
}

fn setup_smart(len: usize) -> SmartString {
    SmartString::from(make_word(len))
}

fn setup_string(len: usize) -> String {
    make_word(len)
}

// --- Creation -------------------------------------------------------------------------------

#[library_benchmark(setup = make_word)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn creation_compact_string(word: String) -> CompactString {
    black_box(CompactString::new(black_box(&word)))
}

#[library_benchmark(setup = make_word)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn creation_smol_str(word: String) -> SmolStr {
    black_box(SmolStr::new(black_box(&word)))
}

#[library_benchmark(setup = make_word)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn creation_smart_string(word: String) -> SmartString {
    black_box(SmartString::from(black_box(&word)))
}

#[library_benchmark(setup = make_word)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn creation_std_string(word: String) -> String {
    black_box(String::from(black_box(&word)))
}

// --- Cloning --------------------------------------------------------------------------------

#[library_benchmark(setup = setup_compact)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn cloning_compact_string(compact: CompactString) -> CompactString {
    black_box(black_box(&compact).clone())
}

#[library_benchmark(setup = setup_smol)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn cloning_smol_str(smol: SmolStr) -> SmolStr {
    black_box(black_box(&smol).clone())
}

#[library_benchmark(setup = setup_smart)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn cloning_smart_string(smart: SmartString) -> SmartString {
    black_box(black_box(&smart).clone())
}

#[library_benchmark(setup = setup_string)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn cloning_std_string(string: String) -> String {
    black_box(black_box(&string).clone())
}

// --- Access ---------------------------------------------------------------------------------

#[library_benchmark(setup = setup_compact)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn access_compact_string(compact: CompactString) {
    black_box(black_box(&compact).as_str());
}

#[library_benchmark(setup = setup_smol)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn access_smol_str(smol: SmolStr) {
    black_box(black_box(&smol).as_str());
}

#[library_benchmark(setup = setup_smart)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn access_smart_string(smart: SmartString) {
    black_box(black_box(&smart).as_str());
}

#[library_benchmark(setup = setup_string)]
#[bench::len_0(0)]
#[bench::len_11(11)]
#[bench::len_12(12)]
#[bench::len_22(22)]
#[bench::len_23(23)]
#[bench::len_24(24)]
#[bench::len_25(25)]
#[bench::len_50(50)]
fn access_std_string(string: String) {
    black_box(black_box(&string).as_str());
}

library_benchmark_group!(
    name = string_creation,
    compare_by_id = true,
    benchmarks = [
        creation_compact_string,
        creation_smol_str,
        creation_smart_string,
        creation_std_string,
    ]
);

library_benchmark_group!(
    name = string_cloning,
    compare_by_id = true,
    benchmarks = [
        cloning_compact_string,
        cloning_smol_str,
        cloning_smart_string,
        cloning_std_string,
    ]
);

library_benchmark_group!(
    name = string_access,
    compare_by_id = true,
    benchmarks = [
        access_compact_string,
        access_smol_str,
        access_smart_string,
        access_std_string,
    ]
);

main!(
    library_benchmark_groups = string_creation,
    string_cloning,
    string_access
);
