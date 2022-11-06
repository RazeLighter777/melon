use crate::{resource_writer, query, world};

pub trait System: Send + Sync {
    fn query(&self) -> query::Query;
    fn execute(
        &self,
        query_result: &mut query::QueryResult,
        resource_writer: &mut resource_writer::ResourceWriter,
        world: &world::World,
    );
}
