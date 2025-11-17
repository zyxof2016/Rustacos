use std::collections::HashSet;
use std::sync::Arc;
use axum::{routing::{get, post, delete, put}, Router, response::{Json, sse::{Sse, Event, KeepAlive}}, extract::{State, Query, Path}};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use adapters_storage_memory::MemStores;
use adapters_notify_sse::SseHub;
use core_model::config::{ConfigItem as DomainConfigItem, ConfigKey};
use core_model::instance::{Instance as DomainInstance, InstanceId, ServiceName};
use core_model::namespace::Namespace as DomainNamespace;
use core_ports::{ConfigStore, InstanceStore, NamespaceStore, Notifier};
use core_usecase::config::PublishConfig;
use uuid::Uuid;
use futures::Stream;
use async_stream::stream;
use tokio::sync::broadcast;

#[derive(Serialize)]
struct ApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
    timestamp: i64,
}

fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        code: 200,
        message: "success".to_string(),
        data: Some(data),
        timestamp: Utc::now().timestamp(),
    })
}

async fn health() -> Json<ApiResponse<&'static str>> {
    ok("UP")
}

#[derive(Clone)]
pub struct AppCtx {
    pub stores: Arc<MemStores>,
    pub notifier: Arc<SseHub>,
}

#[derive(Deserialize)]
struct PublishConfigRequest {
    data_id: String,
    group: String,
    content: String,
    namespace: Option<String>,
    #[serde(default)]
    format: Option<String>,
}

#[derive(Deserialize)]
struct GetConfigQuery {
    data_id: String,
    group: Option<String>,
    namespace: Option<String>,
}

#[derive(Deserialize)]
struct ListConfigQuery {
    namespace: String,
    page: Option<u32>,
    size: Option<u32>,
    search: Option<String>,
}

#[derive(Serialize)]
struct PagedConfigResponse {
    total_count: usize,
    page_number: u32,
    page_size: u32,
    pages: u32,
    data: Vec<ConfigItemDto>,
}

#[derive(Serialize)]
struct ConfigItemDto {
    data_id: String,
    group: String,
    content: String,
    namespace: String,
    update_time: i64,
}

fn to_config_dto(c: DomainConfigItem) -> ConfigItemDto {
    ConfigItemDto {
        data_id: c.key.data_id,
        group: c.key.group,
        content: c.content,
        namespace: c.key.namespace,
        update_time: c.updated_at.timestamp(),
    }
}

async fn publish_config(
    State(ctx): State<AppCtx>,
    Json(req): Json<PublishConfigRequest>,
) -> Json<ApiResponse<bool>> {
    let key = ConfigKey {
        namespace: req.namespace.unwrap_or_else(|| "public".into()),
        group: req.group,
        data_id: req.data_id,
    };
    let uc = PublishConfig {
        store: &*ctx.stores,
        history: &*ctx.stores,
        notifier: Some(&*ctx.notifier),
    };
    match uc.exec(key, req.content, req.format, Some("admin".into())).await {
        Ok(_) => ok(true),
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

async fn get_config(
    State(ctx): State<AppCtx>,
    Query(q): Query<GetConfigQuery>,
) -> Json<ApiResponse<Option<ConfigItemDto>>> {
    let key = ConfigKey {
        namespace: q.namespace.unwrap_or_else(|| "public".into()),
        group: q.group.unwrap_or_else(|| "DEFAULT_GROUP".into()),
        data_id: q.data_id,
    };
    let item = ctx.stores.get(&key).await.map(to_config_dto);
    ok(item)
}

#[derive(Deserialize)]
struct DeleteConfigQuery {
    data_id: String,
    group: Option<String>,
    namespace: Option<String>,
}

async fn delete_config(
    State(ctx): State<AppCtx>,
    Query(q): Query<DeleteConfigQuery>,
) -> Json<ApiResponse<bool>> {
    let key = ConfigKey {
        namespace: q.namespace.unwrap_or_else(|| "public".into()),
        group: q.group.unwrap_or_else(|| "DEFAULT_GROUP".into()),
        data_id: q.data_id,
    };
    match ConfigStore::delete(&*ctx.stores, &key).await {
        Ok(v) => ok(v),
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

async fn list_configs(
    State(ctx): State<AppCtx>,
    Query(q): Query<ListConfigQuery>,
) -> Json<ApiResponse<PagedConfigResponse>> {
    let page = q.page.unwrap_or(1);
    let size = q.size.unwrap_or(10);
    match ConfigStore::list(&*ctx.stores, &q.namespace, page, size, q.search.as_deref()).await {
        Ok((total, items)) => {
            let pages = if size == 0 { 0 } else { (total as u32 + size - 1) / size };
            ok(PagedConfigResponse {
                total_count: total,
                page_number: page,
                page_size: size,
                pages,
                data: items.into_iter().map(to_config_dto).collect(),
            })
        }
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

#[derive(Deserialize)]
struct HistoryQuery {
    data_id: String,
    group: String,
    namespace: String,
}

#[derive(Serialize)]
struct HistoryItemDto {
    version: i64,
    deleted: bool,
    content: String,
    updated_at: i64,
}

async fn list_history(
    State(ctx): State<AppCtx>,
    Query(q): Query<HistoryQuery>,
) -> Json<ApiResponse<Vec<HistoryItemDto>>> {
    let key = ConfigKey { namespace: q.namespace, group: q.group, data_id: q.data_id };
    match core_ports::ConfigHistoryStore::list(&*ctx.stores, &key).await {
        Ok(mut items) => {
            items.sort_by(|a, b| b.version_ts.cmp(&a.version_ts));
            let data = items.into_iter().map(|h| HistoryItemDto {
                version: h.version_ts,
                deleted: h.deleted,
                content: h.content,
                updated_at: h.updated_at.timestamp(),
            }).collect();
            ok(data)
        }
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

#[derive(Deserialize)]
struct RollbackRequest {
    data_id: String,
    group: String,
    namespace: String,
    version: i64,
}

async fn rollback_config(
    State(ctx): State<AppCtx>,
    Json(body): Json<RollbackRequest>,
) -> Json<ApiResponse<bool>> {
    let key = ConfigKey { namespace: body.namespace, group: body.group, data_id: body.data_id };
    match core_ports::ConfigHistoryStore::list(&*ctx.stores, &key).await {
        Ok(items) => {
            if let Some(hist) = items.into_iter().find(|h| h.version_ts == body.version) {
                let uc = PublishConfig {
                    store: &*ctx.stores,
                    history: &*ctx.stores,
                    notifier: Some(&*ctx.notifier),
                };
                match uc.exec(key, hist.content, hist.format, Some("admin".into())).await {
                    Ok(_) => ok(true),
                    Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
                }
            } else {
                Json(ApiResponse { code: 404, message: "version not found".into(), data: None, timestamp: Utc::now().timestamp() })
            }
        }
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

#[derive(Deserialize)]
struct ExportQuery {
    namespace: String,
}

async fn export_configs(
    State(ctx): State<AppCtx>,
    Query(q): Query<ExportQuery>,
) -> Json<ApiResponse<Vec<ConfigItemDto>>> {
    // 直接遍历内存存储
    let items: Vec<ConfigItemDto> = ctx.stores
        .configs
        .iter()
        .filter(|e| e.value().key.namespace == q.namespace)
        .map(|e| to_config_dto(e.value().clone()))
        .collect();
    ok(items)
}

#[derive(Deserialize)]
struct ImportItem {
    data_id: String,
    group: String,
    namespace: String,
    content: String,
    #[serde(default)]
    format: Option<String>,
}

async fn import_configs(
    State(ctx): State<AppCtx>,
    Json(items): Json<Vec<ImportItem>>,
) -> Json<ApiResponse<bool>> {
    for it in items {
        let uc = PublishConfig {
            store: &*ctx.stores,
            history: &*ctx.stores,
            notifier: Some(&*ctx.notifier),
        };
        let key = ConfigKey { namespace: it.namespace, group: it.group, data_id: it.data_id };
        if let Err(e) = uc.exec(key, it.content, it.format, Some("admin".into())).await {
            return Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() });
        }
    }
    ok(true)
}

pub fn routes() -> Router {
    Router::new()
        .route("/health", get(health))
}

pub fn routes_with_mem(ctx: Arc<MemStores>) -> Router {
    let notifier = Arc::new(SseHub::new());
    let app_ctx = AppCtx { stores: ctx.clone(), notifier: notifier.clone() };
    Router::new()
        .route("/health", get(health))
        .route("/nacos/v1/cs/configs", post(publish_config).get(get_config).delete(delete_config))
        .route("/nacos/v1/cs/configs/list", get(list_configs))
        .route("/nacos/v1/cs/configs/history", get(list_history))
        .route("/nacos/v1/cs/configs/history/rollback", post(rollback_config))
        .route("/nacos/v1/cs/configs/export", get(export_configs))
        .route("/nacos/v1/cs/configs/import", post(import_configs))
        // instance
        .route("/nacos/v1/ns/instance", post(register_instance))
        .route("/nacos/v1/ns/instance/beat", post(beat_instance))
        .route("/nacos/v1/ns/instance/:service_name/:instance_id", delete(deregister_instance))
        .route("/nacos/v1/ns/instance/list", get(list_instances))
        // services
        .route("/nacos/v1/ns/service/list", get(list_services))
        // namespaces console
        .route("/nacos/v1/console/namespaces", post(create_namespace).get(list_namespaces))
        .route("/nacos/v1/console/namespaces/:namespace", put(update_namespace).delete(delete_namespace))
        // sse
        .route("/nacos/v1/events/stream", get(stream_events))
        .with_state(app_ctx)
}

// -------------------- Instance APIs --------------------
#[derive(Serialize)]
struct InstanceDto {
    id: String,
    ip: String,
    port: u16,
    service_name: String,
    group_name: String,
    cluster_name: String,
    weight: f64,
    healthy: bool,
    metadata: std::collections::HashMap<String, String>,
    last_beat_time: String,
}

fn to_instance_dto(i: DomainInstance) -> InstanceDto {
    InstanceDto {
        id: i.id.0,
        ip: i.ip,
        port: i.port,
        service_name: i.service.0,
        group_name: i.group,
        cluster_name: i.cluster,
        weight: i.weight,
        healthy: i.healthy,
        metadata: i.metadata,
        last_beat_time: i.last_beat_at.to_rfc3339(),
    }
}

#[derive(Deserialize)]
struct RegisterInstanceRequest {
    ip: String,
    port: u16,
    service_name: String,
    group_name: Option<String>,
    cluster_name: Option<String>,
    weight: Option<f64>,
    metadata: Option<std::collections::HashMap<String, String>>,
}

async fn register_instance(
    State(ctx): State<AppCtx>,
    Json(req): Json<RegisterInstanceRequest>,
) -> Json<ApiResponse<String>> {
    let id = Uuid::new_v4().to_string();
    let service = ServiceName(req.service_name.clone());
    let instance = DomainInstance {
        id: InstanceId(id.clone()),
        ip: req.ip,
        port: req.port,
        service: service.clone(),
        group: req.group_name.unwrap_or_else(|| "DEFAULT_GROUP".into()),
        cluster: req.cluster_name.unwrap_or_else(|| "DEFAULT".into()),
        weight: req.weight.unwrap_or(1.0),
        healthy: true,
        metadata: req.metadata.unwrap_or_default(),
        last_beat_at: Utc::now(),
    };
    let res = InstanceStore::register(&*ctx.stores, instance).await;
    // 通知
    Notifier::notify_instance_change(&*ctx.notifier, &service).await;
    match res {
        Ok(_) => ok(id),
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

#[derive(Deserialize)]
struct BeatRequest {
    service_name: String,
    instance_id: String,
}

async fn beat_instance(
    State(ctx): State<AppCtx>,
    Json(req): Json<BeatRequest>,
) -> Json<ApiResponse<bool>> {
    let service = ServiceName(req.service_name);
    let id = InstanceId(req.instance_id);
    let res = InstanceStore::beat(&*ctx.stores, &service, &id).await;
    // 心跳也可触发变更通知（可选）
    Notifier::notify_instance_change(&*ctx.notifier, &service).await;
    match res {
        Ok(v) => ok(v),
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

async fn deregister_instance(
    State(ctx): State<AppCtx>,
    Path((service_name, instance_id)): Path<(String, String)>,
) -> Json<ApiResponse<()>> {
    let service = ServiceName(service_name);
    let id = InstanceId(instance_id);
    let res = InstanceStore::deregister(&*ctx.stores, &service, &id).await;
    Notifier::notify_instance_change(&*ctx.notifier, &service).await;
    match res {
        Ok(_v) => ok(()),
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

#[derive(Deserialize)]
struct StreamQuery {
    topic: Option<String>, // "config" or "instance"
    access_token: Option<String>,
}

async fn stream_events(
    State(ctx): State<AppCtx>,
    Query(q): Query<StreamQuery>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, StatusCode> {
    // 简易鉴权：若启用 SSE_AUTH_REQUIRED（默认启用），则要求 Authorization 或 accessToken 存在
    let auth_required = std::env::var("SSE_AUTH_REQUIRED")
        .map(|v| !matches!(v.as_str(), "0" | "false" | "False" | "FALSE"))
        .unwrap_or(true);
    if auth_required {
        let has_auth = headers.get("authorization").is_some() || q.access_token.is_some();
        if !has_auth {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    let topic = q.topic.unwrap_or_else(|| "config".into());
    let mut rx = if topic == "instance" {
        ctx.notifier.tx_instance.subscribe()
    } else {
        ctx.notifier.tx_config.subscribe()
    };
    let s = stream! {
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
    Ok(Sse::new(s).keep_alive(KeepAlive::new()))
}

#[derive(Deserialize)]
struct ListInstanceQuery {
    service_name: Option<String>,
}

async fn list_instances(
    State(ctx): State<AppCtx>,
    Query(q): Query<ListInstanceQuery>,
) -> Json<ApiResponse<Vec<InstanceDto>>> {
    let service = q.service_name.map(ServiceName);
    match InstanceStore::list(&*ctx.stores, service.as_ref()).await {
        Ok(items) => ok(items.into_iter().map(to_instance_dto).collect()),
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

async fn list_services(
    State(ctx): State<AppCtx>,
) -> Json<ApiResponse<Vec<String>>> {
    let mut set: HashSet<String> = HashSet::new();
    for it in ctx.stores.instances.iter() {
        set.insert(it.value().service.0.clone());
    }
    ok(set.into_iter().collect())
}

// -------------------- Namespace APIs --------------------
#[derive(Serialize)]
struct NamespaceDto {
    namespace: String,
    namespace_show_name: String,
    namespace_desc: String,
    quota: u32,
    create_time: i64,
    update_time: i64,
}

fn to_namespace_dto(n: DomainNamespace) -> NamespaceDto {
    NamespaceDto {
        namespace: n.id,
        namespace_show_name: n.show_name,
        namespace_desc: n.desc,
        quota: n.quota,
        create_time: n.created_at,
        update_time: n.updated_at,
    }
}

#[derive(Deserialize)]
struct CreateNamespaceRequest {
    namespace: String,
    namespace_show_name: String,
    #[serde(default)]
    namespace_desc: Option<String>,
}

#[derive(Deserialize)]
struct UpdateNamespaceRequest {
    namespace_show_name: String,
    #[serde(default)]
    namespace_desc: Option<String>,
    #[serde(default)]
    quota: Option<u32>,
}

async fn create_namespace(
    State(ctx): State<AppCtx>,
    Json(req): Json<CreateNamespaceRequest>,
) -> Json<ApiResponse<bool>> {
    let now = Utc::now().timestamp();
    let ns = DomainNamespace {
        id: req.namespace,
        show_name: req.namespace_show_name,
        desc: req.namespace_desc.unwrap_or_default(),
        quota: 0,
        created_at: now,
        updated_at: now,
    };
    match NamespaceStore::create(&*ctx.stores, ns).await {
        Ok(_) => ok(true),
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

async fn list_namespaces(
    State(ctx): State<AppCtx>,
) -> Json<ApiResponse<Vec<NamespaceDto>>> {
    match NamespaceStore::list(&*ctx.stores).await {
        Ok(items) => ok(items.into_iter().map(to_namespace_dto).collect()),
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}

async fn update_namespace(
    State(ctx): State<AppCtx>,
    Path(namespace): Path<String>,
    Json(req): Json<UpdateNamespaceRequest>,
) -> Json<ApiResponse<bool>> {
    let now = Utc::now().timestamp();
    // 读旧值
    let mut found = None;
    for it in ctx.stores.namespaces.iter() {
        if it.key() == &namespace {
            found = Some(it.value().clone());
            break;
        }
    }
    if let Some(mut ns) = found {
        ns.show_name = req.namespace_show_name;
        if let Some(desc) = req.namespace_desc { ns.desc = desc; }
        if let Some(q) = req.quota { ns.quota = q; }
        ns.updated_at = now;
        match NamespaceStore::update(&*ctx.stores, ns).await {
            Ok(v) => ok(v),
            Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
        }
    } else {
        Json(ApiResponse { code: 404, message: "namespace not found".into(), data: None, timestamp: Utc::now().timestamp() })
    }
}

async fn delete_namespace(
    State(ctx): State<AppCtx>,
    Path(namespace): Path<String>,
) -> Json<ApiResponse<bool>> {
    match NamespaceStore::delete(&*ctx.stores, &namespace).await {
        Ok(v) => ok(v),
        Err(e) => Json(ApiResponse { code: 500, message: e.to_string(), data: None, timestamp: Utc::now().timestamp() }),
    }
}


