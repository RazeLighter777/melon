use hashbrown::HashSet;
use serde::Deserialize;

trait LoreType: Sync + Send + for<'a> Deserialize<'a> {}



pub struct Tags {
    tags: HashSet<String>,
}

impl Tags {
    pub fn new() -> Self {
        Self { tags: HashSet::new() }
    }
    pub fn with(mut self, s : &str) -> Self {
        self.tags.insert(s.to_string());
        self
    }
}

struct LoreEntry {
    tags: Tags,
    id : u64,
}