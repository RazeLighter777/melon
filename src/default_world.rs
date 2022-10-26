use crate::base_components;
use crate::hook;
use crate::position_map;
use crate::world;

pub struct DefaultWorld {}

impl DefaultWorld {
    pub fn new() -> world::WorldBuilder {
        world::WorldBuilder::new()
            .with_resource(position_map::PositionMap::new())
            .with_hook(hook::ChangeHook::new(position_map::position_hook))
    }
}
