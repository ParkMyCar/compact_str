use serde::Deserialize;
use smart_str::SmartStr;

#[derive(Debug, Deserialize)]
struct Person {
    name: SmartStr,
    age: u8,
    address: Address,
    phones: Vec<SmartStr>,
}

#[derive(Debug, Deserialize)]
struct Address {
    street: SmartStr,
    city: SmartStr,
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
