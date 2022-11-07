#![feature(const_type_name)]
#![feature(associated_type_bounds)]
#![feature(test)]
extern crate test;
use melon::*;
use test::Bencher;

#[bench]
fn insert_test(b: &mut Bencher) {
    b.iter(|| {
        let mut world = default_world::DefaultWorld::get().build();
        for _ in 0..10000 {
            world
                .add_entity()
                .with(base_components::Name {
                    name: "test".to_string(),
                })
                .spawn();
        }
        world.add_entity().spawn();
    })
}
