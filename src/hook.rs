use crate::{component, entity_id, query, world};

pub(crate) struct ChangeHook {
    closure: HookLambda,
    component_type: Option<component::ComponentTypeId>,
}
pub type HookLambda = fn(&query::Change, &world::World) -> Vec<query::Change>;

impl ChangeHook {
    pub fn new(closure: HookLambda) -> Self {
        Self {
            closure,
            component_type: None,
        }
    }
    pub fn new_typed<T: component::ComponentType>(closure: HookLambda) -> Self {
        Self {
            closure,
            component_type: Some(component::get_type_id::<T>()),
        }
    }
    pub(crate) fn execute(
        &self,
        change: &query::Change,
        world: &world::World,
    ) -> Vec<query::Change> {
        (self.closure)(change, world)
    }
    pub(crate) fn get_type(&self) -> Option<component::ComponentTypeId> {
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
