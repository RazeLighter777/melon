use crate::base_components;
use crate::position_map;
use crate::world;

pub struct DefaultWorld {}

impl DefaultWorld {
    pub fn get() -> world::WorldBuilder {
        world::WorldBuilder::new()
            .with_resource(position_map::PositionMap::new())
            .with_typed_hook::<base_components::Position>(position_map::position_hook)
            .with_typed_hook::<base_components::Children>(base_components::changed_children_hook)
    }
}
