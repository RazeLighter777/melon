use crate::{entity_id, resource, base_components};
use std::sync::Mutex;
use rstar;
struct PositionMap{
    map: Mutex<rstar::RTree<PositionedEntity>>,
}

struct PositionedEntity {
    id : entity_id::EntityId,
    position : [i32; 2]
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
        dx * dx + dy * dy
    }
}

impl PositionMap {
    pub fn new() -> Self {
        PositionMap {
           map : Mutex::new(rstar::RTree::new())
        }
    }
    pub(crate) fn insert(&self, id : entity_id::EntityId, position : [i32; 2]) {
        let positioned_entity = PositionedEntity {
            id,
            position
        };
        let mut lk = self.map.lock().unwrap();
        lk.insert(positioned_entity);
        lk.locate_at_point(&position);
    }

}

impl resource::Resource for PositionMap {}