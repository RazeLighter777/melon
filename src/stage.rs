use rayon::prelude::IntoParallelRefIterator;

use crate::system;

pub struct Stage {
    systems: Vec<Box<dyn system::System>>,
}

impl Stage {
    pub fn iter(&self) -> rayon::slice::Iter<'_, Box<dyn system::System>> {
        self.systems.par_iter()
    }
}

pub struct StageBuilder {
    systems: Vec<Box<dyn system::System>>,
}

impl StageBuilder {
    pub fn new() -> Self {
        StageBuilder {
            systems: Vec::new(),
        }
    }
    pub fn with_system(mut self, system: impl system::System + 'static) -> Self {
        self.systems.push(Box::new(system));
        self
    }
    pub fn build(self) -> Stage {
        Stage {
            systems: self.systems,
        }
    }
}

impl Default for StageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
