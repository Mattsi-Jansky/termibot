use reqwest::Client;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Maximum time the shared HTTP client waits to establish a connection.
const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// Maximum time the shared HTTP client spends on a whole request, including reading the body.
/// Without it, a server that accepts a connection but never responds hangs the caller forever.
const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

pub struct DependenciesBuilder {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Default for DependenciesBuilder {
    fn default() -> Self {
        let mut builder = DependenciesBuilder {
            values: HashMap::new(),
        };
        builder.add_dyn(Box::new(default_http_client()));
        builder
    }
}

fn default_http_client() -> Client {
    Client::builder()
        .connect_timeout(HTTP_CONNECT_TIMEOUT)
        .timeout(HTTP_REQUEST_TIMEOUT)
        .build()
        .expect("HTTP client should build with default settings")
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

    #[tokio::test(start_paused = true)]
    async fn shared_http_client_should_time_out_when_server_never_responds() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let mut open_connections = vec![];
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                open_connections.push(socket);
            }
        });
        let dependencies = DependenciesBuilder::default().build();
        let client = dependencies.get_dyn::<Client>().unwrap();
        let client = client.read().await;
        let started = tokio::time::Instant::now();

        let result = client.get(format!("http://{address}/")).send().await;

        assert!(result.unwrap_err().is_timeout());
        assert!(started.elapsed() >= HTTP_REQUEST_TIMEOUT);
    }
}
