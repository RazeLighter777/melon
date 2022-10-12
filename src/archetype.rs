use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Archetype {
    Item,
    Block,
    Civilization,
    Entit
}