<div align="center">
  <h1><code>compact_str</code></h1>
  <p><strong>A memory efficient immutable string type that can store up to 24* bytes on the stack.</strong></p>
  
  <a href="https://github.com/ParkMyCar/compact_str/actions/workflows/ci.yml">
    <img alt="Continuous Integration Status" src="https://github.com/ParkMyCar/compact_str/actions/workflows/ci.yml/badge.svg?event=push"/>
  </a>
  <a href="https://github.com/ParkMyCar/compact_str/actions/workflows/cross_platform.yml">
    <img alt="Cross Platform Status" src="https://github.com/ParkMyCar/compact_str/actions/workflows/cross_platform.yml/badge.svg?event=push"/>
  </a>
  <a href="https://crates.io/crates/compact_str">
    <img alt="version on crates.io" src="https://img.shields.io/crates/v/compact_str"/>
  </a>
  <a href="LICENSE">
    <img alt="mit license" src="https://img.shields.io/crates/l/compact_str"/>
  </a>
  
  <p  align=right><sub>* 12 bytes for 32-bit architectures</sub></p>
</div>

<br />

### About
A `CompactStr` is a more memory efficient immutable string type, that can store smaller strings on the stack, and transparently stores longer strings on the heap. 
They can mostly be used as a drop in replacement for `String` and are particularly useful in parsing, deserializing, or any other application where you may
have smaller strings.

### Properties
A `CompactStr` specifically has the following properties:
  * `size_of::<CompactStr>() == size_of::<String>()`
  * Stores up to 24 bytes on the stack
    * Only up to 23 bytes if the leading character is non-ASCII
    * 12 bytes (or 11 if leading is non-ASCII) if running on a 32 bit architecture
  * Strings longer than 24 bytes are stored on the heap
  * `Clone` is `O(1)`

### Features
`compact_str` has the following features:
1. `serde`, which implements [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) and [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) from the popular [`serde`](https://docs.rs/serde/latest/serde/) crate, for `CompactStr`.

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

`CompactStr: [ len<1> | buffer<23> ]`

#### Memory Layout
Internally a `CompactStr` has three variants:
1. **Heap** allocated, a string >= 24 bytes long
2. **Inline**, a string <= 23 bytes long
3. **Packed**, a string == 24 bytes long and first character is ASCII

To maximize memory usage, we use a [`union`](https://doc.rust-lang.org/reference/items/unions.html) instead of an `enum`. In Rust an `enum` requires at least 1 byte
for the discriminant (tracking what variant we are), instead we use a `union` which allows us to manually define the discriminant. `CompactStr` defines the 
discriminant *within* the first byte, using any extra bits for metadata. Specifically the discriminant has three variants:

1. `0b11111111` - All 1s, indicates **heap** allocated
2. `0b1XXXXXXX` - Leading 1, indicates **inline**, with the trailing 7 bits used to store the length
3. `0b0XXXXXXX` - Leading 0, indicates **packed**, with the trailing 7 bits being the first character of the string

and specifically the overall memory layout of a `CompactStr` is:

1. `heap:   { _padding: [u8; 8], string: Arc<str> }`
2. `inline: { metadata: u8, buffer: [u8; 23] }`
3. `packed: { buffer: [u8; 24] }`

<sub>All variants are 24 bytes long</sub>


For **heap** allocated strings we use an `Arc<str>` which is only 16 bytes, so we prefix it with 8 bytes of padding to make it equal to the other sizes. This 
padding is set to all 1's since it doesn't pertain to the actual string at all, and it allows us to define a unique discriminant. You might be wondering though, how
can we be sure the other two variants will *never* have all 1's as their first byte?
  * The **inline** variant will never have all 1's for it's first byte because we use the trailing 7 bits to store length, all 1's would indicate a length of 127. Our max length is 23, which is < 127, and even on 128-bit architectures we'd only be able to inline 63 bytes, still < our 127 limit.
  * The **packed** variant will never have all 1's for it's first byte because we define the first byte to be ASCII. All strings in Rust use UTF-8 encoding, and UTF-8 encoding does not support Extended ASCII. Meaning, our first character will have a decimal value <= 127, guaranteeing the first bit to always be 0.

### `unsafe` code
`CompactStr` uses a bit of unsafe code because accessing fields from a `union` is inherently unsafe, the compiler can't guarantee what value is actually stored. 
That being said, uses of unsafe code in this library are quite limited and constrained to only where absolutely necessary, and always documented with 
`// SAFETY: <reason>`.

### Testing
Strings and unicode can be quite messy, even further, we're working with things at the bit level. To guard against bugs, `compact_str` uses a mix of unit testing 
for sanity checks and randomized testing for correctness; then automatically runs these tests on 64-bit, 32-bit, big endian, and little endian architectures. 
Further, every 12 hours 1 billion unicode strings are generated and ensured to roundtrip through `CompactStr`, and we assert their location on either the stack or
the heap.

### Similar Crates
Storing strings on the stack is not a new idea, in fact there are a few other crates in the Rust ecosystem that do similar things, an incomplete list:
1. [`smol_str`](https://crates.io/crates/smol_str)
2. [`smartstring`](https://crates.io/crates/smartstring)

<br />
Thanks for readingme!
