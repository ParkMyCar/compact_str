use std::time::{
    Duration,
    Instant,
};

use compact_str::CompactString;
use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

const EMPTY: &str = "";
const SMALL: &str = "small";
const BIG: &str = "This string has thirty-four chars.";
const HUGE: &str = include_str!("../data/moby10b.txt");

macro_rules! benchmarks_simple {
    ($($method:ident),+) => {
        $(
            fn $method(c: &mut Criterion) {
                benchmarks_simple!(@ c $method EMPTY SMALL BIG HUGE);
            }
        )+
    };

    (@ $c:ident $method:ident $($length:ident)+) => {$(
        $c.bench_function(stringify!($method $length), |b| {
            let string = CompactString::new(black_box($length));
            b.iter(|| {
                let _ = black_box(black_box(&string).$method());
            })
        });
    )+}
}

macro_rules! benchmarks_complex {
    ($($method:ident [$expr:expr])+) => {
        $(
            fn $method(c: &mut Criterion) {
                benchmarks_complex!(@ c $method [$expr] EMPTY SMALL BIG HUGE);
            }
        )+
    };

    (@ $c:ident $method:ident [$expr:expr] $($length:ident)+) => {$(
        $c.bench_function(stringify!($method $length), |b| {
            b.iter_custom(|iters| {
                let mut duration = Duration::default();
                for _ in 0..iters {
                    let mut string = CompactString::new(black_box($length));
                    let start = Instant::now();
                    $expr(black_box(&mut string));
                    duration += start.elapsed();
                    drop(string);
                }
                duration
            });
        });

        $c.bench_function(stringify!($method $length repeated), |b| {
            let mut string = CompactString::new(black_box($length));
            b.iter(|| {
                $expr(black_box(&mut string));
            });
        });
    )+}
}

benchmarks_simple!(as_bytes, as_str, capacity, is_empty, is_heap_allocated, len);

benchmarks_complex! {
    as_mut_bytes [|s: &mut CompactString| { let _ = black_box(unsafe { s.as_mut_bytes() }); }]
    as_mut_ptr [|s: &mut CompactString| { let _ = black_box(s.as_mut_ptr()); }]
    as_mut_str [|s: &mut CompactString| { let _ = black_box(s.as_mut_str()); }]
}

criterion_group! {
    micro,
    as_bytes, as_str, capacity, is_empty, is_heap_allocated, len,
    as_mut_bytes, as_mut_ptr, as_mut_str,
}

criterion_main!(micro);
