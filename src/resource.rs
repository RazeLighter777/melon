use std::{any::Any, sync::Arc};

use crate::hashing;
pub trait Resource: Send + Sync {}

pub(crate) struct UntypedResource {
    data: Arc<Box<dyn Any + Send + Sync>>,
}

impl UntypedResource {
    pub(crate) fn new<T: Resource + 'static>(data: T) -> UntypedResource {
        UntypedResource {
            data: Arc::new(Box::new(data)),
        }
    }
    pub(crate) fn get_as<T: Resource + 'static>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }
}

pub(crate) fn get_resource_id<T: Resource>() -> u64 {
    hashing::string_hash(std::any::type_name::<T>())
}
