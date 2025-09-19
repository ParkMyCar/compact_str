use bevy_ecs::component::Component;
use bevy_ecs::prelude::Entity;
use bevy_ecs::world::World;
use bevy_reflect::Reflect;
use compact_str::CompactString;

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
