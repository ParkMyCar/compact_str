//! Random benchmarks to determine if one bit of code is faster than another

use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    BenchmarkId,
    Criterion,
};

fn if_statement_min(c: &mut Criterion) {
    let mask = 192;
    let vals: [u8; 4] = [0, 46, 202, 255];
    c.bench_function("if statement min", |b| {
        b.iter(|| {
            for x in vals {
                let len = if x >= mask { (x & !mask) as usize } else { 24 };
                black_box(len);
            }
        })
    });
}

fn cmp_min(c: &mut Criterion) {
    let mask = 192;
    let vals: [u8; 4] = [0, 46, 202, 255];
    c.bench_function("cmp min", |b| {
        b.iter(|| {
            for x in vals {
                let len = core::cmp::min(x.wrapping_sub(mask), 24);
                black_box(len);
            }
        })
    });
}

fn num_digits(c: &mut Criterion) {
    const MIN: u32 = u32::MIN;
    const HALF: u32 = (u32::MAX / 2);
    const MAX: u32 = u32::MAX;

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

    c.bench_with_input(BenchmarkId::new("logarithm", "u32::MIN"), &MIN, |b, val| {
        b.iter(|| logarithm(*val))
    });

    c.bench_with_input(
        BenchmarkId::new("logarithm", "u32::MAX / 2"),
        &HALF,
        |b, val| b.iter(|| logarithm(*val)),
    );

    c.bench_with_input(BenchmarkId::new("logarithm", "u32::MAX"), &MAX, |b, val| {
        b.iter(|| logarithm(*val))
    });

    c.bench_with_input(
        BenchmarkId::new("match_statement", "u32::MIN"),
        &MIN,
        |b, val| b.iter(|| match_statement(*val)),
    );

    c.bench_with_input(
        BenchmarkId::new("match_statement", "u32::MAX / 2"),
        &HALF,
        |b, val| b.iter(|| match_statement(*val)),
    );

    c.bench_with_input(
        BenchmarkId::new("match_statement", "u32::MAX"),
        &MAX,
        |b, val| b.iter(|| match_statement(*val)),
    );
}

criterion_group!(random, if_statement_min, cmp_min, num_digits);
criterion_main!(random);
