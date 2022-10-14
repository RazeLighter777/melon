use hashbrown::{HashMap, HashSet};

use crate::{
    component::{self, ComponentInstanceId, ComponentType, ComponentTypeId, UntypedComponent},
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
        let i = self.entities.iter_mut();
        i
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
    removed_components: Vec<ComponentInstanceId>,
    removed: bool,
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
    pub fn remove_entity(&mut self) {
        self.removed = true;
    }
    pub fn remove<T: ComponentType>(&mut self) {
        let component_type_id = component::get_type_id::<T>();
        let component = self.components.get(&component_type_id);
        match component {
            Some(_component) => {
                self.removed_components
                    .push(component::ComponentInstanceId::new::<T>(self.id));
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
            removed: false,
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
        match component {
            Some(comp) => {
                comp.set(c);
            }
            None => {}
        }
    }
    pub fn get_changes(&self) -> Vec<Change> {
        let mut res = Vec::new();
        for added_component in self.added_components.iter() {
            res.push(Change::AddComponent(added_component.clone()));
        }
        for component in self.components.values() {
            if let Some(change) = &component.write_cache {
                res.push(Change::UpdateComponent(change.clone()));
            }
        }
        for removed_component in self.removed_components.iter() {
            res.push(Change::RemoveComponent(*removed_component));
        }
        if self.removed {
            res.push(Change::RemoveEntity(self.id));
        }
        res
    }
}

pub enum Change {
    RemoveComponent(ComponentInstanceId),
    AddComponent(UntypedComponent),
    UpdateComponent(UntypedComponent),
    RemoveEntity(entity_id::EntityId),
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
