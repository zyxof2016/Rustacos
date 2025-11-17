use std::sync::Arc;
use axum::Router;
use tower_http::services::ServeDir;
use adapters_storage_memory::MemStores;
use chrono::Utc;

pub fn build_app() -> Router {
    let mem = Arc::new(MemStores::default());
    // 简易 TTL 调度：每 10s 扫描一次，超过 30s 未心跳的实例标记为 unhealthy
    {
        let mem_clone = mem.clone();
        tokio::spawn(async move {
            let ttl_secs: i64 = std::env::var("HEARTBEAT_TTL_SECS")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(30);
            let sweep_secs: u64 = std::env::var("HEARTBEAT_SWEEP_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(10);
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(sweep_secs)).await;
                let now = Utc::now();
                // 先收集需要更新的键，避免持锁过久
                let keys: Vec<String> = mem_clone
                    .instances
                    .iter()
                    .filter_map(|entry| {
                        let v = entry.value();
                        let diff = now.signed_duration_since(v.last_beat_at).num_seconds();
                        if diff > ttl_secs && v.healthy {
                            Some(entry.key().clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                for k in keys {
                    if let Some(mut v) = mem_clone.instances.get_mut(&k) {
                        v.healthy = false;
                    }
                }
            }
        });
    }
    let api = api_compat_nacos::routes_with_mem(mem);
    Router::new()
        .merge(api)
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
}


