use crate::{
    base_components,
    entity_id::{self, EntityId},
    query, resource, world,
};
use hashbrown::HashMap;
use rtree_rs;
pub struct PositionMap {
    entity_to_position: HashMap<entity_id::EntityId, [i32; 2]>,
    map: rtree_rs::RTree<2, i32, EntityId>,
}

impl PositionMap {
    pub fn new() -> Self {
        let rtree = rtree_rs::RTree::new();
        PositionMap {
            entity_to_position: HashMap::new(),
            map: rtree,
        }
    }
    fn insert(&mut self, id: entity_id::EntityId, position: [i32; 2]) {
        self.map.insert(rtree_rs::Rect::new_point(position), id);
        self.entity_to_position.insert(id, position);
    }
    fn update(&mut self, id: entity_id::EntityId, position: [i32; 2]) {
        if let Some(old_position) = self.entity_to_position.get(&id) {
            self.map
                .remove(rtree_rs::Rect::new_point(*old_position), &id);
            self.map.insert(rtree_rs::Rect::new_point(position), id);
            self.entity_to_position.insert(id, position);
        } else {
            self.insert(id, position);
        }
    }
    fn remove(&mut self, id: entity_id::EntityId) {
        let position = self.entity_to_position.get(&id).unwrap();
        self.map.remove(rtree_rs::Rect::new_point(*position), &id);
        self.entity_to_position.remove(&id);
    }
    pub fn get_nearest(
        &self,
        position: [i32; 2],
        n: usize,
    ) -> Vec<(entity_id::EntityId, [i32; 2])> {
        let nearest = self.map.nearby(|rect, _p| {
            //euclidean distances
            let min = position;
            let max = rect.max;
            //distance formula
            //println!("distance {:?}", rect.max);
            (((max[0] - min[0]) * (max[0] - min[0]) + (max[1] - min[1]) * (max[1] - min[1])) as f32)
                .sqrt() as i32
        });
        let plain = nearest.map(|x| (*x.data, x.rect.max));
        plain.take(n).collect()
    }
}

impl resource::Resource for PositionMap {}

pub fn position_hook(change: &query::Change, world: &world::World) -> Vec<query::Change> {
    //println!("size {}", position_map.map.lock().unwrap().len());
    match change {
        query::Change(comp, query::ChangeType::AddComponent) => {
            if let Some(position) = comp.get::<base_components::Position>() {
                world
                    .write_resource(|position_map: &mut PositionMap| {
                        position_map.insert(
                            comp.id().entity_id(),
                            [position.x, position.y],
                        );
                    })
                    .expect("position map not found");
            }
        }
        query::Change(
            comp,
            query::ChangeType::RemoveComponent | query::ChangeType::UnloadComponent,
        ) => {
            world
                .write_resource(|position_map: &mut PositionMap| {
                    position_map.remove(comp.id().entity_id());
                })
                .expect("position map not found");
        }
        query::Change(comp, query::ChangeType::UpdateComponent) => {
            if let Some(position) = comp.get::<base_components::Position>() {
                world
                    .write_resource(|position_map: &mut PositionMap| {
                        position_map.update(
                            comp.id().entity_id(),
                            [position.x, position.y],
                        );
                    })
                    .expect("position map not found");
            }
        }
    }
    Vec::new()
}

impl Default for PositionMap {
    fn default() -> Self {
        Self::new()
    }
}
