use crate::{
    base_components,
    entity_id::{self, EntityId},
    query, resource, world,
};
use hashbrown::HashMap;
use rtree_rs;
use std::sync::Mutex;
pub struct PositionMap {
    entity_to_position: Mutex<HashMap<entity_id::EntityId, [i32; 2]>>,
    map: Mutex<rtree_rs::RTree<2, i32, EntityId>>,
}

impl PositionMap {
    pub fn new() -> Self {
        let rtree = rtree_rs::RTree::new();
        PositionMap {
            entity_to_position: Mutex::new(HashMap::new()),
            map: Mutex::new(rtree),
        }
    }
    fn insert(&self, id: entity_id::EntityId, position: [i32; 2]) {
        let mut map = self.map.lock().unwrap();
        let mut entity_to_position = self.entity_to_position.lock().unwrap();
        map.insert(rtree_rs::Rect::new_point(position), id);
        entity_to_position.insert(id, position);
    }
    fn update(&self, id: entity_id::EntityId, position: [i32; 2]) {
        let mut map = self.map.lock().unwrap();
        let mut entity_to_position = self.entity_to_position.lock().unwrap();
        if let Some(old_position) = entity_to_position.get(&id) {
            map.remove(rtree_rs::Rect::new_point(*old_position), &id);
            map.insert(rtree_rs::Rect::new_point(position), id);
            entity_to_position.insert(id, position);
        } else {
            drop(entity_to_position);
            drop(map);
            self.insert(id, position);
        }
    }
    fn remove(&self, id: entity_id::EntityId) {
        let mut map = self.map.lock().unwrap();
        let mut entity_to_position = self.entity_to_position.lock().unwrap();
        let position = entity_to_position.get(&id).unwrap();
        map.remove(rtree_rs::Rect::new_point(*position), &id);
        entity_to_position.remove(&id);
    }
    pub fn get_nearest(
        &self,
        position: [i32; 2],
        n: usize,
    ) -> Vec<(entity_id::EntityId, [i32; 2])> {
        let lk = self.map.lock().unwrap();
        let nearest = lk.nearby(|rect, p| {
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

pub fn position_hook(change: &query::Change, world: &mut world::World) {
    let position_map = world
        .get_resource::<PositionMap>()
        .expect("PositionMap not found");
    //println!("size {}", position_map.map.lock().unwrap().len());
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
