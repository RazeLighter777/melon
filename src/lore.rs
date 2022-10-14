use serde::Deserialize;

trait LoreType: Sync + Send + for<'a> Deserialize<'a> {}
