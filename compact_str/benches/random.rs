//! Random benchmarks to determine if one bit of code is faster than another

use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

fn if_statement_min(c: &mut Criterion) {
    let mask = 192;
    let vals: [u8; 4] = [0, 46, 202, 255];
    c.bench_function("if statement min", |b| {
        b.iter(|| {
            for x in vals {
                let len = if x >= mask {
                    (x & !mask) as usize
                } else {
                    24
                };
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

criterion_group!(random, if_statement_min, cmp_min);
criterion_main!(random);
