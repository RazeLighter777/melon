use hashbrown::{HashMap, HashSet};

use crate::{
    component::{self, ComponentType, ComponentTypeId, UntypedComponent},
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

pub struct ComponentGroup {
    id: entity_id::EntityId,
    components: HashMap<ComponentTypeId, QueriedComponent>,
    added_components: Vec<UntypedComponent>,
    removed_components: Vec<UntypedComponent>,
}

impl ComponentGroup {
    pub fn get<T: ComponentType>(&self) -> Option<&T> {
        let component_type_id = component::get_type_id::<T>();
        let component = self.components.get(&component_type_id);
        match component {
            Some(component) => component.get(),
            None => None,
        }
    }
    pub fn remove_all(&mut self) {
        self.removed_components
            .extend(self.components.values().map(|x| x.component.clone()));
        self.components.clear();
    }
    pub fn get_id(&self) -> entity_id::EntityId {
        self.id
    }
    pub fn remove<T: ComponentType>(&mut self) {
        let component_type_id = component::get_type_id::<T>();
        let component = self.components.get(&component_type_id);
        match component {
            Some(_component) => {
                self.removed_components.push(_component.component.clone());
            }
            None => (),
        }
    }
    pub fn new(
        id: entity_id::EntityId,
        components: impl IntoIterator<Item = UntypedComponent>,
    ) -> Self {
        ComponentGroup {
            components: components
                .into_iter()
                .map(|component| (component.get_type(), QueriedComponent::new(component)))
                .collect(),
            added_components: Vec::new(),
            id,
            removed_components: Vec::new(),
        }
    }
    pub fn get_unchecked<T: ComponentType>(&self) -> &T {
        let component_type_id = component::get_type_id::<T>();
        let component = self.components.get(&component_type_id).unwrap();
        component.get_unchecked()
    }
    pub fn set<T: ComponentType>(&mut self, c: T) {
        let component_type_id = component::get_type_id::<T>();
        let component = self.components.get_mut(&component_type_id);
        if let Some(comp) = component {
            comp.set(c);
        }
    }
    pub fn get_changes(&self) -> Vec<Change> {
        let mut res = Vec::new();
        for added_component in self.added_components.iter() {
            res.push(Change(added_component.clone(), ChangeType::AddComponent));
        }
        for component in self.components.values() {
            if let Some(change) = &component.write_cache {
                res.push(Change(change.clone(), ChangeType::UpdateComponent));
            }
        }
        for removed_component in self.removed_components.iter() {
            res.push(Change(
                removed_component.clone(),
                ChangeType::RemoveComponent,
            ));
        }
        res
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
pub struct QueriedComponent {
    component: component::UntypedComponent,
    write_cache: Option<component::UntypedComponent>,
}

impl QueriedComponent {
    pub fn new(component: component::UntypedComponent) -> Self {
        QueriedComponent {
            component,
            write_cache: None,
        }
    }
    pub fn get<T: ComponentType + 'static>(&self) -> Option<&T> {
        self.component.get::<T>()
    }
    pub fn get_unchecked<T: ComponentType + 'static>(&self) -> &T {
        self.component.get_unchecked::<T>()
    }
    pub fn set<T: ComponentType + 'static>(&mut self, component: T) {
        self.write_cache = Some(component::UntypedComponent::new(
            component,
            self.component.get_instance_id().get_entity_id(),
        ));
    }
}

#[test]
pub fn test_query() {}
