use crate::{query, world};

pub trait System: Send + Sync {
    fn query(&self) -> query::Query;
    fn execute(
        &self,
        query_result: &mut query::QueryResult,
        world: &world::World,
    );
}
