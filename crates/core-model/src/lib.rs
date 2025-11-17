pub mod config {
    use serde::{Deserialize, Serialize};
    use chrono::{DateTime, Utc};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConfigKey {
        pub namespace: String,
        pub group: String,
        pub data_id: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConfigItem {
        pub key: ConfigKey,
        pub content: String,
        pub format: Option<String>,
        pub updated_at: DateTime<Utc>,
        pub updated_by: Option<String>,
        pub version_ts: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConfigHistoryItem {
        pub key: ConfigKey,
        pub content: String,
        pub format: Option<String>,
        pub version_ts: i64,
        pub deleted: bool,
        pub updated_at: DateTime<Utc>,
        pub actor: Option<String>,
    }
}

pub mod instance {
    use serde::{Deserialize, Serialize};
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InstanceId(pub String);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ServiceName(pub String);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Instance {
        pub id: InstanceId,
        pub ip: String,
        pub port: u16,
        pub service: ServiceName,
        pub group: String,
        pub cluster: String,
        pub weight: f64,
        pub healthy: bool,
        pub metadata: HashMap<String, String>,
        pub last_beat_at: DateTime<Utc>,
    }
}

pub mod namespace {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Namespace {
        pub id: String,
        pub show_name: String,
        pub desc: String,
        pub quota: u32,
        pub created_at: i64,
        pub updated_at: i64,
    }
}


