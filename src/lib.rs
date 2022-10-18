#![feature(const_type_name)]
#![feature(associated_type_bounds)]
#![feature(test)]
extern crate test;
pub mod archetype;
pub mod base_components;
pub mod commands;
pub mod component;
pub mod default_world;
pub mod entity_builder;
pub mod entity_id;
pub mod hashing;
pub mod hook;
pub mod lore;
pub mod position_map;
pub mod query;
pub mod resource;
pub mod stage;
pub mod system;
pub mod world;
use mimalloc::MiMalloc;
use sled;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
