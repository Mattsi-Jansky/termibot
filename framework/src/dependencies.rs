use reqwest::Client;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DependenciesBuilder {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl DependenciesBuilder {
    pub fn new() -> Self {
        DependenciesBuilder {
            values: HashMap::new(),
        }
        .with(Client::new())
    }

    pub fn with<T: Send + Sync + 'static>(mut self, new: T) -> Self {
        self.values
            .insert(new.type_id(), Arc::new(RwLock::new(new)));
        self
    }

    pub fn build(self) -> Dependencies {
        Dependencies {
            values: self.values,
        }
    }
}

pub struct Dependencies {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Dependencies {
    pub(crate) fn get<T: Any + Send + Sync>(
        &self,
    ) -> Option<Arc<RwLock<T>>> {
        self.values.get(&TypeId::of::<T>())
            .map(|arc| arc.clone().downcast().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestType(u32);

    #[tokio::test]
    async fn should_add_and_retrieve_service() {
        let dependencies_builder = DependenciesBuilder::new();

        let dependencies = dependencies_builder.with(TestType(431)).build();

        let result = dependencies.get::<TestType>().unwrap();
        let result = result.read().await;
        assert_eq!(*result, TestType(431))
    }
}
