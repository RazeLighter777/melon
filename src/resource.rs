use std::any::Any;

use crate::hashing;
pub trait Resource: Send + Sync {}

pub(crate) struct UntypedResource {
    data: Box<dyn Any + Send + Sync>,
}

impl UntypedResource {
    pub(crate) fn new<T: Resource + 'static>(data: T) -> UntypedResource {
        UntypedResource {
            data: Box::new(data),
        }
    }
    pub(crate) fn get_as<T: Resource + 'static>(&self) -> &T {
        self.data.downcast_ref::<T>().unwrap()
    }
    pub(crate) fn get_as_mut<T: Resource + 'static>(&mut self) -> &mut T {
        self.data.downcast_mut::<T>().unwrap()
    }
}

pub(crate) fn get_resource_id<T: Resource>() -> u64 {
    hashing::string_hash(std::any::type_name::<T>())
}
