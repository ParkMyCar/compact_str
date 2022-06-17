use compact_str::CompactString;

fn test_compact_str(word: &str) {
    let _ = iai::black_box(CompactString::new(iai::black_box(word)));
}

fn test_std_string(word: &str) {
    let _ = iai::black_box(String::from(iai::black_box(word)));
}

fn compact_str_new_0() {
    test_compact_str("");
}

fn compact_str_new_16() {
    test_compact_str("im sixteen chars");
}

fn compact_str_new_24() {
    test_compact_str("i am twenty four chars!!");
}

fn compact_str_new_59() {
    test_compact_str("I am a very long string that will get allocated on the heap");
}

fn std_str_new_0() {
    test_std_string("");
}

fn std_str_new_59() {
    test_std_string("I am a very long string that will get allocated on the heap");
}

#[cfg(iai)]
iai::main!(
    compact_str_new_0,
    compact_str_new_16,
    compact_str_new_24,
    compact_str_new_59,
    std_str_new_0,
    std_str_new_59,
);

#[cfg(not(iai))]
fn main() {
    compact_str_new_0();
    compact_str_new_16();
    compact_str_new_24();
    compact_str_new_59();
    std_str_new_0();
    std_str_new_59();
}
