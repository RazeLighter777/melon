use crate::resource;
use crate::world;
pub type WorldReferenceWriteClosure = Box<dyn FnOnce(&mut world::World) + Send>;

pub struct ResourceWriter {
    world_reference_closure: Vec<WorldReferenceWriteClosure>,
}

impl ResourceWriter {
    pub fn new() -> Self {
        Self {
            world_reference_closure: Vec::new(),
        }
    }
    pub fn get_resource_writes(self) -> Vec<WorldReferenceWriteClosure> {
        self.world_reference_closure
    }
    pub fn write_resource<R: resource::Resource + 'static, ReturnType>(
        &mut self,
        closure: impl FnOnce(&mut R) -> ReturnType + 'static + Send,
    ) {
        self.world_reference_closure
            .push(Box::new(move |world: &mut world::World| {
                world.write_resource::<R, ReturnType>(closure).unwrap();
            }));
    }
}

impl Default for ResourceWriter {
    fn default() -> Self {
        Self::new()
    }
}
