//! How many strings can you create per second?
//!
//! A port of the benchmark from Daniel Lemire's blog post of the same name, extended to cover
//! [`CompactString`]. See the `README.md` next to this crate for details.
//!
//! Every integer in `0..100_000_000` gets converted into a new string, which is stored into a ring
//! buffer with 1024 slots so the work can't be optimized away. To stay comparable with the numbers
//! from the post, this intentionally mirrors the original methodology, i.e. the best of five runs,
//! instead of using `criterion`.

use std::hint::black_box;
use std::time::Instant;

use compact_str::{format_compact, CompactString, ToCompactString};

const N: u64 = 100_000_000;
const WARM_UP: u64 = 1_000_000;
const RUNS: usize = 5;
const SLOTS: u64 = 1024;

#[inline(never)]
fn std_to_string(buf: &mut [String], n: u64) {
    for i in 0..n {
        buf[(i & (SLOTS - 1)) as usize] = i.to_string();
    }
}

#[inline(never)]
fn itoa_to_string(buf: &mut [String], n: u64) {
    let mut b = itoa::Buffer::new();
    for i in 0..n {
        buf[(i & (SLOTS - 1)) as usize] = b.format(i).to_owned();
    }
}

#[inline(never)]
fn to_compact_string(buf: &mut [CompactString], n: u64) {
    for i in 0..n {
        buf[(i & (SLOTS - 1)) as usize] = i.to_compact_string();
    }
}

#[inline(never)]
fn itoa_to_compact_string(buf: &mut [CompactString], n: u64) {
    let mut b = itoa::Buffer::new();
    for i in 0..n {
        buf[(i & (SLOTS - 1)) as usize] = CompactString::new(b.format(i));
    }
}

#[inline(never)]
fn format_compact(buf: &mut [CompactString], n: u64) {
    for i in 0..n {
        buf[(i & (SLOTS - 1)) as usize] = format_compact!("{i}");
    }
}

fn bench<S: Clone + Default + AsRef<str>>(name: &str, f: fn(&mut [S], u64)) {
    let mut buf = vec![S::default(); SLOTS as usize];
    f(&mut buf, WARM_UP);

    let mut runs = [0.0; RUNS];
    for run in &mut runs {
        let t0 = Instant::now();
        f(black_box(&mut buf), black_box(N));
        *run = t0.elapsed().as_nanos() as f64 / N as f64;
    }
    runs.sort_by(f64::total_cmp);

    // The buffer should be left holding the last `SLOTS` integers.
    for i in (N - SLOTS)..N {
        assert_eq!(buf[(i & (SLOTS - 1)) as usize].as_ref(), i.to_string());
    }
    let total: usize = buf.iter().map(|s| s.as_ref().len()).sum();

    // The post reports the best run. A worst run that's far away from it means the machine is too
    // noisy for the numbers to be trusted.
    let (best, worst) = (runs[0], runs[RUNS - 1]);
    println!(
        "{name:<36} {best:7.2} ns/string  {:8.1} M/s   (worst {worst:.2} ns, check {total})",
        1e3 / best,
    );
}

fn main() {
    println!("Rust String");
    bench("i.to_string()", std_to_string);
    bench("itoa + to_owned()", itoa_to_string);

    println!("Rust CompactString");
    bench("i.to_compact_string()", to_compact_string);
    bench("itoa + CompactString::new()", itoa_to_compact_string);
    bench("format_compact!(\"{i}\")", format_compact);
}
