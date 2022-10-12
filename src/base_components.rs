use serde::{Deserialize, Serialize};

use crate::{component, entity_id};



#[derive(Clone,Serialize, Deserialize)]
pub struct Position {
    pub x : i32,
    pub y : i32,
}
impl component::ComponentType for Position {}


#[derive(Clone,Serialize, Deserialize)]
pub struct Name {
    pub name : String,
}
impl component::ComponentType for Name {}


#[derive(Clone,Serialize, Deserialize)]
pub struct Player {
    pub player_name : String,
    pub player_id : u64,
}
impl component::ComponentType for Player {}
 
#[derive(Clone,Serialize, Deserialize)]
pub struct ChildEntities {
    pub entities : Vec<entity_id::EntityId>,
}
impl component::ComponentType for ChildEntities {}