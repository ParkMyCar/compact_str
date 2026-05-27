# compact_str perf audit

Branch: `perf-audit` · Baseline: `0969dcc7`
Tooling: `asm-harness/` (rlib, `--emit asm`, x86-64-v3 + aarch64) · `scripts/dump_asm.sh` · `scripts/mca.sh` (llvm-mca, sapphirerapids/neoverse-v1) · `scripts/diff_asm.sh` · `bench/benches/perf_audit.rs` (criterion)
µarch reference: `/tmp/cs_asm/cycle_tables.md` (Redwood Cove / Neoverse V2)

## Applied changes (one commit each)

| commit | change | API | x86 ins Δ | criterion Δ | risk |
|---|---|---|---|---|---|
| `2fcf5ec8`→`70d8ac61` | `Repr::new_panic`: skip `Result` for inline arm; keep heap arm inlined (initially `#[cold]`-outlined, reverted after bench showed +39% on heap) | `new`, `From<&str>` | 61→57 | new/11 −2%, new/24 −15% | none |
| `9007adfd` | `as_mut_str`: `[..len]` → `from_raw_parts_mut` (drop dead bounds check) | `as_mut_str`, `DerefMut` | 19→13 | (covered by Drop bench) | none — `len ≤ cap` is invariant |
| `9007adfd` | `with_capacity_panic`: skip `Result` for inline arm (heap arm un-outlined in `70d8ac61`) | `with_capacity` | 28→28 (net 0 after un-outline) | small −48% (Drop dominates), large −6% | none |
| `1918ac2a` | `Capacity::new`: `assert_unchecked(cap & !VALID_MASK == 0)` (64-bit) | `from_string_buffer`, all heap-construct | 24→19 | ~same | none for defined inputs (invariant already required for soundness) |
| `8e2f9630`→`f2cf3236` | `from_inline_ok`: discriminant assert only at `Ok()`-wrapping callers (initially in `from_inline` itself; moved after bench showed new/24 +7% from post-memcpy load) | `from_string`, `from_utf8`, `try_new` | 59→53, 70→66 | (subsumed) | none |
| `846abd8e` | `push_str`: drop bounds check + defer `len()` past `reserve` | `push_str`, `Extend`, `FromIterator` | 54→45 (pre-inline) | ~+1.5% (within noise after as_mut_buf inline) | none — `cap ≥ len+n` after reserve |
| `f2cf3236` | `Drop`: `outlined_drop(ptr, cap)` by-value instead of `&mut Repr` (address no longer escapes → no 24B stack materialization) | `Drop` (every CompactString) | 13→8; arm 18→7 | new/0 −41%, with_capacity/small −48%, clone/inline −17% | none |
| `f2cf3236` | `#[inline]` on `as_mut_buf` + `set_len` (drop PLT-indirect call per push/pop iteration) | all mutators | push_str 45→67 (grows) | (see push_str above) | none — pure inline hint |
| `f2cf3236` | `pop()`: `new_len = chars.as_str().len()` instead of `self.len() − ch.len_utf8()` | `pop` | 65→65 x86, +8 arm | (not benched) | none — same value |
| `ea6355d3` | `#[inline]` on `CompactString::push` (matches `String::push`) | `push` | 1→100 (grows) | (not benched) | none |

## Investigated, not applied

| change | reason |
|---|---|
| `Clone`: invert if/else for tail-call | LLVM still merges through stack temp; +1 ins, +6% mca cyc. |
| `Clone`: `#[cold]` on `clone_heap` | +1 ins on x86. |
| `as_slice`: usize-domain arithmetic to drop redundant `movzx` | flips LLVM cmov→branch; would regress mixed workloads (~17c mispredict). The author's two-separate-ifs pattern is load-bearing. |
| `new`/`with_capacity`: `#[cold] #[inline(never)]` outline of heap arm | criterion: heap-path +39%/+46%. Code-size win (61→25 ins) but runtime loss. Reverted to inline heap arm. |

## Validated benchmark wins (criterion, vs `0969dcc7`)

```
new/0                -41%   (1.65 → 0.97 ns)
new/24               -15%   (3.42 → 2.91 ns)
new/11                -2%
new/59                ~0%
as_str/inline         -8%   (likely measurement variance — as_slice unchanged)
as_str/heap          -10%   (likely measurement variance)
with_capacity/small  -48%   (1.55 → 0.81 ns; Drop dominates)
with_capacity/large   -6%
clone/inline         -17%   (1.96 → 1.63 ns; Drop dominates)
push_str/short→short  +1.5% (within noise)
len, from_string_buffer ~0%
```

## Code-size summary (x86-64-v3 instruction count at call site)

Net **smaller**: `drop` −5, `as_mut_str` −6 (then +16 from as_mut_buf inline), `new`/`from_str` −4, `from_string` −6, `from_string_buffer` −5, `from_utf8` −4
Net **larger** (deliberate inline-for-speed trade): `push` +99, `push_str` +13, `build_key` +54, `extend_chars` +16, `remove` +27

## Remaining workflow proposals (3-vote verified, not yet applied)

| # | API | change | est. impact | caution |
|---|---|---|---|---|
| 2 | `len()` | single `cmp adjusted,24` drives both cmov via signed/unsigned | x86/arm −1 ins | low-risk |
| 3 | `capacity()` | reorder ladder, drop `#[cold]` wrapper | −2 ins inline path | low-risk |
| 4 | `Default` | field-wise `Repr` instead of 24B const memcpy | arm 7→4 ins, drop .rodata load | low-risk |
| 5 | `from_string` → `Repr` (not `Result`) | push unwrap into `#[cold]` 32-bit-only arm | x86 −4, arm −3 | low-risk |
| 6 | `from_string` outline inline-arm | heap-reuse path −14 ins | **bench inline-String case first** (same outlining tradeoff) |
| 14 | `reserve()` split slow path `#[cold]` | fast-path −14 ins | **bench grow path first** |
| 16 | `as_slice` `ensure_read(self.1)` | preserves cmov in PartialEq, eq_str −3-4 ins | low-risk |

## Verification

All commits: `cargo test` (244+79), `cargo +nightly-2026-02-27 miri test` (184+79).
