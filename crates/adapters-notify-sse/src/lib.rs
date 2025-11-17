use std::sync::Arc;
use axum::{extract::Query, response::sse::{Event, KeepAlive, Sse}, routing::get, Router};
use core_model::config::ConfigKey;
use core_model::instance::ServiceName;
use core_ports::Notifier;
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::broadcast;
use futures::Stream;

#[derive(Clone)]
pub struct SseHub {
    pub tx_config: broadcast::Sender<Value>,
    pub tx_instance: broadcast::Sender<Value>,
}

impl SseHub {
    pub fn new() -> Self {
        let (tx_config, _) = broadcast::channel::<Value>(1024);
        let (tx_instance, _) = broadcast::channel::<Value>(1024);
        Self { tx_config, tx_instance }
    }
}

#[async_trait]
impl Notifier for SseHub {
    async fn notify_config_change(&self, key: &ConfigKey) {
        let payload = serde_json::json!({
            "topic": "config",
            "namespace": key.namespace,
            "group": key.group,
            "data_id": key.data_id
        });
        let _ = self.tx_config.send(payload);
    }
    async fn notify_instance_change(&self, service: &ServiceName) {
        let payload = serde_json::json!({
            "topic": "instance",
            "service_name": service.0
        });
        let _ = self.tx_instance.send(payload);
    }
}

#[derive(serde::Deserialize)]
struct StreamQuery {
    topic: Option<String>, // "config" or "instance"
}

pub fn sse_routes(hub: Arc<SseHub>) -> Router {
    async fn stream_handler(
        Query(q): Query<StreamQuery>,
        axum::extract::State(hub): axum::extract::State<Arc<SseHub>>,
    ) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
        let topic = q.topic.unwrap_or_else(|| "config".into());
        let mut rx = if topic == "instance" {
            hub.tx_instance.subscribe()
        } else {
            hub.tx_config.subscribe()
        };
        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        yield Ok(Event::default().json_data(msg).unwrap_or(Event::default().data("invalid")));
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Sse::new(stream).keep_alive(KeepAlive::new())
    }

    Router::new()
        .route("/nacos/v1/events/stream", get(stream_handler))
        .with_state(hub)
}


