use clap::Parser;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};
use dashmap::DashMap;

#[derive(Parser, Debug)]
#[command(name = "rustacos")]
#[command(about = "A Nacos-inspired service discovery and configuration management system")]
struct Args {
    #[arg(short, long, default_value_t = 8848)]
    port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Instance {
    id: String,
    ip: String,
    port: u16,
    service_name: String,
    group_name: String,
    cluster_name: String,
    weight: f64,
    healthy: bool,
    metadata: std::collections::HashMap<String, String>,
    last_beat_time: chrono::DateTime<chrono::Utc>,
}

impl Instance {
    fn new(ip: String, port: u16, service_name: String, group_name: Option<String>, cluster_name: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            ip,
            port,
            service_name,
            group_name: group_name.unwrap_or_else(|| "DEFAULT_GROUP".to_string()),
            cluster_name: cluster_name.unwrap_or_else(|| "DEFAULT".to_string()),
            weight: 1.0,
            healthy: true,
            metadata: std::collections::HashMap::new(),
            last_beat_time: chrono::Utc::now(),
        }
    }

    fn update_beat(&mut self) {
        self.last_beat_time = chrono::Utc::now();
        self.healthy = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigItem {
    data_id: String,
    group: String,
    content: String,
    namespace: String,
    update_time: chrono::DateTime<chrono::Utc>,
}

impl ConfigItem {
    fn new(data_id: String, group: String, content: String, namespace: String) -> Self {
        Self {
            data_id,
            group,
            content,
            namespace,
            update_time: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Namespace {
    namespace: String,
    namespace_show_name: String,
    namespace_desc: String,
    quota: u32,
    create_time: i64,
    update_time: i64,
}

impl Namespace {
    fn new(namespace: String, namespace_show_name: String, namespace_desc: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            namespace,
            namespace_show_name,
            namespace_desc,
            quota: 200,
            create_time: now,
            update_time: now,
        }
    }
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
    timestamp: i64,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            code: 200,
            message: "success".to_string(),
            data: Some(data),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    fn error(code: i32, message: String) -> ApiResponse<()> {
        ApiResponse {
            code,
            message,
            data: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RegisterInstanceRequest {
    ip: String,
    port: u16,
    service_name: String,
    group_name: Option<String>,
    cluster_name: Option<String>,
    weight: Option<f64>,
    metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct BeatRequest {
    service_name: String,
    instance_id: String,
}

#[derive(Debug, Deserialize)]
struct PublishConfigRequest {
    data_id: String,
    group: String,
    content: String,
    namespace: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateNamespaceRequest {
    namespace: String,
    namespace_show_name: String,
    namespace_desc: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateNamespaceRequest {
    namespace_show_name: String,
    namespace_desc: Option<String>,
    quota: Option<u32>,
}

struct AppState {
    instances: Arc<DashMap<String, Instance>>,
    configs: Arc<DashMap<String, ConfigItem>>,
    namespaces: Arc<DashMap<String, Namespace>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();
    
    info!("🦀 启动 Rustacos 服务器...");
    info!("📊 Web 管理界面: http://localhost:{}", args.port);
    info!("🔗 API 端点: http://localhost:{}/nacos/v1", args.port);

    let app_state = Arc::new(AppState {
        instances: Arc::new(DashMap::new()),
        configs: Arc::new(DashMap::new()),
        namespaces: Arc::new({
            let ns = DashMap::new();
            // 添加默认命名空间
            ns.insert("public".to_string(), Namespace::new(
                "public".to_string(),
                "Public Namespace".to_string(),
                "Default namespace".to_string()
            ));
            ns
        }),
    });

    let app = create_app(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    info!("🚀 服务器启动成功，监听端口: {}", args.port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn create_app(app_state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::DELETE, axum::http::Method::PUT])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/health", get(health_check))
        
        .route("/nacos/v1/ns/instance", post(register_instance))
        .route("/nacos/v1/ns/instance/:service_name/:instance_id", delete(deregister_instance))
        .route("/nacos/v1/ns/instance/beat", post(beat))
        .route("/nacos/v1/ns/instance/list", get(get_instances))
        .route("/nacos/v1/ns/service/list", get(list_services))
        .route("/nacos/v1/ns/service/:group_name/:service_name", get(get_service))
        
        .route("/nacos/v1/cs/configs", post(publish_config))
        .route("/nacos/v1/cs/configs", get(get_config))
        .route("/nacos/v1/cs/configs", delete(remove_config))
        .route("/nacos/v1/cs/configs/list", get(list_configs))
        
        .route("/nacos/v1/console/namespaces", post(create_namespace))
        .route("/nacos/v1/console/namespaces", get(list_namespaces))
        .route("/nacos/v1/console/namespaces/:namespace", axum::routing::put(update_namespace))
        .route("/nacos/v1/console/namespaces/:namespace", delete(delete_namespace))
        
        .nest_service("/", tower_http::services::ServeDir::new("static"))
        .fallback_service(tower_http::services::ServeDir::new("static").append_index_html_on_directories(true))
        
        .layer(ServiceBuilder::new().layer(cors))
        .with_state(app_state)
}

async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("UP".to_string()))
}

async fn register_instance(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<RegisterInstanceRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut instance = Instance::new(
        request.ip, 
        request.port, 
        request.service_name.clone(),
        request.group_name,
        request.cluster_name
    );
    
    if let Some(weight) = request.weight {
        instance.weight = weight;
    }
    
    if let Some(metadata) = request.metadata {
        instance.metadata = metadata;
    }
    
    let instance_id = instance.id.clone();
    
    app_state.instances.insert(instance_id.clone(), instance.clone());
    
    info!("✅ 实例注册成功: {}@{}:{}", instance.service_name, instance.ip, instance.port);
    Ok(Json(ApiResponse::success(instance_id)))
}

async fn deregister_instance(
    State(app_state): State<Arc<AppState>>,
    Path((service_name, instance_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    if app_state.instances.remove(&instance_id).is_some() {
        info!("🗑️ 实例注销成功: {}@{}", service_name, instance_id);
        Ok(Json(ApiResponse::success(())))
    } else {
        warn!("⚠️ 实例不存在: {}@{}", service_name, instance_id);
        Ok(Json(ApiResponse::success(())))
    }
}

async fn beat(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<BeatRequest>,
) -> Result<Json<ApiResponse<bool>>, StatusCode> {
    if let Some(mut instance) = app_state.instances.get_mut(&request.instance_id) {
        instance.update_beat();
        info!("💓 心跳接收: {}@{}", request.service_name, request.instance_id);
        Ok(Json(ApiResponse::success(true)))
    } else {
        warn!("⚠️ 心跳失败，实例不存在: {}@{}", request.service_name, request.instance_id);
        Ok(Json(ApiResponse::success(false)))
    }
}

async fn get_instances(
    State(app_state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<Instance>>>, StatusCode> {
    let service_name = params.get("service_name").cloned().unwrap_or_default();
    
    let instances: Vec<Instance> = app_state.instances
        .iter()
        .filter(|entry| service_name.is_empty() || entry.value().service_name == service_name)
        .filter(|entry| entry.value().healthy)
        .map(|entry| entry.value().clone())
        .collect();
    
    Ok(Json(ApiResponse::success(instances)))
}

async fn list_services(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    let services: Vec<String> = app_state.instances
        .iter()
        .map(|entry| entry.value().service_name.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    
    Ok(Json(ApiResponse::success(services)))
}

async fn get_service(
    State(app_state): State<Arc<AppState>>,
    Path((group_name, service_name)): Path<(String, String)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let instances: Vec<Instance> = app_state.instances
        .iter()
        .filter(|entry| entry.value().service_name == service_name)
        .map(|entry| entry.value().clone())
        .collect();
    
    let service_info = serde_json::json!({
        "name": service_name,
        "group_name": group_name,
        "clusters": ["DEFAULT"],
        "instances": instances,
        "instance_count": instances.len()
    });
    
    Ok(Json(ApiResponse::success(service_info)))
}

async fn publish_config(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<PublishConfigRequest>,
) -> Result<Json<ApiResponse<bool>>, StatusCode> {
    let namespace = request.namespace.unwrap_or_else(|| "public".to_string());
    let key = format!("{}+{}+{}", namespace, request.group, request.data_id);
    
    let config = ConfigItem::new(
        request.data_id.clone(),
        request.group.clone(),
        request.content.clone(),
        namespace,
    );
    
    app_state.configs.insert(key.clone(), config.clone());
    
    info!("⚙️ 配置发布成功: {}", key);
    Ok(Json(ApiResponse::success(true)))
}

async fn get_config(
    State(app_state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Option<ConfigItem>>>, StatusCode> {
    let data_id = params.get("data_id").cloned().unwrap_or_default();
    let group = params.get("group").cloned().unwrap_or_else(|| "DEFAULT_GROUP".to_string());
    let namespace = params.get("namespace").cloned().unwrap_or_else(|| "public".to_string());
    
    let key = format!("{}+{}+{}", namespace, group, data_id);
    
    if let Some(config) = app_state.configs.get(&key) {
        Ok(Json(ApiResponse::success(Some(config.clone()))))
    } else {
        Ok(Json(ApiResponse::success(None)))
    }
}

async fn remove_config(
    State(app_state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<bool>>, StatusCode> {
    let data_id = params.get("data_id").cloned().unwrap_or_default();
    let group = params.get("group").cloned().unwrap_or_else(|| "DEFAULT_GROUP".to_string());
    let namespace = params.get("namespace").cloned().unwrap_or_else(|| "public".to_string());
    
    let key = format!("{}+{}+{}", namespace, group, data_id);
    let removed = app_state.configs.remove(&key).is_some();
    
    if removed {
        info!("🗑️ 配置删除成功: {}", key);
    }
    
    Ok(Json(ApiResponse::success(removed)))
}

async fn list_configs(
    State(app_state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<ConfigItem>>>, StatusCode> {
    let namespace = params.get("namespace").cloned().unwrap_or_else(|| "public".to_string());
    
    let configs: Vec<ConfigItem> = app_state.configs
        .iter()
        .filter(|entry| entry.value().namespace == *namespace)
        .map(|entry| entry.value().clone())
        .collect();
    
    Ok(Json(ApiResponse::success(configs)))
}

async fn create_namespace(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<CreateNamespaceRequest>,
) -> Result<Json<ApiResponse<bool>>, StatusCode> {
    let namespace = Namespace::new(
        request.namespace.clone(),
        request.namespace_show_name.clone(),
        request.namespace_desc.unwrap_or_default(),
    );
    
    app_state.namespaces.insert(request.namespace.clone(), namespace.clone());
    
    info!("🗂️ 命名空间创建成功: {}", namespace.namespace);
    Ok(Json(ApiResponse::success(true)))
}

async fn list_namespaces(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Namespace>>>, StatusCode> {
    let namespaces: Vec<Namespace> = app_state.namespaces
        .iter()
        .map(|entry| entry.value().clone())
        .collect();
    
    Ok(Json(ApiResponse::success(namespaces)))
}

async fn update_namespace(
    State(app_state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(request): Json<UpdateNamespaceRequest>,
) -> Result<Json<ApiResponse<bool>>, StatusCode> {
    if let Some(mut ns) = app_state.namespaces.get_mut(&namespace) {
        ns.namespace_show_name = request.namespace_show_name.clone();
        ns.namespace_desc = request.namespace_desc.unwrap_or_default();
        if let Some(quota) = request.quota {
            ns.quota = quota;
        }
        ns.update_time = chrono::Utc::now().timestamp();
        
        info!("🗂️ 命名空间更新成功: {}", namespace);
        Ok(Json(ApiResponse::success(true)))
    } else {
        warn!("⚠️ 命名空间不存在: {}", namespace);
        Ok(Json(ApiResponse::success(false)))
    }
}

async fn delete_namespace(
    State(app_state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
) -> Result<Json<ApiResponse<bool>>, StatusCode> {
    // 防止删除默认命名空间
    if namespace == "public" {
        warn!("⚠️ 不能删除默认命名空间: {}", namespace);
        return Ok(Json(ApiResponse::success(false)));
    }
    
    if app_state.namespaces.remove(&namespace).is_some() {
        // 同时删除该命名空间下的所有配置
        let configs_to_remove: Vec<String> = app_state.configs
            .iter()
            .filter(|entry| entry.value().namespace == namespace)
            .map(|entry| entry.key().clone())
            .collect();
        
        for config_key in configs_to_remove {
            app_state.configs.remove(&config_key);
        }
        
        info!("🗂️ 命名空间删除成功: {}", namespace);
        Ok(Json(ApiResponse::success(true)))
    } else {
        warn!("⚠️ 命名空间不存在: {}", namespace);
        Ok(Json(ApiResponse::success(false)))
    }
}