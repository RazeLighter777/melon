use crate::entity_id;
use crate::component;
use crate::resource;
use crate::world;
pub type WorldReferenceWriteClosure = fn(&mut world::World) -> ();


pub struct Command {
    removed_entities: Vec<entity_id::EntityId>,
    removed_components: Vec<component::ComponentInstanceId>,
    unloaded_entities: Vec<component::ComponentInstanceId>,
    unloaded_components: Vec<component::ComponentInstanceId>,
    world_reference_closure : Vec<Box<dyn FnOnce(&mut world::World ) -> ()>>
}

impl Command {
    pub fn new() -> Self {
        Self {
            removed_entities: Vec::new(),
            removed_components: Vec::new(),
            unloaded_entities: Vec::new(),
            unloaded_components: Vec::new(),
            world_reference_closure : Vec::new()
        }
    }
    pub fn remove_entity(&mut self, id: entity_id::EntityId) {
        self.removed_entities.push(id);
    }
    pub fn remove_component(&mut self, id: component::ComponentInstanceId) {
        self.removed_components.push(id);
    }
    pub fn unload_entity(&mut self, id: component::ComponentInstanceId) {
        self.unloaded_entities.push(id);
    }
    pub fn unload_component(&mut self, id: component::ComponentInstanceId) {
        self.unloaded_components.push(id);
    }
    pub fn get_removed_entities(&self) -> &Vec<entity_id::EntityId> {
        &self.removed_entities
    }
    pub fn get_removed_components(&self) -> &Vec<component::ComponentInstanceId> {
        &self.removed_components
    }
    pub fn get_unloaded_entities(&self) -> &Vec<component::ComponentInstanceId> {
        &self.unloaded_entities
    }
    pub fn get_unloaded_components(&self) -> &Vec<component::ComponentInstanceId> {
        &self.unloaded_components
    }

    pub fn write_resource<R: resource::Resource + 'static, ReturnType>(
        &mut self,
        closure: impl FnOnce(&mut R) -> ReturnType + 'static,
    ){
        self.world_reference_closure.push(Box::new(move |world : &mut world::World| {
            world.write_resource::<R, ReturnType>(closure).unwrap();
        }));
        
    }
}

impl Default for Command {
    fn default() -> Self {
        Self::new()
    }
}