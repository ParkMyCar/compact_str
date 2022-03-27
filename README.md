<div align="center">
  <h1><code>compact_str</code></h1>
  <p><strong>A memory efficient string type that can store up to 24* bytes on the stack.</strong></p>

  <a href="https://crates.io/crates/compact_str">
    <img alt="version on crates.io" src="https://img.shields.io/crates/v/compact_str"/>
  </a>
  <img alt="Minimum supported Rust Version: 1.49" src="https://img.shields.io/badge/MSRV-1.49-blueviolet">
  <a href="LICENSE">
    <img alt="mit license" src="https://img.shields.io/crates/l/compact_str"/>
  </a>

   <br />

  <a href="https://github.com/ParkMyCar/compact_str/actions/workflows/ci.yml">
    <img alt="Continuous Integration Status" src="https://github.com/ParkMyCar/compact_str/actions/workflows/ci.yml/badge.svg?event=push"/>
  </a>
  <a href="https://github.com/ParkMyCar/compact_str/actions/workflows/cross_platform.yml">
    <img alt="Cross Platform Status" src="https://github.com/ParkMyCar/compact_str/actions/workflows/cross_platform.yml/badge.svg?event=push"/>
  </a>
    <a href="https://github.com/ParkMyCar/compact_str/actions/workflows/msrv.yml">
    <img alt="Minimum Supported Rust Version Status" src="https://github.com/ParkMyCar/compact_str/actions/workflows/msrv.yml/badge.svg?event=push"/>
  </a>
  </a>
    <a href="https://github.com/ParkMyCar/compact_str/actions/workflows/clippy.yml">
    <img alt="Clippy Status" src="https://github.com/ParkMyCar/compact_str/actions/workflows/clippy.yml/badge.svg?event=push"/>
  </a>

  <p  align=right><sub>* 12 bytes for 32-bit architectures</sub></p>
</div>

<br />

### About
A `CompactStr` is a more memory efficient string type, that can store smaller strings on the stack, and transparently stores longer strings on the heap (aka a small string optimization).
They can mostly be used as a drop in replacement for `String` and are particularly useful in parsing, deserializing, or any other application where you may
have smaller strings.

### Properties
A `CompactStr` specifically has the following properties:
  * `size_of::<CompactStr>() == size_of::<String>()`
  * Stores up to 24 bytes on the stack
    * 12 bytes if running on a 32 bit architecture
  * Strings longer than 24 bytes are stored on the heap
  * `Clone` is `O(n)`
  * Conversion `From<String>` or `From<Box<str>>` is `O(1)`
  * Heap based string grows at a rate of 1.5x
    * The std library `String` grows at a rate of 2x

### Features
`compact_str` has the following features:
1. `serde`, which implements [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) and [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) from the popular [`serde`](https://docs.rs/serde/latest/serde/) crate, for `CompactStr`.
2. `bytes`, which provides two methods `from_utf8_buf<B: Buf>(buf: &mut B)` and `from_utf8_buf_unchecked<B: Buf>(buf: &mut B)`, which allows for the creation of a `CompactStr` from a [`bytes::Buf`](https://docs.rs/bytes/latest/bytes/trait.Buf.html)

### How it works
Note: this explanation assumes a 64-bit architecture, for 32-bit architectures generally divide any number by 2.

Normally strings are stored on the heap since they're dynamically sized. In Rust a `String` consists of three fields, each of which are the size of a `usize`.
e.g. its layout is something like the following:

`String: [ ptr<8> | len<8> | cap<8> ]`
1. `ptr` is a pointer to a location on the heap that stores the string
2. `len` is the length of the string
3. `cap` is the total capacity of the buffer being pointed to

This results in 24 bytes being stored on the stack, 8 bytes for each field. Then the actual string is stored on the heap, usually with additional memory allocated to prevent re-allocating if the string is mutated.

The idea of `CompactStr` is instead of storing metadata on the stack, just store the string itself. This way for smaller strings we save a bit of memory, and we
don't have to heap allocate so it's more performant. A `CompactStr` is limited to 24 bytes (aka `size_of::<String>()`) so it won't ever use more memory than a
`String` would.

The memory layout of a `CompactStr` looks something like:

`CompactStr: [ buffer<23> | len<1> ]`

#### Memory Layout
Internally a `CompactStr` has two variants:
1. **Inline**, a string <= 24 bytes long
2. **Heap** allocated, a string > 24 bytes long

To maximize memory usage, we use a [`union`](https://doc.rust-lang.org/reference/items/unions.html) instead of an `enum`. In Rust an `enum` requires at least 1 byte
for the discriminant (tracking what variant we are), instead we use a `union` which allows us to manually define the discriminant. `CompactStr` defines the
discriminant *within* the last byte, using any extra bits for metadata. Specifically the discriminant has two variants:

1. `0b11111111` - All 1s, indicates **heap** allocated
2. `0b11XXXXXX` - Two leading 1s, indicates **inline**, with the trailing 6 bits used to store the length

and specifically the overall memory layout of a `CompactStr` is:

1. `heap:   { string: BoxString, _padding: [u8; 8] }`
2. `inline: { buffer: [u8; 24] }`

<sub>Both variants are 24 bytes long</sub>

For **heap** allocated strings we use a custom `BoxString` which is only 16 bytes on the stack, i.e. `[ ptr<8> | len<8> ]`, and has a heap layout of `[ cap<8> | buf<...> ]`. We then add 8 bytes of padding on the stack to make the stack size equal to our inline variant.

For **inline** strings we only have a 24 byte buffer on the stack. This might make you wonder how can we store a 24 byte long string, inline? Don't we also need to store the length somewhere?

To do this, we utilize the fact that the last byte of our string could only ever have a value in the range `[0, 192)`. We know this because all strings in Rust are valid [UTF-8](https://en.wikipedia.org/wiki/UTF-8), and the only valid byte pattern for the last byte of a UTF-8 character (and thus the possible last byte of a string) is `0b0XXXXXXX` aka `[0, 128)` or `0b10XXXXXX` aka `[128, 192)`. This leaves all values in `[192, 255]` as unused in our last byte. Therefore, we can use values in the range of `[192, 215]` to represent a length in the range of `[0, 23]`, and if our last byte has a value `< 192`, we know that's a UTF-8 character, and can interpret the length of our string as `24`.

Specifically, the last byte on the stack for a `CompactStr` has the following uses:
* `[0, 192)` - Is the last byte of a UTF-8 char, the `CompactStr` is stored on the stack and implicitly has a length of `24`
* `[192, 215]` - Denotes a length in the range of `[0, 23]`, this `CompactStr` is stored on the stack.
* `[215, 255)` - Unused
* `255` - Denotes this `CompactStr` is stored on the heap

### Testing
Strings and unicode can be quite messy, even further, we're working with things at the bit level. `compact_str` has an _extensive_ test suite comprised of unit testing, property testing, and fuzz testing, to ensure our invariants are upheld. We test across all major OSes (Windows, macOS, and Linux), architectures (64-bit and 32-bit), and endian-ness (big endian and little endian).

Fuzz testing is run with `libFuzzer` _and_ `AFL++` with `AFL++` running on both `x86_64` and `ARMv7` architectures. We test with [`miri`](https://github.com/rust-lang/miri) to catch cases of undefined behavior, and run all tests on every rust compiler since `v1.49` to ensure support for our minimum supported Rust version (MSRV).

### `unsafe` code
`CompactStr` uses a bit of unsafe code because accessing fields from a `union` is inherently unsafe, the compiler can't guarantee what value is actually stored.
We also have some manually implemented heap data structures, i.e. `BoxString`, and mess with bytes at a bit level.
That being said, uses of unsafe code in this library are quite limited and constrained to only where absolutely necessary, and always documented with
`// SAFETY: <reason>`.

### Similar Crates
Storing strings on the stack is not a new idea, in fact there are a few other crates in the Rust ecosystem that do similar things, an incomplete list:
1. [`smol_str`](https://crates.io/crates/smol_str) - Can inline 22 bytes, `Clone` is `O(1)`, doesn't adjust for 32-bit archs
2. [`smartstring`](https://crates.io/crates/smartstring) - Can inline 23 bytes, `Clone` is `O(n)`, is mutable, relies on the memory layout of `String`

<br />
Thanks for readingme!
