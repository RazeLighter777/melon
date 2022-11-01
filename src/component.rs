use std::{any::Any, fmt::Display, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{entity_builder, entity_id, hashing};

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct ComponentTypeId(u64);

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct ComponentInstanceId(entity_id::EntityId, ComponentTypeId);

impl ComponentInstanceId {
    pub fn new_explicit(
        entity_id: entity_id::EntityId,
        component_type_id: ComponentTypeId,
    ) -> Self {
        ComponentInstanceId(entity_id, component_type_id)
    }
    pub fn new<T: ComponentType + 'static>(entity_id: entity_id::EntityId) -> Self {
        ComponentInstanceId(entity_id, get_type_id::<T>())
    }
    pub fn get_component_type_id(&self) -> ComponentTypeId {
        self.1
    }
    pub fn get_entity_id(&self) -> entity_id::EntityId {
        self.0
    }
}

pub const fn get_type_id<DataType: 'static>() -> ComponentTypeId {
    ComponentTypeId(hashing::string_hash(std::any::type_name::<DataType>()))
}

pub const fn get_type_id_from_str(s: &str) -> ComponentTypeId {
    ComponentTypeId(hashing::string_hash(s))
}

impl ComponentTypeId {
    pub fn new_with_number(id: u64) -> Self {
        ComponentTypeId(id)
    }
    pub fn new<T: 'static>() -> Self {
        get_type_id::<T>()
    }
    pub fn get_number(&self) -> u64 {
        self.0
    }
}

pub trait ComponentType:
    serde::de::DeserializeOwned + Serialize + Any + Send + Sync + Clone
{
    fn initialize(&mut self, _builder: &mut entity_builder::EntityBuilder) {}
    fn into_untyped(self, id: entity_id::EntityId) -> UntypedComponent {
        UntypedComponent::new(self, id)
    }
}

#[derive(Clone)]
pub struct UntypedComponent {
    internal: Arc<UntypedComponentInternal>,
}

struct UntypedComponentInternal {
    component_type_id: ComponentTypeId,
    instance_id: ComponentInstanceId,
    data: Box<dyn Any + Send + Sync>,
}

impl UntypedComponent {
    pub fn get<T: ComponentType + 'static>(&self) -> Option<&T> {
        self.internal.data.downcast_ref::<T>()
    }
    pub fn get_unchecked<T: ComponentType + 'static>(&self) -> &T {
        self.internal.data.downcast_ref::<T>().unwrap()
    }
    pub fn get_type(&self) -> ComponentTypeId {
        self.internal.component_type_id
    }
    pub fn get_entity_id(&self) -> entity_id::EntityId {
        self.internal.instance_id.get_entity_id()
    }
    pub fn get_instance_id(&self) -> ComponentInstanceId {
        self.internal.instance_id
    }
    pub fn new<T: ComponentType>(component: T, entity_id: entity_id::EntityId) -> Self {
        UntypedComponent {
            internal: Arc::new(UntypedComponentInternal {
                component_type_id: get_type_id::<T>(),
                instance_id: ComponentInstanceId::new::<T>(entity_id),
                data: Box::new(component),
            }),
        }
    }
}

impl Display for ComponentTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type<{:X}>", self.0)
    }
}
