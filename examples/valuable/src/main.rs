use compact_str::CompactString;
use valuable::{Valuable, Value};

/// A `CompactString` can be inspected by any object-safe value visitor via the
/// `valuable::Valuable` trait, presenting itself as a `Value::String`.
fn main() {
    let name = CompactString::from("Ferris");

    // `as_value` yields a borrowed `Value::String`.
    match name.as_value() {
        Value::String(s) => println!("CompactString presents as Value::String({s:?})"),
        other => println!("unexpected value: {other:?}"),
    }

    // It behaves just like a `String`/`&str` would when made `Valuable`.
    assert!(matches!(name.as_value(), Value::String("Ferris")));
}
