use bevy_ecs::component::Component;
use bevy_ecs::world::World;
use bevy_reflect::Reflect;
use compact_str::CompactString;
use bevy_ecs::prelude::Entity;

#[derive(Component, Reflect, Debug)]
pub struct Thing(pub CompactString);

pub fn main() {
    let mut world = World::default();

    let thing1 = Thing(CompactString::new("Hello, world!"));
    let thing2 = Thing(CompactString::new("Goodbye, world!"));
    world.spawn(thing1);
    world.spawn(thing2);

    for thing in world.query::<(Entity, &Thing)>().iter(&world) {
        println!("{:?}", thing);
    }
}

// fn main() {
//     let word = "hello world!";

//     // Cursor<&[u8]> is `bytes::Buf`
//     let mut buf = Cursor::new(word.as_bytes());
//     // `from_utf8_buf(...)` can fail, if the provided buffer is not valid UTF-8
//     let compact_str = CompactString::from_utf8_buf(&mut buf).expect("valid utf-8");

//     assert_eq!(compact_str, word);

//     println!("{}", compact_str);
// }
