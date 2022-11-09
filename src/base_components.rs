use serde::{Deserialize, Serialize};

use crate::{
    component::{self, ComponentType},
    entity_id, query, resource_writer, world,
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

pub(crate) fn changed_children_hook(
    change: &query::Change,
    w: &world::World,
    _: &mut resource_writer::ResourceWriter,
) -> Vec<query::Change> {
    match change {
        query::Change(
            comp2,
            removal_type
            @ (query::ChangeType::RemoveComponent | query::ChangeType::UnloadComponent),
        ) => {
            if let Some(comp) = comp2.get::<Children>() {
                let mut changes = Vec::new();
                for e in comp.entities.iter() {
                    changes.push(query::Change(
                        w.get_component_by_instance_id(
                            component::ComponentInstanceId::new::<Parent>(*e),
                        )
                        .unwrap()
                        .clone(),
                        *removal_type,
                    ));
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
                    //if the child already has a parent, replace it as long as it is incorrect
                    if let Some(child_parent) = w.get_component::<Parent>(*child) && child_parent.entity != comp.entity_id() {
                        changes.push(query::Change(
                            component::UntypedComponent::new(
                                Parent {
                                    entity: comp.id().entity_id(),
                                },
                                *child,
                            ),
                            query::ChangeType::UpdateComponent,
                        ));
                    //if the child doesn't have a parent, add one
                    } else {
                        //add the parent to the child
                        changes.push(query::Change(
                            (Parent {
                                entity: comp.id().entity_id(),
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
