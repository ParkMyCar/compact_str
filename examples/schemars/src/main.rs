#![allow(dead_code)]

use compact_str::CompactString;
use schemars::JsonSchema;

/// CompactString can be used as a drop-in replacement for String in schemars schemas.
#[derive(JsonSchema)]
struct Pet {
    id: u64,
    name: CompactString,
    tags: Vec<CompactString>,
    nickname: Option<CompactString>,
}

fn main() {
    // CompactString works seamlessly with schemars' JsonSchema derive, producing a
    // schema identical to one where the fields used `String`.
    let schema = schemars::schema_for!(Pet);
    println!("Pet schema (using CompactString for string fields):");
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
