# Upcoming
* Change Minimum Supported Rust Version to 1.49

# 0.1.2
### October 29, 2021
* impl `FromStr` for `CompactStr`
    * Fixes [`#18 Consider impl trait FromStr for CompactStr`](https://github.com/ParkMyCar/compact_str/issues/18)
    * Implemented by [`#20 impl FromStr for CompactStr`](https://github.com/ParkMyCar/compact_str/pull/20)
* Setting minimum supported Rust version (MSRV) to 1.56
    * Fixes [`#3 Document minimal supported Rust Version`](https://github.com/ParkMyCar/compact_str/issues/3)
    * Implemented by [`#17 Upgrade to Edition 2021 and mac MSRV 1.56`](https://github.com/ParkMyCar/compact_str/pull/17)
* Upgrade to Edition 2021
    * [`#17 Upgrade to Edition 2021 and mac MSRV 1.56`](https://github.com/ParkMyCar/compact_str/pull/17)

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
