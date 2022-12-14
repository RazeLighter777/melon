use melon::*;

#[test]
fn parent() {
    let mut world = default_world::DefaultWorld::get().build();
    let child = world
        .add_entity()
        .with(base_components::Name {
            name: "child".to_string(),
        })
        .spawn();
    let parent = world
        .add_entity()
        .with(base_components::Name {
            name: "parent".to_string(),
        })
        .with(base_components::Children {
            entities: vec![child],
        })
        .spawn();
    assert_eq!(
        world
            .get_component::<base_components::Parent>(child)
            .unwrap()
            .entity,
        parent
    );
    let parent2 = world
        .add_entity()
        .with(base_components::Name {
            name: "parent".to_string(),
        })
        .with(base_components::Children {
            entities: vec![child],
        })
        .spawn();
    assert_eq!(
        world
            .get_component::<base_components::Parent>(child)
            .unwrap()
            .entity,
        parent2
    );
    world.remove_entity(parent2);
    assert_eq!(
        world
            .get_component::<base_components::Parent>(child).is_none(),
        true
    );
    //one entity should be left
    assert_eq!(world.number_of_entities(), 1);
}
