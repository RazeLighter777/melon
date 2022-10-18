use crate::{component, entity_id, query, world};

pub trait ChangeHook {
    fn hook(change: &query::Change, world: &mut world::World);
}

pub trait Unloader {
    fn hook(component: Vec<component::UntypedComponent>, world: &mut world::World);
}

pub trait Loader {
    fn hook(id: Vec<entity_id::EntityId>, world: &mut world::World);
}
