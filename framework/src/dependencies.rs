use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use reqwest::Client;
use tokio::sync::RwLock;


pub struct DependencesBuilder{
    values: HashMap<TypeId, Arc<RwLock<dyn Any + Send + Sync + 'static>>>
}

impl DependencesBuilder {
    pub fn new() -> Self {
        DependencesBuilder { values: HashMap::new() }
            .with(Client::new())
    }

    pub fn with<T: Send + Sync + 'static>(mut self, new: T) -> Self {
        self.values.insert(TypeId::of::<T>(), Arc::new(RwLock::new(new)));
        self
    }

    pub fn build(self) -> Dependencies {
        Dependencies { values: self.values }
    }
}

pub struct Dependencies {
    values: HashMap<TypeId, Arc<RwLock<dyn Any + Send + Sync + 'static>>>
}

impl Dependencies {
    pub(crate) fn get<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<RwLock<dyn Any + Send + Sync + 'static>>> {
        self.values.get(&TypeId::of::<T>()).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestType(u32);

    #[test]
    fn should_add_and_retrieve_service() {
        let dependencies_builder = DependencesBuilder::new();

        let dependencies = dependencies_builder.with(TestType(431)).build();

        let _result = dependencies.get::<TestType>();
    }
}
