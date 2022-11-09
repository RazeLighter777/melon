use std::ops::{Deref, DerefMut};

use hashbrown::{HashMap, HashSet};

use crate::{
    component::{self, ComponentType, ComponentTypeId, TypedComponent, UntypedComponent},
    entity_builder, entity_id, resource,
    resource_writer::{self},
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
        self.query.components.insert(component::type_id::<T>());
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
    resource_writer: resource_writer::ResourceWriter,
}

impl QueryResult {
    pub(crate) fn dissolve(self) -> (Vec<Change>, resource_writer::ResourceWriter) {
        (
            self.entities
                .into_iter()
                .flat_map(|x| x.get_changes())
                .collect(),
            self.resource_writer,
        )
    }
    pub fn iter(&mut self) -> std::slice::IterMut<ComponentGroup> {
        self.entities.iter_mut()
    }
    pub fn write_resource<R: resource::Resource + 'static, ReturnType>(
        &mut self,
        closure: impl FnOnce(&mut R) -> ReturnType + 'static + Send,
    ) {
        self.resource_writer.write_resource(closure);
    }
    pub fn add_entity(&mut self) -> entity_builder::EntityBuilder {
        entity_builder::EntityBuilder::new(self)
    }
}
impl entity_builder::SpawnLocation for QueryResult {
    fn spawn(&mut self, components: Vec<component::UntypedComponent>) {
        self.entities.push(ComponentGroup {
            id: entity_id::EntityId::new(),
            components: components.into_iter().map(|x| (x.get_type(), x)).collect(),
            new: true,
        });
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
                resource_writer: resource_writer::ResourceWriter::new(),
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
    new: bool,
}

pub struct TypedComponentWriteback<'a, T: ComponentType> {
    component: TypedComponent<T>,
    group: &'a mut ComponentGroup,
}

impl<'a, T: ComponentType> Deref for TypedComponentWriteback<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.component.deref()
    }
}

impl<'a, T: ComponentType> DerefMut for TypedComponentWriteback<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component.deref_mut()
    }
}

//write back changes when the TypedComponentWriteback goes out of scope
impl<'a, T: ComponentType> Drop for TypedComponentWriteback<'a, T> {
    fn drop(&mut self) {
        self.group
            .components
            .insert(self.component.get_type_id(), self.component.get_untyped());
    }
}

impl ComponentGroup {
    pub fn get<T: ComponentType>(&mut self) -> Option<TypedComponentWriteback<T>> {
        self.components
            .get(&component::type_id::<T>())
            .cloned()
            .map(|x| TypedComponentWriteback {
                component: TypedComponent::new(x),
                group: self,
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
            new: false,
        }
    }
    pub fn get_changes(&self) -> Vec<Change> {
        self.components
            .iter()
            .filter(|(_, component)| component.is_unqiue())
            .map(|(_, component)| {
                Change(
                    component.clone(),
                    if self.new {
                        ChangeType::AddComponent
                    } else {
                        ChangeType::UpdateComponent
                    },
                )
            })
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
