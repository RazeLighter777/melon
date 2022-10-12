use std::sync::Arc;

use hashbrown::{HashSet, HashMap};
use rayon::prelude::*;
use crate::{component, entity_id::{self}, query::{self, Change}, entity_builder, stage, resource, base_components};

#[derive(Clone)]
pub struct WorldRef {
    reference: Arc<World>,
}

impl std::ops::Deref for WorldRef {
    type Target = Arc<World>;

    fn deref(&self) -> &Arc<World> {
        &self.reference
    }
}

impl WorldRef { 
}

pub struct World {
    components: HashMap<component::ComponentInstanceId, component::UntypedComponent>,
    entities: HashMap<entity_id::EntityId, HashSet<component::ComponentInstanceId>>,
    components_types: HashMap<component::ComponentTypeId, HashSet<entity_id::EntityId>>,
    resources : HashMap<u64, resource::UntypedResource>,
    change_trackers : HashMap<component::ComponentTypeId, Vec<component::ComponentInstanceId>>,
    hooks : Vec<fn(&Change, &mut World) -> ()>,
}

impl World {    
    pub fn query_world(&self, query : query::Query) -> query::QueryResult {
        let mut query_result_builder = query::QueryResultBuilder::new();
        let mut i = query.components.iter();
        match (i.next(), i.len() != 0) {
            (Some(first), false) => {
                
                if let Some(map) = self.components_types.get(first) {
                    map.iter().for_each(|id| {
                        let mut components = Vec::new();
                        if let Some(set) = self.entities.get(id) {
                            set.iter().for_each(|component_id| {
                                if let Some(component) = self.components.get(component_id) {
                                    components.push(component.clone());
                                }
                            });
                        }
                        query_result_builder.with_entity(components, *id);
                    });
                }
            },
            (Some(first), true) => {
                let first_intersection : HashSet<_> = self.components_types.get(first).unwrap().intersection(self.components_types.get(i.next().unwrap()).unwrap()).copied().collect();
                let matches = i.fold(first_intersection, |x, y| {
                    if let Some(component_type_list) = self.components_types.get(y) {
                        x.par_intersection(component_type_list).map(|x| *x).collect()
                    } else {
                        x
                    }
                });
                matches.iter().for_each(|x| {
                    query_result_builder.with_entity(self.entities.get(x).expect("ECS invarient broken").iter().map(|x| self.components.get(x).unwrap().clone()), *x);
                });
            },
            _ => {
                
            }
        }
        query_result_builder.build()
    }

    pub fn new() -> Self {
        World {
            components: HashMap::new(),
            entities: HashMap::new(),
            components_types: HashMap::new(),
            resources : HashMap::new(),
            change_trackers: HashMap::new(),
            hooks: Vec::new(),
        }
    }

    pub fn get_resource<R: resource::Resource + 'static>(&self) -> Option<&R> {
        self.resources.get(&resource::get_resource_id::<R>()).map(|x| x.get_as().unwrap())
    }


    
    pub fn execute_stage(&mut self, stage : &stage::Stage) {
        let changed = stage.iter().map(|system| {
            let mut query_result = self.query_world(system.query());
            system.execute(&mut query_result, self);
            query_result.get_changes()
        }).flatten().collect::<Vec<_>>();
        self.execute_changes(changed);
    }

    fn execute_changes(&mut self, changed: Vec<Change>) {
        changed.iter().for_each(|change| {
            self.hooks.clone().iter().for_each(|hook| {
                hook(change, self);
            });
            match change {
                Change::RemoveComponent(id) => {
                    let tid = &id.get_component_type_id();
                    let eid = &id.get_entity_id();
                    if let Some(set) = self.entities.get_mut(eid) {
                        set.remove(id);
                        if set.is_empty() {
                            self.entities.remove(eid);
                        }
                    }
                    if let Some(set) = self.components_types.get_mut(tid) {
                        set.remove(eid);
                    }
                    self.components.remove(id);
                },
                Change::AddComponent(comp) => {
                    let tid = comp.get_type();
                    let eid = comp.get_instance_id().get_entity_id();
                    let cid = comp.get_instance_id();
                    self.components.insert(cid, comp.clone());
                    if let Some(set) = self.entities.get_mut(&eid) {
                        set.insert(cid);
                    } else {
                        let mut set = HashSet::new();
                        set.insert(cid);
                        self.entities.insert(eid, set);
                    }
                    if let Some(set) = self.components_types.get_mut(&tid) {
                        set.insert(eid);
                    } else {
                        let mut set = HashSet::new();
                        set.insert(eid);
                        self.components_types.insert(tid, set);
                    }
                },
                Change::UpdateComponent(comp) => {
                    let cid = comp.get_instance_id();
                    self.components.insert(cid, comp.clone());
                },
                Change::RemoveEntity(_ent) => {
                    todo!()
                },
            }
        });
    }

    pub fn add_entity(&mut self) -> entity_builder::EntityBuilder {
       entity_builder::EntityBuilder::new(self)
    }
}

impl entity_builder::SpawnLocation for World {
    fn spawn(&mut self, entity_id : entity_id::EntityId, components : Vec<component::UntypedComponent>) {
        for comp in components {
            let component_instance_id = comp.get_instance_id();
            self.components.insert(component_instance_id, comp);
            self.entities.entry(entity_id).or_insert_with(HashSet::new).insert(component_instance_id);
            self.components_types.entry(component_instance_id.get_component_type_id()).or_insert_with(HashSet::new).insert(entity_id);
        }
    }
}

#[test]
fn entity_builder_test() {
    let mut world = WorldBuilder::new().build();
    world.add_entity()
        .with(base_components::Position { x: 0, y : 0})
        .spawn();
    assert_eq!(world.entities.len(), 1);
}

#[test]
fn query_test() {
    
}

pub struct WorldBuilder { 
    world : World,
}

impl WorldBuilder {
    pub fn new() -> WorldBuilder {
        WorldBuilder {
            world : World::new(),
        }
    }
    pub fn with_resource<R : resource::Resource + 'static>(&mut self, resource : R) -> &mut Self {
        self.world.resources.insert(resource::get_resource_id::<R>(), resource::UntypedResource::new(resource));
        self
    }
    pub fn with_hook(&mut self, hook : fn(&Change, &mut World) -> ()) -> &mut Self {
        self.world.hooks.push(hook);
        self
    }
    pub fn build(self) -> World {
        self.world
    }
}