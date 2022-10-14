#![feature(const_type_name)]
#![feature(associated_type_bounds)]
#![feature(test)]
extern crate test;
pub mod archetype;
pub mod base_components;
pub mod base_resources;
pub mod commands;
pub mod component;
pub mod default_world;
pub mod entity_builder;
pub mod entity_id;
pub mod hashing;
pub mod lore;
pub mod query;
pub mod resource;
pub mod stage;
pub mod system;
pub mod world;
use mimalloc::MiMalloc;
use sled;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(test)]
mod tests {
    use test::Bencher;

    use super::*;
    #[test]
    fn it_works() -> Result<(), sled::Error> {
        let r = sled::open("./test.db")?;
        r.insert(
            entity_id::EntityId::new().id().to_ne_bytes(),
            &64i32.to_ne_bytes(),
        )?;
        Ok(())
    }

    struct TestSystem {}
    impl system::System for TestSystem {
        fn query(&self) -> query::Query {
            query::QueryBuilder::new()
                .with::<base_components::Position>()
                .build()
        }
        fn execute(
            &self,
            query_result: &mut query::QueryResult,
            world: &world::World,
        ) -> commands::Command {
            for e in query_result.iter() {
                println!(
                    "Entity has position {}",
                    e.get_unchecked::<base_components::Position>().x
                );
                e.set::<base_components::Position>(base_components::Position {
                    x: e.get_unchecked::<base_components::Position>().x + 1,
                    y: e.get_unchecked::<base_components::Position>().y + 1,
                });
            }
            commands::Command {}
        }
    }

    #[bench]
    fn query_test(b: &mut Bencher) {
        let mut world = default_world::DefaultWorld::new().build();
        world
            .add_entity()
            .with(base_components::Position { x: 0, y: 0 })
            .spawn();
        world
            .add_entity()
            .with(base_components::Position { x: 0, y: 0 })
            .spawn();
        for i in 0..10000 {
            world
                .add_entity()
                .with(base_components::Position { x: i, y: 0 })
                .with(base_components::Name {
                    name: "test".to_string(),
                })
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

    #[bench]
    fn position_map_test(b: &mut Bencher) {
        let mut world = default_world::DefaultWorld::new().build();
        world
            .add_entity()
            .with(base_components::Position { x: 0, y: 0 })
            .spawn();
        world
            .add_entity()
            .with(base_components::Position { x: 0, y: 0 })
            .spawn();
        for i in 0..10000 {
            world
                .add_entity()
                .with(base_components::Position { x: i, y: 0 })
                .with(base_components::Name {
                    name: "test".to_string(),
                })
                .spawn();
        }
        let stage1 = stage::StageBuilder::new()
            .with_system(TestSystem {})
            .build();
        b.iter(|| {
            world.execute_stage(&stage1);
        })
    }
}
