use compact_str::CompactStr;
use pb_jelly::Message;
use proto_user::basic::{
    Address,
    User,
};

fn main() {
    let user = User {
        name: CompactStr::new_inline("John"),
        age: 42,
        address: Some(Address {
            street: "432 Park Ave".into(),
            city: "New York City".into(),
        }),
    };
    let bytes = user.serialize_to_vec();

    let roundtrip_user = User::deserialize_from_slice(&bytes).unwrap();
    println!("{:#?}", roundtrip_user);
    assert_eq!(user, roundtrip_user);
}
