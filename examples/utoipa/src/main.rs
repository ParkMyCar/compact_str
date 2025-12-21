#![allow(dead_code)]

use compact_str::CompactString;
use utoipa::PartialSchema;
use utoipa::ToSchema;

/// CompactString can be used as a drop-in replacement for String in utoipa schemas.
#[derive(ToSchema)]
struct Pet {
    id: u64,
    name: CompactString,
    age: Option<i32>,
}

fn main() {
    // CompactString works seamlessly with utoipa's ToSchema derive
    println!("\nPet schema (using CompactString for 'name' field):");
    println!("{}", serde_json::to_string_pretty(&Pet::schema()).unwrap());
}
