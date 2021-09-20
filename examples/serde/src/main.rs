// the fields in `Person` and `Address` are unread, hence the dead code warnings
#![allow(dead_code)]

use compact_str::CompactStr;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Person {
    name: CompactStr,
    age: u8,
    address: Address,
    phones: Vec<CompactStr>,
}

#[derive(Debug, Deserialize)]
struct Address {
    street: CompactStr,
    city: CompactStr,
}

fn main() {
    let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "address": {
                "street": "10 Downing Street",
                "city": "London"
            },
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;

    let person: Person = serde_json::from_str(data).expect("failed to deserialize");
    println!("{:#?}", person);
}
