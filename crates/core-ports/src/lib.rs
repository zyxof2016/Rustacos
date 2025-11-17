use async_trait::async_trait;
use core_model::config::{ConfigHistoryItem, ConfigItem, ConfigKey};
use core_model::instance::{Instance, InstanceId, ServiceName};
use core_model::namespace::Namespace;

#[async_trait]
pub trait ConfigStore: Send + Sync {
    async fn get(&self, key: &ConfigKey) -> Option<ConfigItem>;
    async fn put(&self, item: ConfigItem) -> anyhow::Result<()>;
    async fn delete(&self, key: &ConfigKey) -> anyhow::Result<bool>;
    async fn list(
        self: &Self,
        namespace: &str,
        page: u32,
        size: u32,
        filter: Option<&str>,
    ) -> anyhow::Result<(usize, Vec<ConfigItem>)>;
}

#[async_trait]
pub trait ConfigHistoryStore: Send + Sync {
    async fn append(&self, item: ConfigHistoryItem) -> anyhow::Result<()>;
    async fn list(&self, key: &ConfigKey) -> anyhow::Result<Vec<ConfigHistoryItem>>;
}

#[async_trait]
pub trait InstanceStore: Send + Sync {
    async fn register(&self, ins: Instance) -> anyhow::Result<()>;
    async fn deregister(&self, service: &ServiceName, id: &InstanceId) -> anyhow::Result<bool>;
    async fn beat(&self, service: &ServiceName, id: &InstanceId) -> anyhow::Result<bool>;
    async fn list(&self, service: Option<&ServiceName>) -> anyhow::Result<Vec<Instance>>;
}

#[async_trait]
pub trait NamespaceStore: Send + Sync {
    async fn create(&self, ns: Namespace) -> anyhow::Result<()>;
    async fn update(&self, ns: Namespace) -> anyhow::Result<bool>;
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
    async fn list(&self) -> anyhow::Result<Vec<Namespace>>;
}

#[async_trait]
pub trait Notifier: Send + Sync {
    async fn notify_config_change(&self, key: &ConfigKey);
    async fn notify_instance_change(&self, service: &ServiceName);
}

#[async_trait]
pub trait SchedulerPort: Send + Sync {
    async fn schedule_heartbeat_cleanup(&self);
}


