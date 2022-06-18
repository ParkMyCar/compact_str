# Upcoming
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
