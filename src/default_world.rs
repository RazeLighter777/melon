use crate::base_components;
use crate::base_resources;
use crate::world;

pub struct DefaultWorld {}

impl DefaultWorld {
    pub fn new() -> world::WorldBuilder {
        world::WorldBuilder::new()
            .with_resource(base_resources::PositionMap::new())
            .with_hook(base_resources::position_hook)
    }
}
