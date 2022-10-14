use std::fmt::Display;

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Hash, Copy)]
pub struct EntityId(u64);

impl Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "entity<{:X}>", self.0)
    }
}

impl EntityId {
    pub fn new_with_number(id: u64) -> Self {
        EntityId(id)
    }
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        EntityId(rng.gen())
    }
    pub fn id(&self) -> u64 {
        self.0
    }
}
