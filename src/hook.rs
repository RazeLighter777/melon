use crate::{component, entity_id, query, world};

#[derive(Clone)]
pub struct ChangeHook {
    closure: fn(&query::Change, &mut world::World) -> (),
    component_type: Option<component::ComponentTypeId>,
}

impl ChangeHook {
    pub fn new(closure: fn(&query::Change, &mut world::World)) -> Self {
        Self {
            closure,
            component_type: None,
        }
    }
    pub fn new_typed<T: component::ComponentType>(
        closure: fn(&query::Change, &mut world::World),
    ) -> Self {
        Self {
            closure,
            component_type: Some(component::get_type_id::<T>()),
        }
    }
    pub fn execute(&self, change: &query::Change, world: &mut world::World) {
        (self.closure)(change, world);
    }
    pub fn get_type(&self) -> Option<component::ComponentTypeId> {
        self.component_type
    }
}

pub trait Unloader: Send + Sync {
    fn hook(&mut self, component: Vec<component::UntypedComponent>, world: &world::World);
}

pub trait Loader: Send + Sync {
    fn hook(
        &mut self,
        id: Vec<entity_id::EntityId>,
        world: &world::World,
    ) -> (Vec<query::Change>, Vec<entity_id::EntityId>);
}
