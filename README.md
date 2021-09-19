# `compact_str`
![CI](https://github.com/ParkMyCar/compact_str/actions/workflows/ci.yml/badge.svg?event=push)
![Cross Platform](https://github.com/ParkMyCar/compact_str/actions/workflows/cross_platform.yml/badge.svg?event=push)

Note: This project is currently in an MVP state. While it works and is tested relatively well, much of the implementation still needs to be cleaned up, documented, and tests refined.

A `CompactStr` is a compact immutable string that is the same size as a `std::string::String`, and can store text on the stack that is up to 24 characters long on 64-bit architectures, or 12 characters long on 32-bit architectures, otherwise the text will get allocated onto the heap.*


\* a `CompactStr` can inline a string up to `size_of::<String>` bytes long if the leading character is ASCII, if the leading character is not ASCII then it will inline up to `size_of::<String> - 1`
