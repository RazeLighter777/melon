use melon::world::*;
use melon::base_components::*;

#[test]
fn entity_builder_test() {
    let mut world = WorldBuilder::new().build();
    world
        .add_entity()
        .with(Position { x: 0, y: 0 })
        .spawn();
    assert_eq!(world.number_of_entities(), 1);
}

#[test]
fn query_test() {}
