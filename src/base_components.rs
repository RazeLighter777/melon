use serde::{Deserialize, Serialize};

use crate::{component, entity_id, hook, world, query};

#[derive(Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
impl component::ComponentType for Position {}

#[derive(Clone, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
}
impl component::ComponentType for Name {}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub player_name: String,
    pub player_id: u64,
}
impl component::ComponentType for Player {}

#[derive(Clone, Serialize, Deserialize)]
pub struct Children {
    pub entities: Vec<entity_id::EntityId>,
}
impl component::ComponentType for Children {}

#[derive(Clone, Serialize, Deserialize)]
pub struct Parent {
    pub entity: entity_id::EntityId,
}
impl component::ComponentType for Parent {}

struct ChildEntityAddComponentHook {

}

impl hook::AddComponentHook for ChildEntityAddComponentHook {
    fn hook(&self, component: &component::UntypedComponent, world: &mut world::World) -> Vec<query::Change> {
        let mut changes = Vec::new();
        if let Some(child_entities) = component.get::<Children>() {
            world.load(child_entities.entities.clone());
            for child_entity in child_entities.entities.iter() {
                changes.push(query::Change::AddComponent(component::UntypedComponent::new(Parent { entity: component.get_instance_id().get_entity_id() }, *child_entity)));
            }
        }
        changes
    }
    fn get_component_type_id(&self) -> component::ComponentTypeId {
        component::get_type_id::<Children>()
    }
}
