<div align="center">
  <h1><code>compact_str</code></h1>
  <p><strong>A memory efficient immutable string type that can store up to 24 characters on the stack.</strong></p>

  <img alt="Continuous Integration Status" src="https://github.com/ParkMyCar/compact_str/actions/workflows/ci.yml/badge.svg?event=push"/>
  <img alt="Cross Platform Status" src="https://github.com/ParkMyCar/compact_str/actions/workflows/cross_platform.yml/badge.svg?event=push"/>
</div>

<br />

### About
A `CompactStr` is a more memory efficient string type, that can store smaller strings on the stack, and transparently stores longer strings on the heap. 
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



### How it works
Note: this explanation assumes a 64-bit architecture, for 32-bit architectures generally divide any number by 2.

Normally strings are stored on the heap, since they're dynamically sized. In Rust a `String` consists of three fields, each of which are the size of a `usize`.
e.g. its layout is something like the following:

`String: [ ptr<8> | len<8> | cap<8> ]`
1. `ptr` is a pointer to a location on the heap that stores the string
2. `len` is the length of the string
3. `cap` is the total capacity of the buffer being pointed to

This results in 24 bytes being stored on the stack, 8 bytes for each field. Then the actual string is stored on the heap, usually with additional memory allocated to prevent re-allocating if the string is mutated.

The idea of `CompactStr` is to store strings <= 24 bytes directly on the stack. This way we're never using more memory than a `String` would, and it'll be faster
since generally the stack is faster than the heap. The memory layout of a `CompactStr` looks something like:

`CompactStr: [ len<1> | buffer<23> ]`

##### Memory Layout
