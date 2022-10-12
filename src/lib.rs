#![feature(const_type_name)]
#![feature(associated_type_bounds)]
#![feature(test)]
extern crate test;
pub mod commands;
pub mod component;
pub mod entity_id;
pub mod hashing;
pub mod stage;
pub mod archetype;
pub mod world;
pub mod base_resources;
pub mod query;
pub mod base_components;
pub mod resource;
pub mod entity_builder;
pub mod system;
#[cfg(test)]
mod tests {
    use test::Bencher;

    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }

    #[bench]
    fn query_test(b : &mut Bencher) {
        let mut world = world::WorldBuilder::new().build();
        world.add_entity()
            .with(base_components::Position { x: 0, y : 0})
            .spawn();
        world.add_entity()
            .with(base_components::Position { x: 0, y : 0})
            .spawn();
        for  i in 0..10000 {
            world.add_entity()
                    .with(base_components::Position { x: 0, y : 0})
                    .with(base_components::Name { name : "test".to_string()})
                    .spawn();
        }
        b.iter(|| {
            let query = query::QueryBuilder::new()
            .with::<base_components::Position>()
            .with::<base_components::Name>()
            .build();
            let res = world.query_world(query);
            res
        })
    }

}
