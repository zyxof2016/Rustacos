#[cfg(target_arch = "wasm32")]
use leptos::*;
use crate::frontend::components::Loading;
#[cfg(target_arch = "wasm32")]
use crate::frontend::services::{ApiClient, Instance, RegisterInstanceRequest, SseHandle};
#[cfg(target_arch = "wasm32")]
use serde_json::Value;
 

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Services() -> impl IntoView {
    let api = ApiClient::new();
    let (loading, set_loading) = create_signal(true);
    let (services, set_services) = create_signal::<Vec<String>>(vec![]);
    let (current_service, set_current_service) = create_signal::<Option<String>>(None);
    let (instances, set_instances) = create_signal::<Vec<Instance>>(vec![]);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (info, set_info) = create_signal::<Option<String>>(None);
    let (detail_open, set_detail_open) = create_signal(false);
    let (detail_instance, set_detail_instance) = create_signal::<Option<Instance>>(None);
    // 保存 SSE 句柄，组件销毁时自动关闭
    let sse_handle = create_rw_signal::<Option<SseHandle>>(None);

    // 初次加载服务列表
    spawn_local(async move {
        match api.list_services().await {
            Ok(list) => {
                let first = list.first().cloned();
            set_services.set(list.clone());
                set_current_service.set(first);
            }
            Err(e) => set_error.set(Some(format!("加载服务失败: {}", e))),
        }
        set_loading.set(false);
    });

    // 当选择的服务变化时加载实例
    create_effect(move |_| {
        if let Some(name) = current_service.get() {
            let name_clone = name.clone();
            let set_instances_cloned = set_instances.clone();
            spawn_local(async move {
                let api = ApiClient::new();
                match api.get_instances(Some(&name_clone)).await {
                    Ok(list) => set_instances_cloned.set(list),
                    Err(e) => {
                        set_instances_cloned.set(vec![]);
                        web_sys::console::error_1(&format!("加载实例失败: {}", e).into());
                    }
                }
            });
        } else {
            set_instances.set(vec![]);
        }
    });

    let on_select = move |name: String| {
        set_current_service.set(Some(name));
    };

    // 注册实例表单
    let (create_open, set_create_open) = create_signal(false);
    let (form_service, set_form_service) = create_signal(String::new());
    let (form_ip, set_form_ip) = create_signal(String::from("127.0.0.1"));
    let (form_port, set_form_port) = create_signal(String::from("8080"));
    let (form_group, set_form_group) = create_signal(String::from("DEFAULT_GROUP"));
    let (form_cluster, set_form_cluster) = create_signal(String::from("DEFAULT"));
    let (form_weight, set_form_weight) = create_signal(String::from("1.0"));
    let (form_metadata, set_form_metadata) = create_signal(String::new());
    let (creating, set_creating) = create_signal(false);

    let reload_services = {
        let set_services = set_services.clone();
        let set_current_service = set_current_service.clone();
        move || {
            spawn_local(async move {
                match ApiClient::new().list_services().await {
                    Ok(list) => {
                        set_current_service.set(list.first().cloned());
                        set_services.set(list);
                    }
                    Err(e) => web_sys::console::error_1(&format!("刷新服务失败: {}", e).into()),
                }
            });
        }
    };

    let reload_instances = {
        let current_service = current_service.clone();
        let set_instances = set_instances.clone();
        move || {
            if let Some(name) = current_service.get() {
                spawn_local(async move {
                    match ApiClient::new().get_instances(Some(&name)).await {
                        Ok(list) => set_instances.set(list),
                        Err(e) => web_sys::console::error_1(&format!("刷新实例失败: {}", e).into()),
                    }
                });
            }
        }
    };

    // 订阅实例变更事件，自动刷新服务列表和当前服务实例
    {
        let reload_services_cb = reload_services.clone();
        let reload_instances_cb = reload_instances.clone();
        let current_service_sig = current_service.clone();
        match ApiClient::subscribe_events("instance", move |msg| {
            if let Ok(v) = serde_json::from_str::<Value>(&msg) {
                let svc = v.get("service_name").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                reload_services_cb();
                if let Some(cur) = current_service_sig.get() {
                    if cur == svc {
                        reload_instances_cb();
                    }
                }
            } else {
                reload_services_cb();
            }
        }) {
            Ok(h) => sse_handle.set(Some(h)),
            Err(e) => web_sys::console::error_1(&format!("SSE 订阅失败: {}", e).into()),
        }
    }

    let on_register = {
        let reload_services = reload_services.clone();
        let reload_instances = reload_instances.clone();
        move |_| {
            let service_name = {
                let v = form_service.get();
                if v.trim().is_empty() {
                    if let Some(cur) = current_service.get() { cur } else { v }
                } else { v }
            };
            let ip = form_ip.get();
            let port = form_port.get().parse::<u16>().ok();
            let weight = form_weight.get().parse::<f64>().ok();
            if service_name.trim().is_empty() || ip.trim().is_empty() || port.is_none() {
                set_error.set(Some("请填写服务名、IP、端口".to_string()));
                return;
            }
            let group = form_group.get();
            let cluster = form_cluster.get();
            let mut metadata: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            let metadata_text = form_metadata.get();
            if !metadata_text.trim().is_empty() {
                match serde_json::from_str::<serde_json::Value>(&metadata_text) {
                    Ok(val) => {
                        if let Some(map) = val.as_object() {
                            for (k, v) in map {
                                metadata.insert(k.clone(), v.as_str().unwrap_or("").to_string());
                            }
                        }
                    }
                    Err(_) => {
                        set_error.set(Some("元数据需为 JSON 对象".to_string()));
                        return;
                    }
                }
            }
            set_creating.set(true);
            spawn_local(async move {
                let req = RegisterInstanceRequest {
                    ip, port: port.unwrap(), service_name: service_name.clone(),
                    group_name: if group.trim().is_empty() { None } else { Some(group.clone()) },
                    cluster_name: if cluster.trim().is_empty() { None } else { Some(cluster.clone()) },
                    weight,
                    metadata: if metadata.is_empty() { None } else { Some(metadata) },
                };
                match ApiClient::new().register_instance(req).await {
                    Ok(_) => {
                        set_info.set(Some("实例注册成功".to_string()));
                        set_error.set(None);
                        set_create_open.set(false);
                        reload_services();
                        reload_instances();
                    }
                    Err(e) => set_error.set(Some(format!("注册失败: {}", e))),
                }
                set_creating.set(false);
            });
        }
    };

    let on_deregister = move |instance_id: String| {
        if let Some(service) = current_service.get() {
            if !web_sys::window().and_then(|w| w.confirm_with_message("确定注销该实例吗？").ok()).unwrap_or(false) {
                return;
            }
            spawn_local(async move {
                match ApiClient::new().deregister_instance(&service, &instance_id).await {
                    Ok(true) => {
                        set_info.set(Some("实例注销成功".to_string()));
                        set_error.set(None);
                        // 刷新
                        match ApiClient::new().get_instances(Some(&service)).await {
                            Ok(list) => set_instances.set(list),
                            Err(_) => set_instances.set(vec![]),
                        }
                        // 也可能影响服务列表（当最后一个实例注销）
                        match ApiClient::new().list_services().await {
                            Ok(list) => set_services.set(list),
                            Err(_) => {}
                        }
                    }
                    Ok(false) => set_error.set(Some("注销失败".to_string())),
                    Err(e) => set_error.set(Some(format!("注销失败: {}", e))),
                }
            });
        }
    };

    let on_beat = move |instance_id: String| {
        if let Some(service) = current_service.get() {
            spawn_local(async move {
                match ApiClient::new().beat(&service, &instance_id).await {
                    Ok(true) => {
                        set_info.set(Some("心跳已发送".to_string()));
                        set_error.set(None);
                        // 可选：轻量刷新实例健康
                        match ApiClient::new().get_instances(Some(&service)).await {
                            Ok(list) => set_instances.set(list),
                            Err(_) => {}
                        }
                    }
                    Ok(false) => set_error.set(Some("心跳失败".to_string())),
                    Err(e) => set_error.set(Some(format!("心跳失败: {}", e))),
                }
            });
        }
    };
    let on_detail = move |ins: Instance| {
        set_detail_instance.set(Some(ins));
        set_detail_open.set(true);
    };

    view! {
        <div class="container mt-4">
            <h2 class="page-title">
                <i class="bi bi-server"></i> {" 服务管理"}
            </h2>
            {move || if let Some(m) = info.get() { view!{<div class="alert alert-success py-2">{m}</div>}.into_view() } else { view!{<></>}.into_view() }}
            {move || if let Some(m) = error.get() { view!{<div class="alert alert-danger py-2">{m}</div>}.into_view() } else { view!{<></>}.into_view() }}
            <div class="card mb-3">
                <div class="card-header d-flex justify-content-between align-items-center">
                    <span>{"注册实例"}</span>
                    <div class="d-flex gap-2">
                        <button class="btn btn-sm btn-outline-secondary" on:click=move |_| { reload_services(); reload_instances(); }>
                            <i class="bi bi-arrow-clockwise"></i> {" 刷新"}
                        </button>
                        <button class="btn btn-sm btn-primary" on:click=move |_| {
                            if let Some(cur) = current_service.get() { set_form_service.set(cur); }
                            set_create_open.set(!create_open.get());
                        }>
                            {move || if create_open.get() { "收起表单" } else { "展开表单" }}
                        </button>
                    </div>
                </div>
                {move || if create_open.get() {
                    view!{
                        <div class="card-body">
                            <div class="row g-2">
                                <div class="col-md-4">
                                    <label class="form-label">{"服务名"}</label>
                                    <input class="form-control" prop:value=form_service on:input=move |e| set_form_service.set(event_target_value(&e)) />
                                </div>
                                <div class="col-md-4">
                                    <label class="form-label">{"IP"}</label>
                                    <input class="form-control" prop:value=form_ip on:input=move |e| set_form_ip.set(event_target_value(&e)) />
                                </div>
                                <div class="col-md-4">
                                    <label class="form-label">{"端口"}</label>
                                    <input class="form-control" prop:value=form_port on:input=move |e| set_form_port.set(event_target_value(&e)) />
                                </div>
                            </div>
                            <div class="row g-2 mt-2">
                                <div class="col-md-4">
                                    <label class="form-label">{"分组"}</label>
                                    <input class="form-control" prop:value=form_group on:input=move |e| set_form_group.set(event_target_value(&e)) />
                                </div>
                                <div class="col-md-4">
                                    <label class="form-label">{"集群"}</label>
                                    <input class="form-control" prop:value=form_cluster on:input=move |e| set_form_cluster.set(event_target_value(&e)) />
                                </div>
                                <div class="col-md-4">
                                    <label class="form-label">{"权重"}</label>
                                    <input class="form-control" prop:value=form_weight on:input=move |e| set_form_weight.set(event_target_value(&e)) />
                                </div>
                            </div>
                            <div class="mt-2">
                                <label class="form-label">{"元数据（JSON 对象）"}</label>
                                <textarea class="form-control" rows=4 prop:value=form_metadata on:input=move |e| set_form_metadata.set(event_target_value(&e)) />
                            </div>
                            <div class="mt-3">
                                <button class="btn btn-primary" on:click=on_register disabled=move || creating.get()>
                                    {move || if creating.get() { view!{<span class="spinner-border spinner-border-sm me-1"></span>}.into_view() } else { view!{<></>}.into_view() }}
                                    {" 注册"}
                                </button>
                            </div>
                        </div>
                    }.into_view()
                } else { view!{<></>}.into_view() }}
            </div>
            <div class="row">
                <div class="col-md-4">
                    <div class="card">
                        <div class="card-header">
                            <h5 class="card-title mb-0">{"服务列表"}</h5>
                        </div>
                        <div class="card-body p-0">
                            {move || if loading.get() {
                                view! { <div class="p-3"><Loading /></div> }.into_view()
                            } else {
                                let list = services.get();
                                view! {
                                    <div class="list-group list-group-flush">
                                        {if list.is_empty() {
                                            view! { <div class="text-center p-3 text-muted">{"暂无服务"}</div> }.into_view()
                                        } else {
                                            list.into_iter().map(|svc| {
                                                let svc_clone = svc.clone();
                                                let active = current_service.get().as_ref().is_some_and(|s| s == &svc_clone);
                                                view! {
                                                    <a href="#"
                                                       class={format!("list-group-item list-group-item-action {}", if active { "active" } else { "" })}
                                                       on:click=move |e| { e.prevent_default(); on_select(svc_clone.clone()); }>
                                                        <div class="d-flex w-100 justify-content-between">
                                                            <h6 class="mb-1">{svc.clone()}</h6>
                                                            <small class="text-muted">{"点击查看详情"}</small>
                                                        </div>
                                                    </a>
                                                }
                                            }).collect_view()
                                        }}
                                    </div>
                                }.into_view()
                            }}
                        </div>
                    </div>
                </div>
                <div class="col-md-8">
                    <div class="card">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <h5 class="card-title mb-0">
                                {move || current_service.get().unwrap_or_else(|| "选择服务查看详情".to_string())}
                            </h5>
                        </div>
                        <div class="card-body">
                            {move || {
                                let list = instances.get();
                                if current_service.get().is_none() {
                                    view! {
                                        <div class="text-center text-muted p-5">
                                            <i class="bi bi-server" style="font-size: 3rem;"></i>
                                            <p class="mt-3">{"请从左侧选择一个服务查看详情"}</p>
                                        </div>
                                    }.into_view()
                                } else if list.is_empty() {
                                    view! {
                                        <div class="text-center text-muted p-5">
                                            <i class="bi bi-server" style="font-size: 3rem;"></i>
                                            <p class="mt-3">{"该服务暂无实例"}</p>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <div class="table-responsive">
                                            <table class="table table-hover">
                                                <thead>
                                                    <tr>
                                                        <th>{"实例ID"}</th>
                                                        <th>{"地址"}</th>
                                                        <th>{"分组"}</th>
                                                        <th>{"集群"}</th>
                                                        <th>{"健康"}</th>
                                                        <th class="text-end">{"操作"}</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {list.into_iter().map(|ins| {
                                                        let addr = format!("{}:{}", ins.ip, ins.port);
                                                        let ins_for_detail = ins.clone();
                                                        let id_for_beat = ins.id.clone();
                                                        let id_for_dereg = ins.id.clone();
                                                        view! {
                                                            <tr>
                                                                <td><code>{ins.id.clone()}</code></td>
                                                                <td>{addr}</td>
                                                                <td>{ins.group_name}</td>
                                                                <td>{ins.cluster_name}</td>
                                                                <td>
                                                                    <span class={format!("badge {}", if ins.healthy { "bg-success" } else { "bg-danger" })}>
                                                                        {if ins.healthy { "健康" } else { "不健康" }}
                                                                    </span>
                                                                </td>
                                                                <td class="text-end">
                                                                    <button class="btn btn-sm btn-outline-secondary me-2"
                                                                            on:click=move |_| on_detail(ins_for_detail.clone())>
                                                                        <i class="bi bi-eye"></i> {" 详情"}
                                                                    </button>
                                                                    <button class="btn btn-sm btn-outline-primary me-2"
                                                                            on:click=move |_| on_beat(id_for_beat.clone())>
                                                                        <i class="bi bi-heart-pulse"></i> {" 心跳"}
                                                                    </button>
                                                                    <button class="btn btn-sm btn-outline-danger"
                                                                            on:click=move |_| on_deregister(id_for_dereg.clone())>
                                                                        <i class="bi bi-trash"></i> {" 注销"}
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_view()
                                }
                            }}
                        </div>
                    </div>
                </div>
            </div>
            {move || if detail_open.get() {
                if let Some(ins) = detail_instance.get() {
                    let metadata = if ins.metadata.is_empty() { "{}".to_string() } else {
                        serde_json::to_string_pretty(&ins.metadata).unwrap_or_else(|_| "{}".to_string())
                    };
                    view!{
                        <div class="card mt-3">
                            <div class="card-header d-flex justify-content-between align-items-center">
                                <span>{"实例详情"}</span>
                                <button class="btn btn-sm btn-secondary" on:click=move |_| set_detail_open.set(false)>{"关闭"}</button>
                            </div>
                            <div class="card-body">
                                <div class="row">
                                    <div class="col-md-6">
                                        <table class="table table-borderless">
                                            <tr><td><strong>{"实例ID:"}</strong></td><td><code>{ins.id.clone()}</code></td></tr>
                                            <tr><td><strong>{"服务名:"}</strong></td><td>{ins.service_name.clone()}</td></tr>
                                            <tr><td><strong>{"地址:"}</strong></td><td>{format!("{}:{}", ins.ip, ins.port)}</td></tr>
                                            <tr><td><strong>{"分组:"}</strong></td><td>{ins.group_name.clone()}</td></tr>
                                        </table>
                                    </div>
                                    <div class="col-md-6">
                                        <table class="table table-borderless">
                                            <tr><td><strong>{"集群:"}</strong></td><td>{ins.cluster_name.clone()}</td></tr>
                                            <tr><td><strong>{"权重:"}</strong></td><td>{ins.weight}</td></tr>
                                            <tr><td><strong>{"健康:"}</strong></td><td>{if ins.healthy { "健康" } else { "不健康" }}</td></tr>
                                            <tr><td><strong>{"最后心跳:"}</strong></td><td>{ins.last_beat_time.clone()}</td></tr>
                                        </table>
                                    </div>
                                </div>
                                <div class="mt-2">
                                    <strong>{"元数据:"}</strong>
                                    <pre class="bg-light p-2 mt-2"><code>{metadata}</code></pre>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else { view!{<></>}.into_view() }
            } else { view!{<></>}.into_view() }}
        </div>
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn Services() {}