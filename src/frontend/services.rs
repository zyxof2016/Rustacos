use gloo_net::http::{Request, RequestBuilder};
use serde::{Deserialize, Serialize};
use web_sys::window;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::EventSource;
#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub service_name: String,
    pub group_name: String,
    pub cluster_name: String,
    pub weight: f64,
    pub healthy: bool,
    pub metadata: std::collections::HashMap<String, String>,
    pub last_beat_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    pub data_id: String,
    pub group: String,
    pub content: String,
    pub namespace: String,
    pub update_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    pub namespace: String,
    pub namespace_show_name: String,
    pub namespace_desc: String,
    pub quota: u32,
    pub create_time: i64,
    pub update_time: i64,
}

#[derive(Debug, Serialize)]
pub struct RegisterInstanceRequest {
    pub ip: String,
    pub port: u16,
    pub service_name: String,
    pub group_name: Option<String>,
    pub cluster_name: Option<String>,
    pub weight: Option<f64>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct PublishConfigRequest {
    pub data_id: String,
    pub group: String,
    pub content: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateNamespaceRequest {
    pub namespace: String,
    pub namespace_show_name: String,
    pub namespace_desc: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateNamespaceRequest {
    pub namespace_show_name: String,
    pub namespace_desc: Option<String>,
    pub quota: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    pub timestamp: i64,
}

pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: "/nacos/v1".to_string(),
        }
    }

    // SSE 订阅（topic: "config"|"instance"）
    #[cfg(target_arch = "wasm32")]
    pub fn subscribe_events<F>(topic: &str, mut on_message: F) -> Result<SseHandle, String>
    where
        F: 'static + FnMut(String),
    {
        let mut url = format!("/nacos/v1/events/stream?topic={}", topic);
        if let Some(token) = window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("accessToken").ok().flatten())
        {
            url.push_str(&format!("&access_token={}", urlencoding::encode(&token)));
        }
        let es = EventSource::new(&url).map_err(|e| format!("SSE 打开失败: {:?}", e))?;
        let onmsg = Closure::<dyn FnMut(web_sys::MessageEvent)>::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if let Some(text) = e.data().as_string() {
                on_message(text);
            }
        }));
        es.set_onmessage(Some(onmsg.as_ref().unchecked_ref()));
        // onopen 提示
        let onopen = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |_e: web_sys::Event| {
            if let Some(w) = window() {
                if let Ok(f) = Reflect::get(&w, &JsValue::from_str("showToast")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        let _ = func.call2(&JsValue::NULL, &JsValue::from_str("实时连接已建立"), &JsValue::from_str("success"));
                    }
                }
            }
        }));
        es.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        // onerror 提示（EventSource 会自动重连）
        let onerror = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |_e: web_sys::Event| {
            if let Some(w) = window() {
                if let Ok(f) = Reflect::get(&w, &JsValue::from_str("showToast")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        let _ = func.call2(&JsValue::NULL, &JsValue::from_str("实时连接断开，重试中…"), &JsValue::from_str("primary"));
                    }
                }
            }
        }));
        es.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        Ok(SseHandle { es, _onmessage: onmsg, _onopen: onopen, _onerror: onerror })
    }

 
    fn auth_header_value() -> Option<String> {
        let storage = window()?.local_storage().ok()??;
        let token = storage.get_item("accessToken").ok()??;
        Some(format!("Bearer {}", token))
    }

    fn with_auth(req: RequestBuilder) -> RequestBuilder {
        if let Some(value) = Self::auth_header_value() {
            req.header("Authorization", &value)
        } else {
            req
        }
    }

    // 发送心跳
    pub async fn beat(&self, service_name: &str, instance_id: &str) -> Result<bool, String> {
        #[derive(Serialize)]
        struct BeatRequest<'a> {
            service_name: &'a str,
            instance_id: &'a str,
        }
        let url = format!("{}/ns/instance/beat", self.base_url);
        match Self::with_auth(Request::post(&url))
            .json(&BeatRequest { service_name, instance_id })
            .map_err(|e| format!("序列化失败: {}", e))?
            .send()
            .await
        {
            Ok(response) => {
                if response.ok() {
                    let result: ApiResponse<bool> = response
                        .json()
                        .await
                        .map_err(|e| format!("解析响应失败: {}", e))?;
                    Ok(result.data.unwrap_or(false))
                } else {
                    Err(format!("请求失败: {}", response.status_text()))
                }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 注册服务实例
    pub async fn register_instance(&self, data: RegisterInstanceRequest) -> Result<String, String> {
        let url = format!("{}/ns/instance", self.base_url);
        
        match Self::with_auth(Request::post(&url))
            .json(&data)
            .map_err(|e| format!("序列化失败: {}", e))?
            .send().await {
            Ok(response) => {
                let result: ApiResponse<String> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 {
                    if let Some(id) = result.data { Ok(id) } else { Err("注册失败".into()) }
                } else {
                    Err(result.message)
                }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 注销服务实例
    pub async fn deregister_instance(&self, service_name: &str, instance_id: &str) -> Result<bool, String> {
        let url = format!("{}/ns/instance/{}/{}", self.base_url, service_name, instance_id);
        
        match Self::with_auth(Request::delete(&url)).send().await {
            Ok(response) => {
                let result: ApiResponse<()> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(true) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 获取实例列表
    pub async fn get_instances(&self, service_name: Option<&str>) -> Result<Vec<Instance>, String> {
        let mut url = format!("{}/ns/instance/list", self.base_url);
        if let Some(service) = service_name {
            url.push_str(&format!("?service_name={}", service));
        }
        
        match Self::with_auth(Request::get(&url)).send().await {
            Ok(response) => {
                let result: ApiResponse<Vec<Instance>> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(result.data.unwrap_or_default()) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 获取服务列表
    pub async fn list_services(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/ns/service/list", self.base_url);
        
        match Self::with_auth(Request::get(&url)).send().await {
            Ok(response) => {
                let result: ApiResponse<Vec<String>> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(result.data.unwrap_or_default()) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 发布配置
    pub async fn publish_config(&self, data: PublishConfigRequest) -> Result<bool, String> {
        let url = format!("{}/cs/configs", self.base_url);
        
        match Self::with_auth(Request::post(&url)).json(&data).map_err(|e| format!("序列化失败: {}", e))?.send().await {
            Ok(response) => {
                let result: ApiResponse<bool> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(result.data.unwrap_or(false)) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 获取配置
    pub async fn get_config(&self, data_id: &str, group: &str, namespace: &str) -> Result<Option<ConfigItem>, String> {
        let url = format!(
            "{}/cs/configs?data_id={}&group={}&namespace={}",
            self.base_url, data_id, group, namespace
        );
        
        match Self::with_auth(Request::get(&url)).send().await {
            Ok(response) => {
                let result: ApiResponse<Option<ConfigItem>> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(result.data.unwrap_or(None)) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 删除配置
    pub async fn remove_config(&self, data_id: &str, group: &str, namespace: &str) -> Result<bool, String> {
        let url = format!(
            "{}/cs/configs?data_id={}&group={}&namespace={}",
            self.base_url, data_id, group, namespace
        );
        
        match Self::with_auth(Request::delete(&url)).send().await {
            Ok(response) => {
                let result: ApiResponse<bool> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(result.data.unwrap_or(false)) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 列出配置
    pub async fn list_configs(&self, namespace: &str) -> Result<Vec<ConfigItem>, String> {
        let url = format!("{}/cs/configs/list?namespace={}", self.base_url, namespace);
        
        match Self::with_auth(Request::get(&url)).send().await {
            Ok(response) => {
                let result: ApiResponse<Vec<ConfigItem>> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(result.data.unwrap_or_default()) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 创建命名空间
    pub async fn create_namespace(&self, data: CreateNamespaceRequest) -> Result<bool, String> {
        let url = format!("{}/console/namespaces", self.base_url);
        
        match Self::with_auth(Request::post(&url)).json(&data).map_err(|e| format!("序列化失败: {}", e))?.send().await {
            Ok(response) => {
                let result: ApiResponse<bool> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(result.data.unwrap_or(false)) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 列出命名空间
    pub async fn list_namespaces(&self) -> Result<Vec<Namespace>, String> {
        let url = format!("{}/console/namespaces", self.base_url);
        
        match Self::with_auth(Request::get(&url)).send().await {
            Ok(response) => {
                let result: ApiResponse<Vec<Namespace>> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(result.data.unwrap_or_default()) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 更新命名空间
    pub async fn update_namespace(&self, namespace: &str, data: UpdateNamespaceRequest) -> Result<bool, String> {
        let url = format!("{}/console/namespaces/{}", self.base_url, namespace);
        
        match Self::with_auth(Request::put(&url)).json(&data).map_err(|e| format!("序列化失败: {}", e))?.send().await {
            Ok(response) => {
                let result: ApiResponse<bool> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(result.data.unwrap_or(false)) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }

    // 删除命名空间
    pub async fn delete_namespace(&self, namespace: &str) -> Result<bool, String> {
        let url = format!("{}/console/namespaces/{}", self.base_url, namespace);
        
        match Self::with_auth(Request::delete(&url)).send().await {
            Ok(response) => {
                let result: ApiResponse<bool> = response.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
                if result.code == 200 { Ok(result.data.unwrap_or(false)) } else { Err(result.message) }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub struct SseHandle {
    es: EventSource,
    // 保持闭包不被回收
    _onmessage: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _onopen: Closure<dyn FnMut(web_sys::Event)>,
    _onerror: Closure<dyn FnMut(web_sys::Event)>,
}
#[cfg(target_arch = "wasm32")]
impl SseHandle {
    pub fn close(self) {
        self.es.close();
    }
}