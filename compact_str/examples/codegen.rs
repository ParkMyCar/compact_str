//! Codegen inspection harness.
//!
//! Each wrapper is `#[no_mangle] #[inline(never)]` so `cargo asm --example codegen <name>`
//! yields the isolated machine code for that public API as a caller would see it.
#![allow(clippy::missing_safety_doc, improper_ctypes_definitions)]

use compact_str::CompactString;
use std::hash::Hash;

// ─── construction ──────────────────────────────────────────────────────────────
#[no_mangle]
#[inline(never)]
pub fn cs_new(s: &str) -> CompactString {
    CompactString::new(s)
}
#[no_mangle]
#[inline(never)]
pub fn cs_new_short() -> CompactString {
    CompactString::new("hello world")
}
#[no_mangle]
#[inline(never)]
pub fn cs_new_long() -> CompactString {
    CompactString::new("the quick brown fox jumps over the lazy dog")
}
#[no_mangle]
#[inline(never)]
pub fn cs_const_new() -> CompactString {
    CompactString::const_new("the quick brown fox jumps over the lazy dog")
}
#[no_mangle]
#[inline(never)]
pub fn cs_default() -> CompactString {
    CompactString::default()
}
#[no_mangle]
#[inline(never)]
pub fn cs_with_capacity(n: usize) -> CompactString {
    CompactString::with_capacity(n)
}
#[no_mangle]
#[inline(never)]
pub fn cs_from_string(s: String) -> CompactString {
    CompactString::from(s)
}
#[no_mangle]
#[inline(never)]
pub fn cs_from_string_buffer(s: String) -> CompactString {
    CompactString::from_string_buffer(s)
}
#[no_mangle]
#[inline(never)]
pub fn cs_from_str(s: &str) -> CompactString {
    CompactString::from(s)
}
#[no_mangle]
#[inline(never)]
pub fn cs_from_utf8(b: &[u8]) -> Result<CompactString, core::str::Utf8Error> {
    CompactString::from_utf8(b)
}
#[no_mangle]
#[inline(never)]
pub fn cs_from_utf8_lossy(b: &[u8]) -> CompactString {
    CompactString::from_utf8_lossy(b)
}

// ─── read-only access ──────────────────────────────────────────────────────────
#[no_mangle]
#[inline(never)]
pub fn cs_as_str(s: &CompactString) -> &str {
    s.as_str()
}
#[no_mangle]
#[inline(never)]
pub fn cs_as_bytes(s: &CompactString) -> &[u8] {
    s.as_bytes()
}
#[no_mangle]
#[inline(never)]
pub fn cs_len(s: &CompactString) -> usize {
    s.len()
}
#[no_mangle]
#[inline(never)]
pub fn cs_is_empty(s: &CompactString) -> bool {
    s.is_empty()
}
#[no_mangle]
#[inline(never)]
pub fn cs_capacity(s: &CompactString) -> usize {
    s.capacity()
}
#[no_mangle]
#[inline(never)]
pub fn cs_is_heap(s: &CompactString) -> bool {
    s.is_heap_allocated()
}
#[no_mangle]
#[inline(never)]
pub fn cs_as_static(s: &CompactString) -> Option<&'static str> {
    s.as_static_str()
}
#[no_mangle]
#[inline(never)]
pub fn cs_deref(s: &CompactString) -> &str {
    &**s
}
#[no_mangle]
#[inline(never)]
pub fn cs_as_mut_str(s: &mut CompactString) -> &mut str {
    s.as_mut_str()
}

// ─── mutation ──────────────────────────────────────────────────────────────────
#[no_mangle]
#[inline(never)]
pub fn cs_push(s: &mut CompactString, c: char) {
    s.push(c)
}
#[no_mangle]
#[inline(never)]
pub fn cs_push_str(s: &mut CompactString, t: &str) {
    s.push_str(t)
}
#[no_mangle]
#[inline(never)]
pub fn cs_pop(s: &mut CompactString) -> Option<char> {
    s.pop()
}
#[no_mangle]
#[inline(never)]
pub fn cs_insert(s: &mut CompactString, i: usize, c: char) {
    s.insert(i, c)
}
#[no_mangle]
#[inline(never)]
pub fn cs_insert_str(s: &mut CompactString, i: usize, t: &str) {
    s.insert_str(i, t)
}
#[no_mangle]
#[inline(never)]
pub fn cs_remove(s: &mut CompactString, i: usize) -> char {
    s.remove(i)
}
#[no_mangle]
#[inline(never)]
pub fn cs_truncate(s: &mut CompactString, n: usize) {
    s.truncate(n)
}
#[no_mangle]
#[inline(never)]
pub fn cs_clear(s: &mut CompactString) {
    s.clear()
}
#[no_mangle]
#[inline(never)]
pub fn cs_reserve(s: &mut CompactString, n: usize) {
    s.reserve(n)
}
#[no_mangle]
#[inline(never)]
pub fn cs_shrink_to_fit(s: &mut CompactString) {
    s.shrink_to_fit()
}
#[no_mangle]
#[inline(never)]
pub fn cs_split_off(s: &mut CompactString, at: usize) -> CompactString {
    s.split_off(at)
}
#[no_mangle]
#[inline(never)]
pub fn cs_repeat(s: &CompactString, n: usize) -> CompactString {
    s.repeat(n)
}

// ─── traits ────────────────────────────────────────────────────────────────────
#[no_mangle]
#[inline(never)]
pub fn cs_clone(s: &CompactString) -> CompactString {
    s.clone()
}
#[no_mangle]
#[inline(never)]
pub fn cs_drop(s: CompactString) {
    drop(s)
}
#[no_mangle]
#[inline(never)]
pub fn cs_eq_str(s: &CompactString, t: &str) -> bool {
    s == t
}
#[no_mangle]
#[inline(never)]
pub fn cs_eq_self(a: &CompactString, b: &CompactString) -> bool {
    a == b
}
#[no_mangle]
#[inline(never)]
pub fn cs_hash(s: &CompactString, h: &mut std::collections::hash_map::DefaultHasher) {
    s.hash(h)
}
#[no_mangle]
#[inline(never)]
pub fn cs_into_string(s: CompactString) -> String {
    s.into_string()
}
#[no_mangle]
#[inline(never)]
pub fn cs_from_iter_chars(it: std::vec::IntoIter<char>) -> CompactString {
    it.collect()
}
#[no_mangle]
#[inline(never)]
pub fn cs_extend_chars(s: &mut CompactString, it: std::vec::IntoIter<char>) {
    s.extend(it)
}

// ─── realistic composite (what users actually write) ───────────────────────────
#[no_mangle]
#[inline(never)]
pub fn cs_build_key(a: &str, b: u32) -> CompactString {
    let mut s = CompactString::new(a);
    s.push(':');
    s.push_str(itoa::Buffer::new().format(b));
    s
}

fn main() {
    // touch every symbol so the linker keeps them
    let mut s = cs_new("x");
    cs_len(&s);
    cs_is_empty(&s);
    cs_as_str(&s);
    cs_as_bytes(&s);
    cs_capacity(&s);
    cs_is_heap(&s);
    cs_as_static(&s);
    cs_deref(&s);
    cs_as_mut_str(&mut s);
    cs_push(&mut s, 'y');
    cs_push_str(&mut s, "z");
    cs_pop(&mut s);
    cs_insert(&mut s, 0, 'a');
    cs_insert_str(&mut s, 0, "b");
    cs_remove(&mut s, 0);
    cs_truncate(&mut s, 1);
    cs_reserve(&mut s, 10);
    cs_shrink_to_fit(&mut s);
    let _ = cs_split_off(&mut s, 0);
    let _ = cs_repeat(&s, 2);
    let c = cs_clone(&s);
    cs_eq_str(&s, "x");
    cs_eq_self(&s, &c);
    cs_hash(&s, &mut std::collections::hash_map::DefaultHasher::new());
    cs_clear(&mut s);
    cs_drop(c);
    let _ = cs_into_string(cs_default());
    let _ = cs_with_capacity(10);
    let _ = cs_from_string(String::new());
    let _ = cs_from_string_buffer(String::new());
    let _ = cs_from_str("x");
    let _ = cs_from_utf8(b"x");
    let _ = cs_from_utf8_lossy(b"x");
    let _ = cs_const_new();
    let _ = cs_new_short();
    let _ = cs_new_long();
    let _ = cs_from_iter_chars(vec!['a'].into_iter());
    cs_extend_chars(&mut s, vec!['a'].into_iter());
    let _ = cs_build_key("k", 1);
}
