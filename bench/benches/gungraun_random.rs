//! Gungraun (Callgrind) instruction-count benchmarks to determine if one bit of code is faster
//! than another.

use std::hint::black_box;

use gungraun::{library_benchmark, library_benchmark_group, main};

#[library_benchmark]
fn if_statement_min() {
    let mask = 192;
    let vals: [u8; 4] = [0, 46, 202, 255];
    for x in vals {
        let len = if x >= mask { (x & !mask) as usize } else { 24 };
        black_box(len);
    }
}

#[library_benchmark]
fn cmp_min() {
    let mask = 192;
    let vals: [u8; 4] = [0, 46, 202, 255];
    for x in vals {
        let len = core::cmp::min(x.wrapping_sub(mask), 24);
        black_box(len);
    }
}

#[inline(always)]
fn logarithm(val: u32) -> usize {
    ((val as f64).log10().floor()) as usize + 1
}

#[inline(always)]
fn match_statement(val: u32) -> usize {
    match val {
        u32::MIN..=9 => 1,
        10..=99 => 2,
        100..=999 => 3,
        1000..=9999 => 4,
        10000..=99999 => 5,
        100000..=999999 => 6,
        1000000..=9999999 => 7,
        10000000..=99999999 => 8,
        100000000..=999999999 => 9,
        1000000000..=u32::MAX => 10,
    }
}

#[library_benchmark]
#[bench::min(u32::MIN)]
#[bench::half(u32::MAX / 2)]
#[bench::max(u32::MAX)]
fn num_digits_logarithm(val: u32) -> usize {
    black_box(logarithm(black_box(val)))
}

#[library_benchmark]
#[bench::min(u32::MIN)]
#[bench::half(u32::MAX / 2)]
#[bench::max(u32::MAX)]
fn num_digits_match_statement(val: u32) -> usize {
    black_box(match_statement(black_box(val)))
}

library_benchmark_group!(
    name = random,
    benchmarks = [
        if_statement_min,
        cmp_min,
        num_digits_logarithm,
        num_digits_match_statement,
    ]
);

main!(library_benchmark_groups = random);
