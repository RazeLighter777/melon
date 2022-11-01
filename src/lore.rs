use std::any::{type_name, Any};

use hashbrown::{HashMap, HashSet};
use serde::Deserialize;
use serde_json::Value;

use crate::{hashing, resource};

pub trait LoreType: Sync + Any + Send + for<'a> Deserialize<'a> {}

pub struct Tags {
    tags: HashSet<String>,
}

impl Tags {
    pub fn new() -> Self {
        Self {
            tags: HashSet::new(),
        }
    }
    pub fn with(mut self, s: &str) -> Self {
        self.tags.insert(s.to_string());
        self
    }
    pub fn with_all(mut self, s: impl IntoIterator<Item = String>) -> Self {
        self.tags.extend(s.into_iter());
        self
    }
    pub fn has(&self, s: &str) -> bool {
        self.tags.contains(s)
    }
    pub fn read(&self) -> &HashSet<String> {
        &self.tags
    }
    pub fn hash(&self) -> u64 {
        self.tags
            .iter()
            .fold(0, |x, y| x.wrapping_add(hashing::string_hash(y)))
    }
}

impl Default for Tags {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum LoreError {
    EntryNotFound(String),
    TypeNotRegistered(String),
    EntryAlreadyExists(String),
    InvalidLoreEntry(String),
    LoreMissingTag(String),
    IOError(std::io::Error),
    JSONError(serde_json::Error),
}
struct LoreEntry {
    data: Box<dyn Any + Sync + Send>,
}

pub struct Lorebook {
    entries: HashMap<u64, LoreEntry>,
    tables: HashMap<String, HashSet<u64>>,
}

impl resource::Resource for Lorebook {}

type LoreEntryDeserializer =  fn(serde_json::Value) -> Result<LoreEntry, LoreError>;

pub struct LorebookBuilder {
    lorebook: Lorebook,
    types: HashMap<u64, LoreEntryDeserializer>,
}

#[derive(Deserialize)]
pub(crate) struct BasicLoreEntry {
    pub tags: Vec<String>,
    pub tp: String,
    pub merge: Option<Vec<String>>,
}

fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

impl LorebookBuilder {
    pub fn new() -> Self {
        Self {
            lorebook: Lorebook {
                entries: HashMap::new(),
                tables: HashMap::new(),
            },
            types: HashMap::new(),
        }
    }
    pub fn register<T: LoreType>(mut self) -> Self {
        let x = type_name::<T>();
        println!("{}", x);
        self.types.insert(hashing::string_hash(x), |v| {
            let data = serde_json::from_value::<T>(v)
                .map_err(|x| LoreError::InvalidLoreEntry(x.to_string()))?;
            Ok(LoreEntry {
                data: Box::new(data),
            })
        });
        self
    }

    fn insert_lorebook_entry(&mut self, entry: LoreEntry, tags: &Tags) -> Result<(), LoreError> {
        let hash = tags.hash();
        if self.lorebook.entries.contains_key(&hash) {
            return Err(LoreError::EntryAlreadyExists(format!(
                "Entry with hash {} already exists",
                hash
            )));
        }
        //add to tag tables
        for tag in &tags.tags {
            self.lorebook
                .tables
                .entry(tag.clone())
                .or_default()
                .insert(hash);
        }
        self.lorebook.entries.insert(hash, entry);
        Ok(())
    }
    pub fn build(mut self, path: &std::path::Path) -> Result<Lorebook, LoreError> {
        //
        if path.is_dir() {
            //loops through every file in directory
            let mut jsons: HashMap<u64, Value> = HashMap::new();
            for entry in std::fs::read_dir(path).map_err(LoreError::IOError)? {
                let entry = entry.map_err(LoreError::IOError)?;
                let path = entry.path();
                if path.is_file() {
                    let file = std::fs::File::open(path).map_err(LoreError::IOError)?;
                    let reader = std::io::BufReader::new(file);
                    let contents: Value =
                        serde_json::from_reader(reader).map_err(LoreError::JSONError)?;
                    let basic_info = serde_json::from_value::<BasicLoreEntry>(contents.clone())
                        .map_err(|x| LoreError::LoreMissingTag(x.to_string()))?;
                    jsons.insert(Tags::new().with_all(basic_info.tags).hash(), contents);
                }
            }
            //for each json file, check for merge tag and merge with other lore entries
            for json in &jsons {
                let basic_info = serde_json::from_value::<BasicLoreEntry>(json.1.clone())
                    .map_err(|x| LoreError::LoreMissingTag(x.to_string()))?;
                //if merge tag is present, merge with other lore entries
                let tags = Tags::new().with_all(basic_info.tags);
                if let Some(merge_tags) = basic_info.merge {
                    let tg = Tags::new().with_all(merge_tags);
                    let t = tg.hash();
                    if let Some(merge_json) = jsons.get(&t) {
                        let mut merged_json = merge_json.clone();
                        merge(&mut merged_json, merge_json);
                        let x = self
                            .types
                            .get(&hashing::string_hash(&basic_info.tp))
                            .ok_or(LoreError::TypeNotRegistered("type".to_string()))?(
                            merged_json
                        )?;
                        self.insert_lorebook_entry(x, &tags)?;
                    }
                } else {
                    let x = self
                        .types
                        .get(&hashing::string_hash(&basic_info.tp))
                        .ok_or(LoreError::TypeNotRegistered("type".to_string()))?(
                        json.1.clone()
                    )?;
                    self.insert_lorebook_entry(x, &tags)?;
                }
            }
        }
        Ok(self.lorebook)
    }
}

impl Lorebook {
    pub fn get<T: LoreType>(&self, tags: Tags) -> Result<&T, LoreError> {
        let i = self
            .entries
            .get(&tags.hash())
            .ok_or(LoreError::EntryNotFound(
                "Could not find entry with tags".to_string(),
            ))?;
        i.data
            .downcast_ref::<T>()
            .ok_or(LoreError::InvalidLoreEntry(
                "Could not downcast entry".to_string(),
            ))
    }
    pub fn get_all_with_tag<T: LoreType>(&self, tag: &Tags) -> Result<Vec<&T>, LoreError> {
        let mut entries: Vec<&T> = Vec::new();
        //find hashset intersection of all tags
        println!("tag: {:?}", tag.tags);
        println!("tables: {:?}", self.tables);
        let mut hashes: Option<HashSet<u64>> = None;
        for t in &tag.tags {
            if let Some(h) = self.tables.get(t) {
                if let Some(hash) = hashes {
                    hashes = Some(hash.intersection(h).fold(HashSet::new(), |x, y| {
                        x.clone().insert(*y);
                        x
                    }));
                } else if let Some(h) = self.tables.get(t) {
                    hashes = Some(h.clone());
                }
            }
        }
        if let Some(h) = hashes {
            for hash in h {
                if let Some(entry) = self.entries.get(&hash) {
                    if let Some(e) = entry.data.downcast_ref::<T>() {
                        entries.push(e);
                    }
                }
            }
        }
        Ok(entries)
    }
}

impl Default for LorebookBuilder {
    fn default() -> Self {
        Self::new()
    }
}