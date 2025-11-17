#[cfg(target_arch = "wasm32")]
use leptos::*;
 

#[cfg(target_arch = "wasm32")]
use crate::frontend::{
    components::{StatCard, Loading},
};
#[cfg(target_arch = "wasm32")]
use crate::frontend::services::ApiClient;
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
 

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Dashboard() -> impl IntoView {
    let (loading, set_loading) = create_signal(true);
    let (total_services, set_total_services) = create_signal(0usize);
    let (total_instances, set_total_instances) = create_signal(0usize);
    let (total_namespaces, set_total_namespaces) = create_signal(0usize);
    let (total_configs, set_total_configs) = create_signal(0usize);
    let (recent_services, set_recent_services) = create_signal::<Vec<String>>(vec![]);
    let (error, set_error) = create_signal::<Option<String>>(None);

    // 加载统计
    let load_all = {
        let set_total_services = set_total_services.clone();
        let set_total_instances = set_total_instances.clone();
        let set_total_namespaces = set_total_namespaces.clone();
        let set_total_configs = set_total_configs.clone();
        let set_recent_services = set_recent_services.clone();
        let set_error = set_error.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                let api = ApiClient::new();
                // 并行加载服务、实例、命名空间
                let (services_res, instances_res, namespaces_res) = futures::join!(
                    api.list_services(),
                    api.get_instances(None),
                    api.list_namespaces()
                );
                match (services_res, instances_res, namespaces_res) {
                    (Ok(services), Ok(instances), Ok(namespaces)) => {
                        set_total_services.set(services.len());
                        set_total_instances.set(instances.len());
                        set_total_namespaces.set(namespaces.len());
                        set_recent_services.set(services.iter().take(5).cloned().collect());
                        // 统计所有命名空间的配置总数（读取分页接口总量）
                        let mut config_sum = 0usize;
                        for ns in namespaces {
                            let url = format!("/nacos/v1/cs/configs/list?namespace={}&page=1&size=1", ns.namespace);
                            let req = {
                                if let Some(token) = web_sys::window()
                                    .and_then(|w| w.local_storage().ok().flatten())
                                    .and_then(|s| s.get_item("accessToken").ok().flatten()) {
                                    Request::get(&url).header("Authorization", &format!("Bearer {}", token))
                                } else {
                                    Request::get(&url)
                                }
                            };
                            if let Ok(resp) = req.send().await {
                                if resp.ok() {
                                    #[derive(serde::Deserialize)]
                                    struct Paged { total_count: usize }
                                    #[derive(serde::Deserialize)]
                                    struct Wrap { code: i32, data: Option<Paged> }
                                    if let Ok(wrap) = resp.json::<Wrap>().await {
                                        if wrap.code == 200 {
                                            if let Some(p) = wrap.data { config_sum += p.total_count; }
                                        }
                                    }
                                }
                            }
                        }
                        set_total_configs.set(config_sum);
                        set_error.set(None);
                    }
                    _ => {
                        set_error.set(Some("加载仪表盘数据失败".to_string()));
                    }
                }
                set_loading.set(false);
            });
        }
    };
    load_all();
    
    view! {
        <div class="container mt-4">
            <h2 class="page-title">
                <i class="bi bi-speedometer2"></i> {" 系统仪表盘"}
            </h2>
            
            <div class="row mb-4">
                <StatCard 
                    title="服务总数".to_string()
                    value={total_services.get().to_string()}
                    icon="bi-server".to_string()
                    color="primary".to_string()
                />
                <StatCard 
                    title="实例总数".to_string()
                    value={total_instances.get().to_string()}
                    icon="bi-hdd-stack".to_string()
                    color="success".to_string()
                />
                <StatCard 
                    title="配置总数".to_string()
                    value={total_configs.get().to_string()}
                    icon="bi-gear".to_string()
                    color="info".to_string()
                />
                <StatCard 
                    title="命名空间总数".to_string()
                    value={total_namespaces.get().to_string()}
                    icon="bi-collection".to_string()
                    color="warning".to_string()
                />
            </div>
            
            <div class="row">
                <div class="col-md-6">
                    <div class="card">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <h5 class="card-title mb-0">{"最近服务"}</h5>
                            <button class="btn btn-sm btn-outline-secondary" on:click=move |_| load_all()>
                                <i class="bi bi-arrow-clockwise"></i> {" 刷新"}
                            </button>
                        </div>
                        <div class="card-body">
                            {move || {
                                if let Some(msg) = error.get() {
                                    view! { <div class="alert alert-danger">{msg}</div> }.into_view()
                                } else if loading.get() {
                                    view! { <Loading /> }.into_view()
                                } else {
                                    view! {
                                        <div class="list-group">
                                            {recent_services.get().into_iter().map(|s| {
                                                view!{
                                                    <div class="list-group-item d-flex justify-content-between align-items-center">
                                                        <span>{s.clone()}</span>
                                                        <a href="/services" class="btn btn-sm btn-outline-primary">{"查看"}</a>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_view()
                                }
                            }}
                        </div>
                    </div>
                </div>
                <div class="col-md-6">
                    <div class="card">
                        <div class="card-header">
                            <h5 class="card-title mb-0">{"系统状态"}</h5>
                        </div>
                        <div class="card-body">
                            <div class="list-group">
                                <div class="list-group-item d-flex justify-content-between align-items-center">
                                    <span>{"服务器状态"}</span>
                                    <span class="badge bg-success rounded-pill">{"运行中"}</span>
                                </div>
                                <div class="list-group-item d-flex justify-content-between align-items-center">
                                    <span>{"数据存储"}</span>
                                    <span class="badge bg-primary rounded-pill">{"内存"}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn Dashboard() {}