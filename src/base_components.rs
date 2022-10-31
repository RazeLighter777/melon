use serde::{Deserialize, Serialize};

use crate::{
    component::{self, ComponentType},
    entity_id, query, world,
};

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

pub fn changed_children_hook(change: &query::Change, w: &world::World) -> Vec<query::Change> {
    match change {
        query::Change(
            comp2,
            removal_type
            @ (query::ChangeType::RemoveComponent | query::ChangeType::UnloadComponent),
        ) => {
            if let Some(comp) = comp2.get::<Children>() {
                let mut changes = Vec::new();
                for child in comp.entities.iter() {
                    w.get_all_components_of_entity(*child)
                        .iter()
                        .for_each(|comp| {
                            for c in comp.iter() {
                                changes.push(query::Change(c.clone(), *removal_type));
                            }
                        });
                }
                changes
            } else {
                Vec::new()
            }
        }
        query::Change(
            comp,
            query::ChangeType::AddComponent | query::ChangeType::UpdateComponent,
        ) => {
            if let Some(comp2) = comp.get::<Children>() {
                let mut changes = Vec::new();
                for child in comp2.entities.iter() {
                    //if the child already has a parent, replace it
                    if let Some(child_parent) = w.get_component_by_instance_id(
                        component::ComponentInstanceId::new::<Parent>(*child),
                    ) {
                        //update the parent of the child
                        changes.push(query::Change(
                            (Parent {
                                entity: comp.get_instance_id().get_entity_id(),
                            })
                            .into_untyped(child_parent.get_instance_id().get_entity_id()),
                            query::ChangeType::UpdateComponent,
                        ));
                    } else {
                        //add the parent to the child
                        changes.push(query::Change(
                            (Parent {
                                entity: comp.get_instance_id().get_entity_id(),
                            })
                            .into_untyped(*child),
                            query::ChangeType::AddComponent,
                        ));
                    }
                }
                changes
            } else {
                Vec::new()
            }
        }
    }
}
