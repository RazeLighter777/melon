#![feature(const_type_name)]
#![feature(associated_type_bounds)]
#![feature(test)]
extern crate test;
use melon::*;

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
    ) {

        for e in query_result.iter() {
            e.get::<base_components::Position>().unwrap().x += 1;
        }
    }
}

#[test]
fn system() {
    let mut world = default_world::DefaultWorld::get().build();
    let x = world
        .add_entity()
        .with(base_components::Position { x: 0, y: 0 })
        .with(base_components::Name {
            name: "test".to_string(),
        })
        .spawn();
    let stage1 = stage::StageBuilder::new()
        .with_system(TestSystem {})
        .build();
    world.execute_stage(&stage1);
    assert_eq!(world.get_component::<base_components::Position>(x).unwrap().x, 1);
}
