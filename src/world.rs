use std::sync::{Arc, Mutex, RwLock};

use crate::{
    component, entity_builder,
    entity_id::{self},
    hook::{self, ChangeHook},
    query::{self, Change},
    resource, stage,
};
use hashbrown::{HashMap, HashSet};
use rayon::prelude::*;

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

impl WorldRef {}

#[derive(Clone, Debug)]
pub enum WorldError {
    ResourceNotFound,
    EntityNotFound,
}

pub struct World {
    components: HashMap<component::ComponentInstanceId, component::UntypedComponent>,
    entities: HashMap<entity_id::EntityId, HashSet<component::ComponentInstanceId>>,
    components_types: HashMap<component::ComponentTypeId, HashSet<entity_id::EntityId>>,
    resources: HashMap<u64, Arc<RwLock<resource::UntypedResource>>>,
    //change_trackers: HashMap<component::ComponentTypeId, Vec<component::ComponentInstanceId>>,
    loader: Option<Arc<Mutex<Box<dyn hook::Loader>>>>,
    hooks: Vec<ChangeHook>,
    unloader: Option<Arc<Mutex<Box<dyn hook::Unloader>>>>,
}

impl World {
    pub fn query_world(&self, query: query::Query) -> query::QueryResult {
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
            }
            (Some(first), true) => {
                let first_intersection: HashSet<_> = self
                    .components_types
                    .get(first)
                    .unwrap()
                    .intersection(self.components_types.get(i.next().unwrap()).unwrap())
                    .copied()
                    .collect();
                let matches = i.fold(first_intersection, |x, y| {
                    if let Some(component_type_list) = self.components_types.get(y) {
                        x.par_intersection(component_type_list)
                            .map(|x| *x)
                            .collect()
                    } else {
                        x
                    }
                });
                matches.iter().for_each(|x| {
                    query_result_builder.with_entity(
                        self.entities
                            .get(x)
                            .expect("ECS invarient broken")
                            .iter()
                            .map(|x| self.components.get(x).unwrap().clone()),
                        *x,
                    );
                });
            }
            _ => {}
        }
        query_result_builder.build()
    }

    pub fn new() -> Self {
        World {
            components: HashMap::new(),
            entities: HashMap::new(),
            components_types: HashMap::new(),
            resources: HashMap::new(),
            hooks: Vec::new(),
            loader: None,
            unloader: None,
        }
    }

    pub fn get_component_by_instance_id(
        &self,
        id: component::ComponentInstanceId,
    ) -> Option<&component::UntypedComponent> {
        self.components.get(&id)
    }

    pub fn get_all_components_of_entity(
        &self,
        id: entity_id::EntityId,
    ) -> Option<Vec<component::UntypedComponent>> {
        self.entities.get(&id).map(|x| {
            x.iter()
                .map(|x| self.components.get(x).unwrap().clone())
                .collect()
        })
    }

    pub fn number_of_entities(&self) -> usize {
        self.entities.len()
    }

    pub fn read_resource<R: resource::Resource + 'static, ReturnType>(
        &self,
        closure: impl FnOnce(&R) -> ReturnType,
    ) -> Result<ReturnType, WorldError> {
        if let Some(resource) = self.resources.get(&resource::get_resource_id::<R>()) {
            let resource = resource.read().unwrap();
            Ok(closure(resource.get_as::<R>()))
        } else {
            Err(WorldError::EntityNotFound)
        }
    }

    pub fn write_resource<R: resource::Resource + 'static, ReturnType>(
        &self,
        closure: impl FnOnce(&mut R) -> ReturnType,
    ) -> Result<ReturnType, WorldError> {
        if let Some(resource) = self.resources.get(&resource::get_resource_id::<R>()) {
            let mut resource = resource.write().unwrap();
            Ok(closure(resource.get_as_mut::<R>()))
        } else {
            Err(WorldError::EntityNotFound)
        }
    }

    pub fn execute_stage(&mut self, stage: &stage::Stage) {
        //println!("Executing");
        let changed = stage
            .iter()
            .map(|system| {
                let mut query_result = self.query_world(system.query());
                system.execute(&mut query_result, self);
                query_result.get_changes()
            })
            .flatten()
            .collect::<Vec<_>>();
        self.execute_changes(changed);
    }

    pub fn load(&mut self, id: Vec<entity_id::EntityId>) -> Vec<entity_id::EntityId> {
        let mut loaded = Vec::new();
        if let Some(loader) = &self.loader.clone() {
            if let Ok(mut ld) = loader.lock() {
                let res = ld.hook(id, self);
                self.execute_changes(res.0);
                loaded = res.1;
            }
        }
        loaded
    }

    pub fn get_component<T: component::ComponentType + 'static>(
        &self,
        id: entity_id::EntityId,
    ) -> Option<&T> {
        self.entities
            .get(&id)
            .and_then(|x| x.get(&component::ComponentInstanceId::new::<T>(id)))
            .and_then(|x| self.components.get(x).and_then(|x| x.get::<T>()))
    }

    fn execute_changes(&mut self, changed: impl IntoParallelIterator<Item = Change>) {
        //execute untyped hooks
        let change_map = Mutex::new(HashMap::new());
        changed.into_par_iter().for_each(|x| {
            let mut change_map = change_map.lock().unwrap();
            change_map
                .entry(x.0.get_type())
                .or_insert_with(Vec::new)
                .push(x);
        });
        let changed = change_map.into_inner().unwrap();
        let newchanges = self
            .hooks
            .par_iter()
            .map(|hook| {
                if let Some(ctype) = hook.get_type() {
                    if let Some(changes) = changed.get(&ctype) {
                        changes
                            .par_iter()
                            .map(|x| hook.execute(x, self))
                            .collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    }
                } else {
                    changed
                        .par_iter()
                        .map(|x| x.1)
                        .flatten()
                        .map(|x| hook.execute(x, self))
                        .collect::<Vec<_>>()
                }
            })
            .flatten()
            .flatten()
            .collect::<Vec<_>>();
        if !newchanges.is_empty() {
            self.execute_changes(newchanges);
        }
        changed.iter().flat_map(|x| x.1).for_each(|change| {
            match change {
                query::Change(
                    comp,
                    _removed_or_unloaded @ (query::ChangeType::RemoveComponent
                    | query::ChangeType::UnloadComponent),
                ) => {
                    let tid = &&comp.get_type();
                    let eid = &comp.get_instance_id().get_entity_id();
                    let id = &comp.get_instance_id();
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
                }
                query::Change(comp, query::ChangeType::AddComponent) => {
                    let tid = comp.get_type();
                    //execute add component hooks
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
                }
                query::Change(comp, query::ChangeType::UpdateComponent) => {
                    let id = comp.get_instance_id();
                    self.components.insert(id, comp.clone());
                }
            }
        });
    }

    pub fn add_entity(&mut self) -> entity_builder::EntityBuilder {
        entity_builder::EntityBuilder::new(self)
    }
}

impl entity_builder::SpawnLocation for World {
    fn spawn(&mut self, components: Vec<component::UntypedComponent>) {
        self.execute_changes(
            components
                .into_iter()
                .map(|x| query::Change(x, query::ChangeType::AddComponent))
                .collect::<Vec<_>>(),
        );
    }
}

pub struct WorldBuilder {
    world: World,
}

impl WorldBuilder {
    pub fn new() -> WorldBuilder {
        WorldBuilder {
            world: World::new(),
        }
    }
    pub fn with_resource<R: resource::Resource + 'static>(mut self, resource: R) -> Self {
        self.world.resources.insert(
            resource::get_resource_id::<R>(),
            Arc::new(RwLock::new(resource::UntypedResource::new(resource))),
        );
        self
    }
    pub fn with_loader(mut self, loader: impl hook::Loader + 'static) -> Self {
        self.world.loader = Some(Arc::new(Mutex::new(Box::new(loader))));
        self
    }
    pub fn with_unloader(mut self, unloader: impl hook::Unloader + 'static) -> Self {
        self.world.unloader = Some(Arc::new(Mutex::new(Box::new(unloader))));
        self
    }
    pub fn with_hook(mut self, _hook: hook::HookLambda) -> Self {
        self.world.hooks.push(hook::ChangeHook::new(_hook));
        self
    }
    pub fn with_typed_hook<T: component::ComponentType + 'static>(
        mut self,
        _hook: hook::HookLambda,
    ) -> Self {
        self.world
            .hooks
            .push(hook::ChangeHook::new_typed::<T>(_hook));
        self
    }
    pub fn build(self) -> World {
        self.world
    }
}
impl Default for WorldBuilder {
    fn default() -> Self {
        WorldBuilder::new()
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}