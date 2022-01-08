use std::io::Cursor;

use compact_str::CompactStr;

fn main() {
    let word = "hello world!";

    // Cursor<&[u8]> is `bytes::Buf`
    let mut buf = Cursor::new(word.as_bytes());
    // `from_utf8_buf(...)` can fail, if the provided buffer is not valid UTF-8
    let compact_str = CompactStr::from_utf8_buf(&mut buf).expect("valid utf-8");

    assert_eq!(compact_str, word);

    println!("{}", compact_str);
}
