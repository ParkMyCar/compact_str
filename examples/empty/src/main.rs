#![feature(bench_black_box)]

use compact_str::CompactString;

fn main() {
    let input = String::from("hello world my name is parker");
    let empty = CompactString::new(std::hint::black_box(input));
    let s = empty.as_str();
    let l = s.len();
    std::hint::black_box(s);
    std::hint::black_box(l);
}
