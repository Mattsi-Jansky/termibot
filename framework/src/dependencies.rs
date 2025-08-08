use reqwest::Client;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DependenciesBuilder {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Default for DependenciesBuilder {
    fn default() -> Self {
        let mut builder = DependenciesBuilder {
            values: HashMap::new(),
        };
        builder.add_dyn(Box::new(Client::new()));
        builder
    }
}

impl DependenciesBuilder {
    pub fn add_dyn<T: Send + Sync + 'static + ?Sized>(&mut self, new: Box<T>) {
        self.values
            .insert(new.type_id(), Arc::new(RwLock::new(new)));
    }

    pub fn add<T: Send + Sync + 'static>(&mut self, new: T) {
        self.values
            .insert(new.type_id(), Arc::new(RwLock::new(new)));
    }

    pub fn build(self) -> Dependencies {
        Dependencies {
            values: self.values,
        }
    }
}

#[derive(Default)]
pub struct Dependencies {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Dependencies {
    pub fn get<T: Any + Send + Sync>(&self) -> Option<Arc<RwLock<T>>> {
        self.values
            .get(&TypeId::of::<T>())
            .map(|arc| arc.clone().downcast().unwrap())
    }

    pub fn get_dyn<T: Any + Send + Sync + ?Sized>(&self) -> Option<Arc<RwLock<Box<T>>>> {
        self.values
            .get(&TypeId::of::<Box<T>>())
            .map(|arc| arc.clone().downcast().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestType(u32);

    impl TestTrait for TestType {
        fn get_value(&self) -> u32 {
            self.0
        }
    }

    trait TestTrait {
        fn get_value(&self) -> u32;
    }

    #[tokio::test]
    async fn should_add_and_retrieve_service() {
        let mut dependencies_builder = DependenciesBuilder::default();
        dependencies_builder.add(TestType(431));
        let dependencies = dependencies_builder.build();

        let result = dependencies.get::<TestType>().unwrap();
        let result = result.read().await;
        assert_eq!(*result, TestType(431))
    }

    #[tokio::test]
    async fn should_add_and_retrieve_dynamic_service() {
        let mut dependencies_builder = DependenciesBuilder::default();
        dependencies_builder.add_dyn::<dyn TestTrait + Send + Sync>(Box::new(TestType(431)));
        let dependencies = dependencies_builder.build();

        let result = dependencies
            .get_dyn::<dyn TestTrait + Send + Sync>()
            .unwrap();
        let result = result.read().await;
        assert_eq!(result.get_value(), 431);
    }
}
