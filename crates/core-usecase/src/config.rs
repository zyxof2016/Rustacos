use chrono::Utc;
use core_model::config::{ConfigHistoryItem, ConfigItem, ConfigKey};
use core_ports::{ConfigHistoryStore, ConfigStore, Notifier};

pub struct PublishConfig<'a> {
    pub store: &'a dyn ConfigStore,
    pub history: &'a dyn ConfigHistoryStore,
    pub notifier: Option<&'a dyn Notifier>,
}

impl<'a> PublishConfig<'a> {
    pub async fn exec(
        &self,
        key: ConfigKey,
        content: String,
        format: Option<String>,
        actor: Option<String>,
    ) -> anyhow::Result<()> {
        // 读取旧值，写历史
        if let Some(old) = self.store.get(&key).await {
            let hist = ConfigHistoryItem {
                key: old.key.clone(),
                content: old.content.clone(),
                format: old.format.clone(),
                version_ts: Utc::now().timestamp(),
                deleted: false,
                updated_at: Utc::now(),
                actor: actor.clone(),
            };
            self.history.append(hist).await?;
        }
        // 写新值
        let item = ConfigItem {
            key: key.clone(),
            content: content.clone(),
            format,
            updated_at: Utc::now(),
            updated_by: actor.clone(),
            version_ts: Utc::now().timestamp(),
        };
        self.store.put(item).await?;
        // 写历史
        let hist_new = ConfigHistoryItem {
            key,
            content,
            format: None,
            version_ts: Utc::now().timestamp(),
            deleted: false,
            updated_at: Utc::now(),
            actor,
        };
        let key_for_notify = hist_new.key.clone();
        self.history.append(hist_new).await?;
        if let Some(n) = self.notifier {
            n.notify_config_change(&key_for_notify).await;
        }
        Ok(())
    }
}


