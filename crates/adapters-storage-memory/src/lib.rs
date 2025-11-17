use async_trait::async_trait;
use chrono::Utc;
use core_model::config::{ConfigHistoryItem, ConfigItem, ConfigKey};
use core_model::instance::{Instance, InstanceId, ServiceName};
use core_model::namespace::Namespace;
use core_ports::{ConfigHistoryStore, ConfigStore, InstanceStore, NamespaceStore};
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct MemStores {
    pub configs: Arc<DashMap<String, ConfigItem>>,
    pub histories: Arc<DashMap<String, Vec<ConfigHistoryItem>>>,
    pub instances: Arc<DashMap<String, Instance>>,
    pub namespaces: Arc<DashMap<String, Namespace>>,
}

fn key_of(k: &ConfigKey) -> String {
    format!("{}+{}+{}", k.namespace, k.group, k.data_id)
}

#[async_trait]
impl ConfigStore for MemStores {
    async fn get(&self, key: &ConfigKey) -> Option<ConfigItem> {
        self.configs.get(&key_of(key)).map(|v| v.clone())
    }
    async fn put(&self, item: ConfigItem) -> anyhow::Result<()> {
        self.configs.insert(key_of(&item.key), item);
        Ok(())
    }
    async fn delete(&self, key: &ConfigKey) -> anyhow::Result<bool> {
        Ok(self.configs.remove(&key_of(key)).is_some())
    }
    async fn list(
        &self,
        namespace: &str,
        page: u32,
        size: u32,
        filter: Option<&str>,
    ) -> anyhow::Result<(usize, Vec<ConfigItem>)> {
        let mut v: Vec<ConfigItem> = self
            .configs
            .iter()
            .filter(|e| e.value().key.namespace == namespace)
            .filter(|e| {
                if let Some(f) = filter {
                    e.value().key.data_id.contains(f)
                } else {
                    true
                }
            })
            .map(|e| e.value().clone())
            .collect();
        v.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        let total = v.len();
        let start = ((page.saturating_sub(1)) as usize * size as usize).min(total);
        let end = (start + size as usize).min(total);
        Ok((total, v[start..end].to_vec()))
    }
}

#[async_trait]
impl ConfigHistoryStore for MemStores {
    async fn append(&self, item: ConfigHistoryItem) -> anyhow::Result<()> {
        let key = key_of(&item.key);
        let mut entry = self.histories.entry(key).or_default();
        // 就地 push，避免不必要拷贝
        entry.value_mut().push(item);
        Ok(())
    }
    async fn list(&self, key: &ConfigKey) -> anyhow::Result<Vec<ConfigHistoryItem>> {
        Ok(self
            .histories
            .get(&key_of(key))
            .map(|v| v.value().clone())
            .unwrap_or_default())
    }
}

#[async_trait]
impl InstanceStore for MemStores {
    async fn register(&self, ins: Instance) -> anyhow::Result<()> {
        self.instances.insert(ins.id.0.clone(), ins);
        Ok(())
    }
    async fn deregister(&self, _service: &ServiceName, id: &InstanceId) -> anyhow::Result<bool> {
        Ok(self.instances.remove(&id.0).is_some())
    }
    async fn beat(&self, _service: &ServiceName, id: &InstanceId) -> anyhow::Result<bool> {
        if let Some(mut i) = self.instances.get_mut(&id.0) {
            i.last_beat_at = Utc::now();
            i.healthy = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    async fn list(&self, service: Option<&ServiceName>) -> anyhow::Result<Vec<Instance>> {
        Ok(self
            .instances
            .iter()
            .filter(|e| {
                if let Some(s) = service {
                    e.value().service.0 == s.0
                } else {
                    true
                }
            })
            .map(|e| e.value().clone())
            .collect())
    }
}

#[async_trait]
impl NamespaceStore for MemStores {
    async fn create(&self, ns: Namespace) -> anyhow::Result<()> {
        self.namespaces.insert(ns.id.clone(), ns);
        Ok(())
    }
    async fn update(&self, ns: Namespace) -> anyhow::Result<bool> {
        Ok(self.namespaces.insert(ns.id.clone(), ns).is_some())
    }
    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        Ok(self.namespaces.remove(id).is_some())
    }
    async fn list(&self) -> anyhow::Result<Vec<Namespace>> {
        Ok(self.namespaces.iter().map(|e| e.value().clone()).collect())
    }
}


