# Upcoming
... all released!

# 0.8.0
### July 8, 2024

## Breaking Changes 💥

* Consolidate `CompactString::new_inline(...)` and `CompactString::from_static_str(...)` into `CompactString::const:new(...)`. Methods are currently marked as deprecated and will be removed in `v0.9.0`.
    * Implemented in [`Add const_new(); remove new_inline() and from_static_str()`](https://github.com/ParkMyCar/compact_str/pull/336)

## Changes

* Add support for [`borsh`](https://crates.io/crates/borsh) under an optional feature.
    * Implemented in [`Add borsh support`](https://github.com/ParkMyCar/compact_str/pull/393)
* Add additial `PartialEq` impls for `CompactString`
    * Implemented in [`fix: More PartialEq impls`](https://github.com/ParkMyCar/compact_str/pull/381)
* Match alignment of internal `InlineBuffer` and `Repr`.
    * Implemeneted in [`Align InlineBuffer same as Repr`](https://github.com/ParkMyCar/compact_str/pull/358)
* Fix conflict between `serde` and `no_std` features.
    * Implemented in [`fix serde no-std issue`](https://github.com/ParkMyCar/compact_str/pull/347)
* Improve performance of `CompactString::is_empty`.
    * Implemented in [`Simplify is_empty()`](https://github.com/ParkMyCar/compact_str/pull/330)
* Implement additional `From` impls that `std::string::String` has.
    * Implemented in [`Add missing From impls that String has`](https://github.com/ParkMyCar/compact_str/pull/328)
* Implement [`Clone::clone_from`](https://doc.rust-lang.org/std/clone/trait.Clone.html#method.clone_from) for `CompactString`.
    * Implemented in [`Impl Clone::clone_from for CompactString`](https://github.com/ParkMyCar/compact_str/pull/325)
* Make re-allocations of a heap-based `CompactString` fallible.
    * Implemented in [`Make (re)allocations fallible`](https://github.com/ParkMyCar/compact_str/pull/323)
* Inline short `&'static str`s
    * Implemented in [`Inline short static strings`](https://github.com/ParkMyCar/compact_str/pull/321)
* Add support for serializing a `CompactString` from [`diesel`](https://crates.io/crates/diesel) and [`sqlx`](https://crates.io/crates/sqlx)
    * Implemented in [`Implement diesel compatibility`](https://github.com/ParkMyCar/compact_str/pull/318)
    * Implemented in [`Implement for sqlx`](https://github.com/ParkMyCar/compact_str/pull/329)

... and everything from `v0.8.0-beta`

## Fixed Issues
* Re-enabled specialization for `String` in `trait ToCompactString` by upgrading to `castaway v0.2.3`
    * Implemented in [`deps: Upgrade to castaway v0.2.3`](https://github.com/ParkMyCar/compact_str/pull/394)

# 0.8.0-beta
### October 8, 2023

## Outstanding Issues
* Re-enable specialization for `String` in `trait ToCompactString`, [#304](https://github.com/ParkMyCar/compact_str/issues/304)

## Changes

* Support storing a `&'static str` in a `CompactString` in `O(1)`
    * Implemented in [`feat: Support O(1) CompactString::from_static_str`](https://github.com/ParkMyCar/compact_str/pull/273)
* Support `no_std` environments
    * Implemented in [`support no_std`](https://github.com/ParkMyCar/compact_str/pull/287)
* Add `repeat()` API to `CompactString`
    * Implemented in [`feat: Implement CompactString::repeat`](https://github.com/ParkMyCar/compact_str/pull/275)
* Add `to_ascii_lowercase()`, `to_ascii_uppercase()`, `to_lowercase()`, and `to_uppercase()` APIs to `CompactString`
    * Implemented in [`feat: Implement case conversion fn for CompactString`](https://github.com/ParkMyCar/compact_str/pull/274)
* Add `from_str_to_lower(...)` and `from_str_to_upper(...)` APIs to `CompactString`
    * Implemented in [`feat: Impl CompactString::from_str_to_{lower, upper}case`](https://github.com/ParkMyCar/compact_str/pull/284)
* Improve the performance of the `CompactString::as_str()` API
    * Implemented in [`Remove branches from Repr::as_slice()`](https://github.com/ParkMyCar/compact_str/pull/306)
* Improve the performance of the `CompactString::len()` API
    * Implemented in [`Simplify Repr::len()`](https://github.com/ParkMyCar/compact_str/pull/283)
* Improve the performance of the `Clone` implementation for `CompactString`
    * Implemented in [`Simplify clone()`](https://github.com/ParkMyCar/compact_str/pull/299)
* Improve the performance of some internal types.
    * Implemented in [`Fix and optimize Capacity`](https://github.com/ParkMyCar/compact_str/pull/285)
* Open more niches for enum layout optimizations
    * Implemented in [`Open up more niches`](https://github.com/ParkMyCar/compact_str/pull/276)
* Update MSRV to 1.59
    * Implemented in [`MSRV: Bump to 1.59 `](https://github.com/ParkMyCar/compact_str/pull/296)

# 0.7.1
### June 21, 2023
* Improve the performance of the `ToCompactString` trait
    * Implemented in [`fix: Don't count bytes in ToCompactString`](https://github.com/ParkMyCar/compact_str/pull/270)

# 0.7.0
### February 21, 2023
* Use the `-Zrandomize-layout` `rustc` flag in CI
  * Implemented in [`ci: Randomize Layout in CI`](https://github.com/ParkMyCar/compact_str/pull/266)
* Change `as_ptr()` to require only `&self` and not `&mut self`
  * Implemented in [`refactor: Change CompactString::as_ptr to take &self`](https://github.com/ParkMyCar/compact_str/pull/262)
* Add `into_bytes()` method behind the `smallvec` feature which converts a `CompactString` into a byte vector using a [`SmallVec`](https://docs.rs/smallvec/latest/smallvec/)
  * Implemented in [`api: Add CompactString::into_bytes`](https://github.com/ParkMyCar/compact_str/pull/258)
* Add `from_string_buffer()` method which __always__ re-uses the underlying buffer from `String`
  * Implemented in [`api: Update From<String> and From<Box<str>> to eagerly inline`](https://github.com/ParkMyCar/compact_str/pull/256)
* Eagerly inline strings when `Clone`-ing a `CompactString`
  * Implemented in [`perf: Inline strings when Clone-ing`](https://github.com/ParkMyCar/compact_str/pull/254)
* Updated `From<String>` and `From<Box<str>>` to eagerly inline strings
  * Implemented in [`api: Update From<String> and From<Box<str>> to eagerly inline`](https://github.com/ParkMyCar/compact_str/pull/256)
* Fix a typo in the documentation on `CompactString`
  * Implemented in [`Fix a typo in documentation`](https://github.com/ParkMyCar/compact_str/pull/235/files)
* Implement `AsRef<[u8]>` for `CompactString`
  * Implemented in [`impl AsRef<[u8]> for CompactString`](https://github.com/ParkMyCar/compact_str/pull/230/files)
* Improve the performance of string and length access by using branchless instructions
  * Implemented in [`perf: Refactor underlying buffers for branchless access`](https://github.com/ParkMyCar/compact_str/pull/229)
* Implement `From<CompactString> for Cow<'_, str>`
  * Implemented in [`Implement From<CompactString> for Cow<'_, str>`](https://github.com/ParkMyCar/compact_str/pull/228)
* Improve the performance of `CompactString::new_inline`
  * Implemented in [`Copy inline string reversed`](https://github.com/ParkMyCar/compact_str/pull/219)
* Implement more `FromIterator` and `Extend` traits for `CompactString`
  * Implemented in [`Implement more FromIterator & Extend traits`](https://github.com/ParkMyCar/compact_str/pull/218)
* Add `into_string()` method
  * Implemented in [`Implement more FromIterator & Extend traits`](https://github.com/ParkMyCar/compact_str/pull/218)

# 0.6.1
### August 22, 2022
* Enable the `std` feature in `proptest` to fix documentation on docs.rs
  * Implemented in [`Enable "proptest/std" feature`](https://github.com/ParkMyCar/compact_str/pull/217)

# 0.6.0
### August 21, 2022
* Add `from_utf16_lossy()`, `from_utf16be_lossy()`, and `from_utf16le_lossy()`
    * Implemented in [`feat: implement from_utf16_lossy API`](https://github.com/ParkMyCar/compact_str/pull/211)
    * Implemented in [`Implement from_utf16ne_lossy and from_utf16be_lossy`](https://github.com/ParkMyCar/compact_str/pull/210)
* Add `from_utf16be()` and `from_utf16le()` methods
    * Implemented in [`Implement from_utf16le, from_utf16be, from_utf16ne`](https://github.com/ParkMyCar/compact_str/pull/207)
* Implement `rkyv::Archive`, `rkyv::Deserialize`, and `rkyv::Serialize` for `CompactString`
    * Implemented in [`Add rkyv serialization`](https://github.com/ParkMyCar/compact_str/pull/208)
* Improve performance when counting number of bytes to write into a `CompactString`
    * Implemented in [`Don't format char to determine it's UTF-8 length`](https://github.com/ParkMyCar/compact_str/pull/197)
* Improve macro hygiene for `format_compact!` macro
    * Implemented in [`Macro hygiene: use re-exported format_args!()`](https://github.com/ParkMyCar/compact_str/pull/196)
* Improve performance when a `CompactString` is allocated on the heap
    * Implemented in [`#160 perf: Implement realloc for BoxString`](https://github.com/ParkMyCar/compact_str/pull/160)
* Implement [`markup::Render`](https://docs.rs/markup/latest/markup/trait.Render.html) trait
    * Implemented in [`#157 Implement markup::Render trait and document features`](https://github.com/ParkMyCar/compact_str/pull/157)
* Implement the `Arbitrary` trait from [`arbitrary`](https://docs.rs/arbitrary/latest/arbitrary/), [`proptest`](https://docs.rs/proptest/latest/proptest/), and [`quickcheck`](https://docs.rs/quickcheck/latest/quickcheck/)
    * Implemented in [`146 feat: Implemented the Arbitrary trait from various crate`](https://github.com/ParkMyCar/compact_str/pull/146)
* impl `From<CompactString>` for `String`
    * Implemented in [`#118 feat: add impl From<CompactString> for String`](https://github.com/ParkMyCar/compact_str/pull/118)
* impl `AddAssign` (`+=`) for `CompactString`
    * Implemented in [`add AddAssign operator`](https://github.com/ParkMyCar/compact_str/pull/159)

##### `std::String` API Parity Milestone
* Add `from_utf8_lossy()` method
    * Implemented in [`Implement from_utf8_lossy()`](https://github.com/ParkMyCar/compact_str/pull/198)
* Add `from_utf8_unchecked()` method
    * Implemented in [`feat: from_utf8_unchecked()`](https://github.com/ParkMyCar/compact_str/pull/194)
* Add `retain()` method
    * Implemented in [`Implement retain()`](https://github.com/ParkMyCar/compact_str/pull/193)
* Add `remove()` method
    * Implemented in [`feat: Implement CompactString::remove()`](https://github.com/ParkMyCar/compact_str/pull/191)
* Add `from_utf16()` method
    * Implemented in [`#170 feat: Implement CompactString::from_utf16`](https://github.com/ParkMyCar/compact_str/pull/170)
* Add `split_off()` method
    * Implemented in [`#154 Implement split_off()`](https://github.com/ParkMyCar/compact_str/pull/154)
* Add `drain()` method
    * Implemented in [`#153 Implement drain()`](https://github.com/ParkMyCar/compact_str/pull/153)
* Add `clear()` method
    * Implemented in [`#149 Implement clear()`](https://github.com/ParkMyCar/compact_str/pull/149)
* Add `insert()` and `insert_str()` methods
    * Implemented in [`#148 Implement insert() and insert_str()`](https://github.com/ParkMyCar/compact_str/pull/148)
* Add `truncate()` method
    * Implemented in [`#132 Implement truncate()`](https://github.com/ParkMyCar/compact_str/pull/132)
* Add `replace_range()` method
    * Implemented in [`#125 Implement replace_range()`](https://github.com/ParkMyCar/compact_str/pull/125)
* Add `as_mut_str()` method
    * Implemented in [`#124 Add as_mut_str() method`](https://github.com/ParkMyCar/compact_str/pull/124)

# 0.5.2
### July 24, 2022
* Fix error when creating `CompactString` with capacity `16711422` on 32-bit archiectures
    * Implemented in [`#161 fix: Test case discovered by AFL`](https://github.com/ParkMyCar/compact_str/pull/161)
    * Backported in [`#167 backport(v0.5): Test case discovered by AFL`](https://github.com/ParkMyCar/compact_str/pull/167)

# 0.5.1
### June 27, 2022
* Fix error when importing `compact_str` by change the existing Add<...> impls
    * Implemented in [`#103 fix/feat: Change the existing Add<...> impls`](https://github.com/ParkMyCar/compact_str/pull/103)
    * Backported

# 0.5.0
### June 18, 2022
* Add examples for `CompactStringExt` and `ToCompactString` traits, and `format_compact!(...)` macro
    * Implemented in [`#114 cleanup and examples: Removes bounds check, adds more examples, removes const_panic hack `](https://github.com/ParkMyCar/compact_str/pull/9)
* Remove potential bounds check when converting to &str
    * Implemented in [`#114 cleanup and examples: Removes bounds check, adds more examples, removes const_panic hack `](https://github.com/ParkMyCar/compact_str/pull/9)
    * Implemented in [`#9 Remove potential bounds check from a hot path`](https://github.com/ParkMyCar/compact_str/pull/9)
* Remove `CompactStr` type alias to prep for `v0.5`, as the deprecation message noted
    * Implemented in [`#110 chore: Remove CompactStr type alias`](https://github.com/ParkMyCar/compact_str/pull/110)
* Add `CompactStringExt` which provides methods to join and concatenate collections into a `CompactString`
    * Implemented in [`#109 feat: CompactStringExt trait`](https://github.com/ParkMyCar/compact_str/pull/109)
* Encode `CompactString` in such a way that `size_of::<CompactString>() == size_of::<Option<CompactString>>()`
    * Implemented in [`#105 perf: Option<CompactString> same size as CompactString`](https://github.com/ParkMyCar/compact_str/pull/105)
    * Implemented in [`#75: smol option`](https://github.com/ParkMyCar/compact_str/pull/75)
    * Implemented in [`#22 draft: Optimize Option<CompactStr> to be the same size as CompactStr`](https://github.com/ParkMyCar/compact_str/pull/22)
* Update MSRV to 1.57
* impl `AsRef<OsStr>` for `CompactStr`
    * Implemented in [`#102 Impl AsRef<OsStr> for CompactString`](https://github.com/ParkMyCar/compact_str/pull/102)
* Add `format_compact!` macro
    * Implemented in [`#101 Add macro_rules format_compact!`](https://github.com/ParkMyCar/compact_str/pull/101)

# 0.4.1
### June 27, 2022
* Fix error when importing `compact_str` by change the existing Add<...> impls
    * Implemented in [`#103 fix/feat: Change the existing Add<...> impls`](https://github.com/ParkMyCar/compact_str/pull/103)
    * Backported

# 0.4.0
### May 27, 2022
* Rename `CompactStr` -> `CompactString` and `ToCompactStr` -> `ToComapctString`
    * Implemented in [`#97 refactor: Rename CompactStr to CompactString`](https://github.com/ParkMyCar/compact_str/pull/95)
* Improve performance of `ToCompactStr` by reducing copies for some specialized types
    * Implemented in [`#95 perf: Reduce copies in ToCompactStr for integer types`](https://github.com/ParkMyCar/compact_str/pull/95)
* Introduce the `ToCompactStr` trait, with specialized impls for common types
    * Implemented in [`#16 Add && Impl new trait ToCompactStr`](https://github.com/ParkMyCar/compact_str/pull/16)
* Improve the performance of `From<Cow<'_, str>>`
    * Implemented in [`#90 Optimize From<Cow<'a, str>> impl for CompactStr`](https://github.com/ParkMyCar/compact_str/pull/90)
* impl various `Add<T>` for `CompactStr`, enabling concatination with `+`
    Implemented in [`#81 impl a bunch of Add<T>s for CompactStr, and Add<CompactStr> for String`](https://github.com/ParkMyCar/compact_str/pull/81)
* Improved the performance of `Drop` for inlined strings
    * Implemented in [`#78 perf: Improve the performance of Repr::Drop for Inlined Variants`](https://github.com/ParkMyCar/compact_str/pull/78)
* impl `fmt::Write` for `CompactStr`
    * Implemented in [`#73 Implement fmt::Write for CompactStr`](https://github.com/ParkMyCar/compact_str/pull/73)
* Inline short heap strings when resizing. After enabling `O(1)` conversion from `String` and `Box<str>` it became possible for short strings to be heap allocated. Now if we need to resize a short heap string, we'll inline it, instead of re-heap allocating.
    * Implemented in [`#70 perf: Inline short heap strings when resizing`](https://github.com/ParkMyCar/compact_str/pull/70)

# 0.3.2
### March 27, 2022
* Enable `O(1)` conversion from `String` or `Box<str>` to `CompactStr`
    * Implemented in [`#65 perf: Move Capacity onto the Stack`](https://github.com/ParkMyCar/compact_str/pull/65)
* Update the README to remove references to "immutable". `CompactStr` became mutable with `v0.3.0`

# 0.3.1
### March 6, 2022
* impl `Extend<Cow<str>>` for `CompactStr`
    * Implemented in [`#64 feature: impl Extend<Cow<'_, str>> for CompactStr`](https://github.com/ParkMyCar/compact_str/pull/64)
* impl `From<Cow<str>>` for `CompactStr`
    * Implemented in [`#62 impl From<Cow<'_, str>> for CompactStr`](https://github.com/ParkMyCar/compact_str/pull/62)

# 0.3.0
### February 27, 2022
* Add `CompactStr::from_utf8(...)` API
    * Implemented in [`#57 feature: Add from_utf8 API`](https://github.com/ParkMyCar/compact_str/pull/57)
* Changed the heap variant from an atomically reference counted string, to a normal heap allocated string
    * Implenented in [`#56 feature: BoxString`](https://github.com/ParkMyCar/compact_str/pull/56)
    * Note: This change was made after much deliberation and research into C++ strings and the performance of "copy on write" once mutation is introduced
* Combined the Inline and Packed variants into one variant, store the discriminant in the last byte instead of first
    * Implemented in [`#49 refactor: Combine Inline and Packed Variants`](https://github.com/ParkMyCar/compact_str/pull/49)
    * Note: This simplified the code, and improved the performance of inline string creation and modification
* Removed all required dependencies from `ComapctStr`
    * Implemented in [`#48 vendor: static-assertions`](https://github.com/ParkMyCar/compact_str/pull/48)
* Add more public docs and doc tests for `CompactStr`
    * Implemented in [`#46 chore: Add public documentation to CompactStr`](https://github.com/ParkMyCar/compact_str/pull/46)
* Add `CompactStr::pop(...)`, `CompactStr::push(...)`, and `CompactStr::push_str(...)` APIs
   * Implemented in [`#45 feature: impl the Extend trait for CompactStr`](https://github.com/ParkMyCar/compact_str/pull/40)
* Implement the [`Extend`](https://doc.rust-lang.org/std/iter/trait.Extend.html) trait for `CompactStr`
    * Implemented in [`#45 feature: impl the Extend trait for CompactStr`](https://github.com/ParkMyCar/compact_str/pull/40)
* Add `bytes` feature to `CompactStr`, includes `from_utf8_buf*(...)` APIs
    * Implemented in [`#40 feature: bytes`](https://github.com/ParkMyCar/compact_str/pull/40)
* Add a `CompactStr::as_mut_slice(...)` API
    * Implemented in [`#37 feature: as_mut_slice API`](https://github.com/ParkMyCar/compact_str/pull/37)
* Add a `CompactStr::reserve(...)` API
    * Implemented in [`#36 feature: reserve API`](https://github.com/ParkMyCar/compact_str/pull/36)
* Improve CI, add workflows for MSRV, Miri, All Features, Fuzzing, Docs, and self-hosted ARMv7
    * Implemented in `#26`, `#32`, `34`, `#35`, `#42`, `#56`

# 0.2.0
### November 14, 2021
* Change Minimum Supported Rust Version to 1.49
    * Implemented in [`#24 Make Minimum Supported Rust Version 1.49`](https://github.com/ParkMyCar/compact_str/pull/24)
* Implement `FromIterator` for `CompactStr`
    * Implemented in [`#23 impl FromIterator<...> for CompactStr`](https://github.com/ParkMyCar/compact_str/pull/23)

# 0.1.2
### October 29, 2021
* impl `FromStr` for `CompactStr`
    * Fixes [`#18 Consider impl trait FromStr for CompactStr`](https://github.com/ParkMyCar/compact_str/issues/18)
    * Implemented by [`#20 impl FromStr for CompactStr`](https://github.com/ParkMyCar/compact_str/pull/20)
* Setting minimum supported Rust version (MSRV) to 1.56
    * Fixes [`#3 Document minimal supported Rust Version`](https://github.com/ParkMyCar/compact_str/issues/3)
    * Implemented by [`#17 Upgrade to Edition 2021 and mac MSRV 1.56`](https://github.com/ParkMyCar/compact_str/pull/17)
* Upgrade to Edition 2021
    * [`#17 Upgrade to Edition 2021 and make MSRV 1.56`](https://github.com/ParkMyCar/compact_str/pull/17)

# 0.1.1
### September 30, 2021
* impl `PartialEq` from more types
    * [`#15` Add impl PartialEq<CompactStr> for &str](https://github.com/ParkMyCar/compact_str/pull/15)
* Add missing `#[inline]` and `#[repr(C)]` annotations
    * [`#6` add missing repr(C)](https://github.com/ParkMyCar/compact_str/pull/6)
    * [`#5` add missing inline](https://github.com/ParkMyCar/compact_str/pull/5)
* Fix typos
    * [`#10` missing rename](https://github.com/ParkMyCar/compact_str/pull/10)
    * [`#8` fix typo](https://github.com/ParkMyCar/compact_str/pull/8)
* Avoid future incompatibilities with warnings
    * [`#4` avoid incompatibility with future warnings hazard](https://github.com/ParkMyCar/compact_str/pull/4)

# 0.1
### September 19, 2021
* Initial release!
