use std::{
    any::Any,
    borrow::Borrow,
    fmt::Display,
    ops::{Deref, DerefMut},
    sync::Arc,
};

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
        ComponentInstanceId(entity_id, type_id::<T>())
    }
    pub fn component_type_id(&self) -> ComponentTypeId {
        self.1
    }
    pub fn entity_id(&self) -> entity_id::EntityId {
        self.0
    }
}

pub const fn type_id<DataType: 'static>() -> ComponentTypeId {
    ComponentTypeId(hashing::string_hash(std::any::type_name::<DataType>()))
}

pub const fn type_id_from_str(s: &str) -> ComponentTypeId {
    ComponentTypeId(hashing::string_hash(s))
}

impl ComponentTypeId {
    pub fn new_with_number(id: u64) -> Self {
        ComponentTypeId(id)
    }
    pub fn new<T: 'static>() -> Self {
        type_id::<T>()
    }
    pub fn num(&self) -> u64 {
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
    pub(crate) fn is_unqiue(&self) -> bool {
        Arc::<_>::strong_count(&self.internal) == 1
    }
    pub fn get_unchecked<T: ComponentType + 'static>(&self) -> &T {
        self.internal.data.downcast_ref::<T>().unwrap()
    }
    pub fn get_type(&self) -> ComponentTypeId {
        self.internal.component_type_id
    }
    pub fn entity_id(&self) -> entity_id::EntityId {
        self.internal.instance_id.entity_id()
    }
    pub fn id(&self) -> ComponentInstanceId {
        self.internal.instance_id
    }
    pub fn new<T: ComponentType>(component: T, entity_id: entity_id::EntityId) -> Self {
        UntypedComponent {
            internal: Arc::new(UntypedComponentInternal {
                component_type_id: type_id::<T>(),
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

enum TypedComponentInternal<T: ComponentType> {
    Unchanged(UntypedComponent),
    Changed(T, ComponentInstanceId),
}

pub struct TypedComponent<T: ComponentType> {
    internal: TypedComponentInternal<T>,
}

impl<T: ComponentType> TypedComponent<T> {
    pub(crate) fn new(component: UntypedComponent) -> Self {
        TypedComponent {
            internal: TypedComponentInternal::Unchanged(component),
        }
    }
    pub fn get(&self) -> &T {
        let x = self.internal.borrow();
        match x {
            TypedComponentInternal::Unchanged(x) => x.get_unchecked::<T>(),
            TypedComponentInternal::Changed(x, _) => x,
        }
    }
    pub fn make_mut(&mut self) -> &mut T {
        if let TypedComponentInternal::Unchanged(c) = &mut self.internal {
            self.internal = TypedComponentInternal::Changed(c.get_unchecked::<T>().clone(), c.id());
        }
        if let TypedComponentInternal::Changed(comp, _) = &mut self.internal {
            comp
        } else {
            unreachable!()
        }
    }
    pub fn get_untyped(&self) -> UntypedComponent {
        match &self.internal {
            TypedComponentInternal::Unchanged(x) => x.clone(),
            TypedComponentInternal::Changed(x, id) => x.clone().into_untyped(id.entity_id()),
        }
    }
    pub fn get_type_id(&self) -> ComponentTypeId {
        match &self.internal {
            TypedComponentInternal::Unchanged(c) => c.get_type(),
            TypedComponentInternal::Changed(_, id) => id.component_type_id(),
        }
    }
    pub fn get_entity_id(&self) -> entity_id::EntityId {
        match &self.internal {
            TypedComponentInternal::Unchanged(c) => c.entity_id(),
            TypedComponentInternal::Changed(_, id) => id.entity_id(),
        }
    }
    pub fn get_instance_id(&self) -> ComponentInstanceId {
        match &self.internal {
            TypedComponentInternal::Unchanged(c) => c.id(),
            TypedComponentInternal::Changed(_, id) => *id,
        }
    }
}

impl<T: ComponentType> Deref for TypedComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T: ComponentType> DerefMut for TypedComponent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.make_mut()
    }
}
