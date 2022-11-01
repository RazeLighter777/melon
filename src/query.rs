use std::ops::{Deref, DerefMut};

use hashbrown::{HashMap, HashSet};

use crate::{
    component::{self, ComponentType, ComponentTypeId, UntypedComponent, TypedComponent},
    entity_id,
};

pub struct Query {
    pub components: HashSet<component::ComponentTypeId>,
}

pub struct QueryBuilder {
    query: Query,
}

impl QueryBuilder {
    pub fn new() -> Self {
        QueryBuilder {
            query: Query {
                components: HashSet::new(),
            },
        }
    }
    pub fn with<T: ComponentType>(mut self) -> Self {
        self.query.components.insert(component::get_type_id::<T>());
        self
    }
    pub fn build(self) -> Query {
        self.query
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
pub struct QueryResult {
    entities: Vec<ComponentGroup>,
}

impl QueryResult {
    pub(crate) fn get_changes(self) -> Vec<Change> {
        self.entities
            .into_iter()
            .flat_map(|x| x.get_changes())
            .collect()
    }
    pub fn iter(&mut self) -> std::slice::IterMut<ComponentGroup> {
        self.entities.iter_mut()
    }
}

pub struct QueryResultBuilder {
    query_result: QueryResult,
}
impl QueryResultBuilder {
    pub fn new() -> Self {
        QueryResultBuilder {
            query_result: QueryResult {
                entities: Vec::new(),
            },
        }
    }
    pub fn with_entity(
        &mut self,
        components: impl IntoIterator<Item = UntypedComponent>,
        id: entity_id::EntityId,
    ) -> &mut Self {
        self.query_result
            .entities
            .push(ComponentGroup::new(id, components));
        self
    }
    pub fn build(self) -> QueryResult {
        self.query_result
    }
}

impl Default for QueryResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ComponentGroup {
    id: entity_id::EntityId,
    components: HashMap<ComponentTypeId, UntypedComponent>,
}

pub struct TypedComponentWriteback<'a, T: ComponentType> {
    component : TypedComponent<T>,
    group : &'a mut ComponentGroup,
}

impl <'a, T: ComponentType> Deref for TypedComponentWriteback<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.component.deref()
    }
}

impl <'a, T: ComponentType> DerefMut for TypedComponentWriteback<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component.deref_mut()
    }
}

//write back changes when the TypedComponentWriteback goes out of scope
impl <'a, T: ComponentType> Drop for TypedComponentWriteback<'a, T> {
    fn drop(&mut self) {
        self.group.components.insert(self.component.get_type_id(), self.component.get_untyped());
    }
}

impl ComponentGroup {
    pub fn get<T: ComponentType>(&mut self) -> Option<TypedComponentWriteback<T>> {
        self.components
            .get(&component::get_type_id::<T>())
            .cloned()
            .map(|x| TypedComponentWriteback {
                component: TypedComponent::new(x.clone()),
                group : self,
            })
    }
    pub fn get_id(&self) -> entity_id::EntityId {
        self.id
    }
    pub fn new(
        id: entity_id::EntityId,
        components: impl IntoIterator<Item = UntypedComponent>,
    ) -> Self {
        ComponentGroup {
            components: components
                .into_iter()
                .map(|component| (component.get_type(), component))
                .collect(),
            id,
        }
    }
    pub fn get_changes(&self) -> Vec<Change> {
        self.components
            .iter()
            .filter(|(_, component)| component.is_unqiue())
            .map(|(_, component)| Change(component.clone(), ChangeType::UpdateComponent))
            .collect()
    }
}

pub struct Change(pub UntypedComponent, pub ChangeType);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ChangeType {
    RemoveComponent,
    UnloadComponent,
    AddComponent,
    UpdateComponent,
}

