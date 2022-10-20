use crate::{component, entity_id, query, world};


#[derive(Clone)]
pub struct  ChangeHook {
    closure : fn(&query::Change, & world::World) -> (),
}

impl ChangeHook {
    pub fn new(closure : fn(&query::Change, & world::World)) -> Self {
        Self {closure }
    }
    pub fn execute(&self, change : &query::Change, world : &mut world::World) {
        (self.closure)(change, world);
    }
}



pub trait Unloader : Send + Sync {
    fn hook(&mut self, component: Vec<component::UntypedComponent>, world: &world::World);
}


pub trait Loader : Send + Sync {
    fn hook(&mut self, id: Vec<entity_id::EntityId>, world: &world::World) -> (Vec<query::Change>, Vec<entity_id::EntityId>);
}

pub trait AddComponentHook : Send + Sync {
    fn hook(&self, component: &component::UntypedComponent, world: &mut world::World) -> Vec<query::Change>;
    fn get_component_type_id(&self) -> component::ComponentTypeId;
}
