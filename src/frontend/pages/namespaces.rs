#[cfg(target_arch = "wasm32")]
use leptos::*;
use crate::frontend::components::Loading;
#[cfg(target_arch = "wasm32")]
use crate::frontend::services::{ApiClient, Namespace, CreateNamespaceRequest, UpdateNamespaceRequest};

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Namespaces() -> impl IntoView {
    let api = ApiClient::new();
    let (loading, set_loading) = create_signal(true);
    let (namespaces, set_namespaces) = create_signal::<Vec<Namespace>>(vec![]);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (info, set_info) = create_signal::<Option<String>>(None);

    // 创建表单
    let (create_id, set_create_id) = create_signal(String::new());
    let (create_show_name, set_create_show_name) = create_signal(String::new());
    let (create_desc, set_create_desc) = create_signal(String::new());
    let (creating, set_creating) = create_signal(false);

    // 编辑表单
    let (edit_target, set_edit_target) = create_signal::<Option<String>>(None);
    let (edit_show_name, set_edit_show_name) = create_signal(String::new());
    let (edit_desc, set_edit_desc) = create_signal(String::new());
    let (edit_quota, set_edit_quota) = create_signal(String::new());
    let (updating, set_updating) = create_signal(false);

    // 加载命名空间
    let reload = {
        let set_namespaces = set_namespaces.clone();
        let set_error = set_error.clone();
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                match ApiClient::new().list_namespaces().await {
                    Ok(list) => set_namespaces.set(list),
                    Err(e) => set_error.set(Some(e)),
                }
                set_loading.set(false);
            });
        }
    };
    reload();

    // 创建
    let on_create = {
        let reload = reload.clone();
        move |_| {
            let id = create_id.get();
            let show = create_show_name.get();
            let desc = create_desc.get();
            if id.trim().is_empty() || show.trim().is_empty() {
                set_error.set(Some("请填写必填字段".to_string()));
                return;
            }
            set_creating.set(true);
            spawn_local(async move {
                let req = CreateNamespaceRequest {
                    namespace: id.clone(),
                    namespace_show_name: show.clone(),
                    namespace_desc: if desc.trim().is_empty() { None } else { Some(desc.clone()) },
                };
                match ApiClient::new().create_namespace(req).await {
                    Ok(true) => {
                        set_info.set(Some("命名空间创建成功".to_string()));
                        set_error.set(None);
                        // 清空
                        set_create_id.set(String::new());
                        set_create_show_name.set(String::new());
                        set_create_desc.set(String::new());
                        reload();
                    }
                    Ok(false) => set_error.set(Some("创建失败".to_string())),
                    Err(e) => set_error.set(Some(format!("创建失败: {}", e))),
                }
                set_creating.set(false);
            });
        }
    };

    // 打开编辑
    let open_edit = move |ns: Namespace| {
        set_edit_target.set(Some(ns.namespace));
        set_edit_show_name.set(ns.namespace_show_name);
        set_edit_desc.set(ns.namespace_desc);
        set_edit_quota.set(ns.quota.to_string());
    };

    // 保存编辑
    let on_update = {
        let reload = reload.clone();
        move |_| {
            if let Some(target) = edit_target.get() {
                let show = edit_show_name.get();
                if show.trim().is_empty() {
                    set_error.set(Some("请填写显示名称".to_string()));
                    return;
                }
                let desc = edit_desc.get();
                let quota_text = edit_quota.get();
                let quota_val = quota_text.parse::<u32>().ok();
                set_updating.set(true);
                spawn_local(async move {
                    let req = UpdateNamespaceRequest {
                        namespace_show_name: show.clone(),
                        namespace_desc: if desc.trim().is_empty() { None } else { Some(desc.clone()) },
                        quota: quota_val,
                    };
                    match ApiClient::new().update_namespace(&target, req).await {
                        Ok(true) => {
                            set_info.set(Some("命名空间更新成功".to_string()));
                            set_error.set(None);
                            set_edit_target.set(None);
                            reload();
                        }
                        Ok(false) => set_error.set(Some("更新失败".to_string())),
                        Err(e) => set_error.set(Some(format!("更新失败: {}", e))),
                    }
                    set_updating.set(false);
                });
            }
        }
    };

    // 删除
    let on_delete = {
        let reload = reload.clone();
        move |ns: String| {
            if ns == "public" {
                set_error.set(Some("不能删除默认命名空间".to_string()));
                return;
            }
            if !web_sys::window().and_then(|w| w.confirm_with_message(&format!("确定删除命名空间 \"{}\" 及其下所有配置吗？", ns)).ok()).unwrap_or(false) {
                return;
            }
            spawn_local(async move {
                match ApiClient::new().delete_namespace(&ns).await {
                    Ok(true) => {
                        set_info.set(Some("命名空间删除成功".to_string()));
                        set_error.set(None);
                        reload();
                    }
                    Ok(false) => set_error.set(Some("删除失败".to_string())),
                    Err(e) => set_error.set(Some(format!("删除失败: {}", e))),
                }
            });
        }
    };
    
    view! {
        <div class="container mt-4">
            <h2 class="page-title">
                <i class="bi bi-folder"></i> {" 命名空间管理"}
            </h2>
            // 提示
            {move || if let Some(msg) = info.get() {
                view! { <div class="alert alert-success py-2">{msg}</div> }.into_view()
            } else { view!{<></>}.into_view() }}
            {move || if let Some(msg) = error.get() {
                view! { <div class="alert alert-danger py-2">{msg}</div> }.into_view()
            } else { view!{<></>}.into_view() }}

            // 新增表单
            <div class="card mb-3">
                <div class="card-header d-flex justify-content-between align-items-center">
                    <span>{"创建命名空间"}</span>
                    <button class="btn btn-sm btn-outline-secondary" on:click=move |_| { reload(); }>
                        <i class="bi bi-arrow-clockwise"></i>{" 刷新"}
                    </button>
                </div>
                <div class="card-body">
                    <div class="row">
                        <div class="col-md-4">
                            <label class="form-label">{"命名空间ID"}</label>
                            <input class="form-control"
                                   placeholder="例如: dev、test、prod"
                                   prop:value=create_id
                                   on:input=move |e| set_create_id.set(event_target_value(&e)) />
                            <div class="form-text">{"只能包含字母、数字、下划线和连字符（后端未强校验时请自觉遵守）"}</div>
                        </div>
                        <div class="col-md-4">
                            <label class="form-label">{"显示名称"}</label>
                            <input class="form-control"
                                   placeholder="例如: 开发环境"
                                   prop:value=create_show_name
                                   on:input=move |e| set_create_show_name.set(event_target_value(&e)) />
                        </div>
                        <div class="col-md-4">
                            <label class="form-label">{"描述"}</label>
                            <input class="form-control"
                                   placeholder="描述信息"
                                   prop:value=create_desc
                                   on:input=move |e| set_create_desc.set(event_target_value(&e)) />
                        </div>
                    </div>
                    <div class="mt-3">
                        <button class="btn btn-primary"
                                disabled=move || creating.get()
                                on:click=on_create>
                            {move || if creating.get() { view!{<span class="spinner-border spinner-border-sm me-1"></span>}.into_view() } else { view!{<></>}.into_view() }}
                            {" 创建"}
                        </button>
                    </div>
                </div>
            </div>

            <div class="card">
                <div class="card-body">
                    {move || if loading.get() {
                        view! { <Loading /> }.into_view()
                    } else if let Some(msg) = error.get() {
                        view! { <div class="alert alert-danger">{msg}</div> }.into_view()
                    } else {
                        let ns_list = namespaces.get();
                        view! {
                            <div class="table-responsive">
                                <table class="table table-hover">
                                    <thead>
                                        <tr>
                                            <th>{"命名空间"}</th>
                                            <th>{"显示名称"}</th>
                                            <th>{"描述"}</th>
                                            <th>{"配额"}</th>
                                            <th class="text-end">{"操作"}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {ns_list.into_iter().map(|ns| {
                                            let id = ns.namespace.clone();
                                            let is_editing = edit_target.get().as_ref().is_some_and(|t| t == &id);
                                            if is_editing {
                                                view! {
                                                    <tr>
                                                        <td><code>{id.clone()}</code></td>
                                                        <td>
                                                            <input class="form-control form-control-sm"
                                                                   prop:value=edit_show_name
                                                                   on:input=move |e| set_edit_show_name.set(event_target_value(&e)) />
                                                        </td>
                                                        <td>
                                                            <input class="form-control form-control-sm"
                                                                   prop:value=edit_desc
                                                                   on:input=move |e| set_edit_desc.set(event_target_value(&e)) />
                                                        </td>
                                                        <td style="max-width:120px;">
                                                            <input class="form-control form-control-sm"
                                                                   prop:value=edit_quota
                                                                   on:input=move |e| set_edit_quota.set(event_target_value(&e)) />
                                                        </td>
                                                        <td class="text-end">
                                                            <button class="btn btn-sm btn-primary me-2"
                                                                    disabled=move || updating.get()
                                                                    on:click=on_update>
                                                                {move || if updating.get() { view!{<span class="spinner-border spinner-border-sm me-1"></span>}.into_view() } else { view!{<></>}.into_view() }}
                                                                {"保存"}
                                                            </button>
                                                            <button class="btn btn-sm btn-secondary" on:click=move |_| set_edit_target.set(None)>{"取消"}</button>
                                                        </td>
                                                    </tr>
                                                }.into_view()
                                            } else {
                                                let ns_clone_for_edit = ns.clone();
                                                view! {
                                                    <tr>
                                                        <td><code>{ns.namespace.clone()}</code></td>
                                                        <td>{ns.namespace_show_name}</td>
                                                        <td>{ns.namespace_desc}</td>
                                                        <td>{ns.quota}</td>
                                                        <td class="text-end">
                                                            <button class="btn btn-sm btn-outline-primary me-2"
                                                                    on:click=move |_| open_edit(ns_clone_for_edit.clone())>
                                                                <i class="bi bi-pencil"></i> {" 编辑"}
                                                            </button>
                                                            <button class="btn btn-sm btn-outline-danger"
                                                                    disabled=ns.namespace == "public"
                                                                    on:click=move |_| on_delete(ns.namespace.clone())>
                                                                <i class="bi bi-trash"></i> {" 删除"}
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }.into_view()
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        }.into_view()
                    }}
                </div>
            </div>
        </div>
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn Namespaces() {}