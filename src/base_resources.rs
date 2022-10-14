use crate::{base_components, entity_id, query, resource, world};
use hashbrown::HashMap;
use rstar;
use std::sync::Mutex;
pub struct PositionMap {
    entity_to_position: Mutex<HashMap<entity_id::EntityId, [i32; 2]>>,
    map: Mutex<rstar::RTree<PositionedEntity>>,
}

#[derive(PartialEq)]
struct PositionedEntity {
    id: entity_id::EntityId,
    position: [i32; 2],
}
impl rstar::RTreeObject for PositionedEntity {
    type Envelope = rstar::AABB<[i32; 2]>;
    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_point(self.position)
    }
}

impl rstar::PointDistance for PositionedEntity {
    fn distance_2(&self, point: &[i32; 2]) -> i32 {
        let dx = self.position[0] - point[0];
        let dy = self.position[1] - point[1];
        ((dx * dx) as f32 + (dy * dy) as f32).sqrt() as i32
    }
}

impl PositionMap {
    pub fn new() -> Self {
        PositionMap {
            entity_to_position: Mutex::new(HashMap::new()),
            map: Mutex::new(rstar::RTree::new()),
        }
    }
    fn insert(&self, id: entity_id::EntityId, position: [i32; 2]) {
        let positioned_entity = PositionedEntity { id, position };
        let mut lk = self.map.lock().unwrap();
        lk.insert(positioned_entity);
        let mut lk = self.entity_to_position.lock().unwrap();
        lk.insert(id, position);
    }
    fn update(&self, id: entity_id::EntityId, position: [i32; 2]) {
        let positioned_entity = PositionedEntity { id, position };
        let mut lk = self.map.lock().unwrap();
        lk.remove(&positioned_entity);
        lk.insert(positioned_entity);
        let mut lk = self.entity_to_position.lock().unwrap();
        lk.insert(id, position);
    }
    fn remove(&self, id: entity_id::EntityId) {
        let mut lk = self.entity_to_position.lock().unwrap();
        if let Some(x) = lk.remove(&id) {
            let mut lk = self.map.lock().unwrap();
            lk.remove(&PositionedEntity { id, position: x });
        }
    }
    pub fn get_nearest(&self, position: [i32; 2], n: usize) -> Vec<entity_id::EntityId> {
        let lk = self.map.lock().unwrap();
        let nearest = lk.nearest_neighbor_iter(&position);
        let plain = nearest.map(|x| x.id);
        plain.take(n).collect()
    }
}

impl resource::Resource for PositionMap {}

pub fn position_hook(change: &query::Change, world: &mut world::World) {
    let position_map = world
        .get_resource::<PositionMap>()
        .expect("PositionMap not found");
    match change {
        query::Change::AddComponent(comp) => {
            if let Some(position) = comp.get::<base_components::Position>() {
                position_map.insert(
                    comp.get_instance_id().get_entity_id(),
                    [position.x, position.y],
                );
            }
        }
        query::Change::RemoveComponent(comp) => {
            position_map.remove(comp.get_entity_id());
        }
        query::Change::UpdateComponent(comp) => {
            if let Some(position) = comp.get::<base_components::Position>() {
                position_map.update(
                    comp.get_instance_id().get_entity_id(),
                    [position.x, position.y],
                );
            }
        }
        query::Change::RemoveEntity(_) => todo!(),
    }
}
