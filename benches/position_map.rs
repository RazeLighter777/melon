#![feature(const_type_name)]
#![feature(associated_type_bounds)]
#![feature(test)]
extern crate test;
use melon::*;
use test::Bencher;

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
        _world: &world::World,
    ) -> commands::Command {
        for e in query_result.iter() {
            e.set::<base_components::Position>(base_components::Position {
                x: e.get_unchecked::<base_components::Position>().x + 1,
                y: e.get_unchecked::<base_components::Position>().y + 1,
            });
        }
        commands::Command {}
    }
}

#[bench]
fn position_map_test(b: &mut Bencher) {
    let mut world = default_world::DefaultWorld::new().build();
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
        world.read_resource(|position_map: &position_map::PositionMap| {
            position_map.get_nearest([0, 0], 10)
        }).unwrap();
        
    })
}
