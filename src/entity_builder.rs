use crate::{component, entity_id};

pub trait SpawnLocation {
    fn spawn(&mut self, components: Vec<component::UntypedComponent>);
}
pub struct EntityBuilder<'spawner> {
    spawn_location: &'spawner mut dyn SpawnLocation,
    id: entity_id::EntityId,
    components: Vec<component::UntypedComponent>,
}

impl<'spawner> EntityBuilder<'spawner> {
    pub fn new(spawn_location: &'spawner mut dyn SpawnLocation) -> Self {
        EntityBuilder {
            spawn_location,
            id: entity_id::EntityId::new(),
            components: Vec::new(),
        }
    }
    pub fn with<T: component::ComponentType>(mut self, component: T) -> Self {
        self.components
            .push(component::UntypedComponent::new(component, self.id));
        self
    }
    pub fn spawn(self) -> entity_id::EntityId {
        self.spawn_location.spawn(self.components);
        self.id
    }
}
