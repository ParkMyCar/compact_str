# compact_str perf audit — change log

Each row is one logical change, landed as its own commit on `perf-audit`.
Measurements: instruction count from `scripts/dump_asm.sh`, cycles from
`scripts/mca.sh` (llvm-mca, sapphirerapids / neoverse-v1, 100 iter ÷ 100).
Baseline at `/tmp/cs_asm/baseline/`.

| commit | change | API(s) | x86 ins | x86 cyc/call | arm ins | behavior risk |
|---|---|---|---|---|---|---|
| `2fcf5ec8` | split inline path from `Result` in `new()`; outline heap arm to `#[cold]` `new_heap_panic` | `new`, `From<&str>` | 61 → 25 (−59%) | 51.0 → 19.1 (−63%) | 52 → 31 (−40%) | none — same len threshold, same outputs; dropped `len==0` fast path is identical via inline arm |
|  |  | `cs_new_long` (known >24) | 20 → 9 (−55%) |  | 23 → 6 (−74%) |  |
|  |  | `cs_build_key` (composite) | 114 → 86 (−25%) |  | 103 → 86 (−17%) |  |
| `9007adfd` | replace `[..len]` slice with `from_raw_parts_mut` in `as_mut_str` | `as_mut_str`, `DerefMut` | 19 → 13 (−32%) | 15.5 → 11.1 (−28%) | 24 → 16 (−33%) | none — `len ≤ capacity` is a `Repr` invariant |
| `9007adfd` | add `with_capacity_panic` to skip `Result` for inline arm | `with_capacity` | 28 → 22 (−21%) | 22.5 → 16.4 (−27%) | 28 → 27 (−4%) | none — same threshold, same outputs |
| `<next>` | `assert_unchecked(cap & !VALID_MASK == 0)` in `Capacity::new` (64-bit) | `from_string_buffer` | 24 → 19 (−21%) |  | 20 → 12 (−40%) | none for defined inputs — invariant already required for soundness |

## Investigated, not applied

| change | why dropped |
|---|---|
| `Clone`: invert + early-return for tail-call | LLVM still merges through stack temp; +1 ins, +6% cyc. Reverted. |
| `Clone`: `#[cold]` on `clone_heap` | +1 ins on x86. Reverted. |
| `as_slice`: usize arithmetic to drop redundant `movzx` | flips LLVM cmov→branch heuristic, contradicting deliberate branchless design. Would regress mixed inline/heap workloads (~17c mispredict). Reverted. |

## Follow-ups noted

- `Capacity` assert_unchecked relies on `cap < 2^56` which `allocate_ptr` doesn't enforce; consider adding the bound check (separate soundness PR).
- `Clone`/`with_capacity` double-copy through `[rsp]` is a Rust RVO limitation when one if/else arm is an outlined call. Possible fix: tail-call (unstable `become`) or per-arm epilogue.
- `From<String>` inline path still has dead unwrap-check (LLVM can't see through `memcpy` that last byte ≠ 218). Candidate: `assert_unchecked` in `Repr::from_inline`.

## Verification

Every commit: `cargo test` (244 + 79 doc), `cargo +nightly-2026-02-27 miri test` (184 + 79).
