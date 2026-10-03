# How many strings can you create per second?

A port of the benchmark from Daniel Lemire's blog post, [How many strings can you create per
second?][post], extended to cover `CompactString`.

The post converts every integer in `0..100_000_000` into a new string, and stores each one into a
ring buffer with 1024 slots. C++ wins by a wide margin, because none of the strings are longer than
eight bytes, so with the small string optimization `std::string` never has to allocate. Rust's
`String` always does. `CompactString` can inline up to 24 bytes, so how does it compare?

## Running

```sh
./run.sh
```

This builds everything, describes the machine and the toolchains, and then runs each of the
benchmarks pinned to a single CPU (Linux only, pass a CPU number to pick which). All you need is
`cargo`, the C++ baselines are skipped if there is no compiler for them.

This crate is intentionally not a part of the root workspace, so the only things that need to get
downloaded and built are `compact_str`, `itoa`, and the few crates they depend on.

## What gets measured

|                               | allocates | from the post |
| ----------------------------- | :-------: | :-----------: |
| `i.to_string()`               |    yes    |      yes      |
| `itoa` + `to_owned()`         |    yes    |      yes      |
| `i.to_compact_string()`       |    no     |               |
| `itoa` + `CompactString::new` |    no     |               |
| `format_compact!("{i}")`      |    no     |               |
| C++ `std::to_string(i)`       |    no     |      yes      |

The methodology is the same as in [the original][source]: one warm up with a million strings, and
then the best out of five runs, built with a plain `--release`. On top of that we

* print the worst run too, which shows how noisy the machine was.
* check that the ring buffer ends up with the strings we expect.
* run a second time with fat LTO and `codegen-units = 1`. `compact_str` gets compiled as a separate
  crate, whereas `std::to_string` is defined in a header and `String` is largely generic, so both of
  those have an easier time getting inlined.
* run the C++ version with every combination of compiler and standard library that is installed,
  they differ by quite a bit.

## Getting representative numbers

* Only compare numbers that come from the same machine. That's what the baselines are for.
* Prefer bare metal, or at least an instance with dedicated cores. On a virtual machine there is no
  control over frequency scaling, nor over what the neighbors are up to.
* If the best and the worst run are more than a few percent apart, then something else was competing
  for the core. When SMT is active that includes the sibling of the CPU we're pinned to.
* On a hybrid CPU make sure to pick a performance core, the default is likely an efficiency core.
* The `String` rows are mostly a benchmark of `malloc` and `free`, so they depend on the libc, or
  whichever `#[global_allocator]` is used, at least as much as they depend on the CPU.
* This workload is as friendly to a branch predictor as it gets, 90% of the integers have exactly
  eight digits and the length changes just seven times. Expect integers of random lengths to be
  slower across the board.

[post]: https://lemire.me/blog/2026/09/25/how-many-strings-can-you-create-per-second/
[source]: https://github.com/lemire/Code-used-on-Daniel-Lemire-s-blog/tree/master/2026/09/24
