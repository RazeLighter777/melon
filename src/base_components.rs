use serde::{Deserialize, Serialize};

use crate::{component, entity_id, hook, query, world};

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

struct ChildEntityAddComponentHook {}

fn changed_parent_hook(change: &query::Change, w: &mut world::World) -> Vec<query::Change> {
    match change {
        query::Change(comp2, query::ChangeType::RemoveComponent) => {
            if let Some(comp) = comp2.get::<Children>() {
                let mut changes = Vec::new();
                for child in comp.entities.iter() {
                    w.get_all_components_of_entity(*child)
                        .iter()
                        .for_each(|comp| {
                            for c in comp.iter() {
                                changes.push(query::Change(
                                    c.clone(),
                                    query::ChangeType::RemoveComponent,
                                ));
                            }
                        });
                }
                changes
            } else {
                Vec::new()
            }
        }
        query::Change(comp, query::ChangeType::AddComponent) => {
            if let Some(comp) = comp.get::<Children>() {
                let mut changes = Vec::new();
                for child in comp.entities.iter() {
                    changes.push(query::Change::AddComponent());
                }
                changes
            } else {
                Vec::new()
            }
        }
        query::Change(comp, query::ChangeType::UpdateComponent) => {
            if let Some(comp) = comp.get::<Children>() {
                let mut changes = Vec::new();
                for child in comp.entities.iter() {
                    changes.push(query::Change::AddComponent());
                }
                changes
            } else {
                Vec::new()
            }
        }
        query::Change::RemoveEntity(_) => todo!(),
    }
}
