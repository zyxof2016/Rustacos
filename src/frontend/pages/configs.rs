#[cfg(target_arch = "wasm32")]
use leptos::*;
use crate::frontend::components::Loading;
#[cfg(target_arch = "wasm32")]
use crate::frontend::services::{ApiResponse, ConfigItem, Namespace, ApiClient, PublishConfigRequest, SseHandle};
#[cfg(target_arch = "wasm32")]
use serde_json::Value;
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsValue, JsCast};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use js_sys::Function;

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Configs() -> impl IntoView {
    #[derive(Clone, Debug, serde::Deserialize)]
    struct PagedConfigResponse {
        total_count: usize,
        page_number: u32,
        page_size: u32,
        pages: u32,
        data: Vec<ConfigItem>,
    }

    let (loading, set_loading) = create_signal(true);
    let (namespaces, set_namespaces) = create_signal::<Vec<Namespace>>(vec![]);
    let (current_ns, set_current_ns) = create_signal::<String>("public".to_string());
    let (configs, set_configs) = create_signal::<Vec<ConfigItem>>(vec![]);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (info, set_info) = create_signal::<Option<String>>(None);
    let (search, set_search) = create_signal::<String>(String::new());
    let (page, set_page) = create_signal::<u32>(1);
    let (size, set_size) = create_signal::<u32>(10);
    let (total_pages, set_total_pages) = create_signal::<u32>(0);
    let (total_count, set_total_count) = create_signal::<usize>(0);
    let (view_open, set_view_open) = create_signal(false);
    let (view_content, set_view_content) = create_signal(String::new());
    let (view_type, set_view_type) = create_signal(String::from("text"));
    let (history_open, set_history_open) = create_signal(false);
    let (history_items, set_history_items) = create_signal::<Vec<(i64, bool, String)>>(vec![]);
    let (left_ver, set_left_ver) = create_signal::<Option<i64>>(None);
    let (right_ver, set_right_ver) = create_signal::<Option<i64>>(None);
    let (sbs_open, set_sbs_open) = create_signal(false);
    let (sbs_html, set_sbs_html) = create_signal(String::new());
    let (diff_open, set_diff_open) = create_signal(false);
    let (diff_html, set_diff_html) = create_signal(String::new());
    // SSE 句柄
    let sse_handle = create_rw_signal::<Option<SseHandle>>(None);

    // 加载命名空间 + 当前命名空间配置
    spawn_local(async move {
        let api = crate::frontend::services::ApiClient::new();
        match api.list_namespaces().await {
            Ok(list) => set_namespaces.set(list),
            Err(e) => set_error.set(Some(e)),
        }
        set_loading.set(false);
    });

    // 根据命名空间加载配置列表（第一页，size=10）
    let load_configs = move |ns: String| {
        let set_configs_cloned = set_configs.clone();
        let set_error_cloned = set_error.clone();
        let keyword = search.get();
        let page_now = page.get();
        let size_now = size.get();
        let set_total_pages_cloned = set_total_pages.clone();
        let set_total_count_cloned = set_total_count.clone();
        spawn_local(async move {
            let url = format!(
                "/nacos/v1/cs/configs/list?namespace={}&page={}&size={}&search={}",
                ns, page_now, size_now, urlencoding::encode(&keyword)
            );
            let req = {
                if let Some(token) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|s| s.get_item("accessToken").ok().flatten()) {
                    Request::get(&url).header("Authorization", &format!("Bearer {}", token))
                } else {
                    Request::get(&url)
                }
            };
            match req.send().await {
                Ok(resp) => {
                    if resp.ok() {
                        let wrapped: ApiResponse<PagedConfigResponse> = match resp.json().await {
                            Ok(v) => v,
                            Err(e) => { set_error_cloned.set(Some(format!("解析失败: {}", e))); return; }
                        };
                        if let Some(paged) = wrapped.data {
                            set_configs_cloned.set(paged.data);
                            set_total_pages_cloned.set(paged.pages);
                            set_total_count_cloned.set(paged.total_count);
                        } else {
                            set_configs_cloned.set(vec![]);
                            set_total_pages_cloned.set(0);
                            set_total_count_cloned.set(0);
                        }
                    } else {
                        set_error_cloned.set(Some(format!("请求失败: {}", resp.status())));
                    }
                }
                Err(e) => set_error_cloned.set(Some(format!("网络错误: {}", e))),
            }
        });
    };

    // 初次加载 public
    load_configs(current_ns.get());

    // 订阅配置变更：同命名空间则刷新列表
    {
        let current_ns_sig = current_ns.clone();
        let load_configs_cb = load_configs.clone();
        match ApiClient::subscribe_events("config", move |msg| {
            if let Ok(v) = serde_json::from_str::<Value>(&msg) {
                let ns = v.get("namespace").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                if ns == current_ns_sig.get() {
                    load_configs_cb(ns);
                }
            }
        }) {
            Ok(h) => sse_handle.set(Some(h)),
            Err(e) => web_sys::console::error_1(&format!("SSE 订阅失败: {}", e).into()),
        }
    }

    // 辅助：类型到 CodeMirror/hljs 模式
    fn cm_mode(t: &str) -> &str {
        match t {
            "json" => "javascript",
            "xml" => "xml",
            "yaml" | "yml" => "yaml",
            "properties" => "properties",
            "html" => "htmlmixed",
            _ => "text/plain",
        }
    }
    fn hljs_lang(t: &str) -> &str {
        match t {
            "json" => "json",
            "xml" => "xml",
            "yaml" | "yml" => "yaml",
            "properties" => "properties",
            "html" => "html",
            _ => "text",
        }
    }

    // 命名空间选择变更
    let on_change_ns = move |ns: String| {
        set_current_ns.set(ns.clone());
        set_page.set(1);
        load_configs(ns);
    };

    // 创建表单
    let (create_open, set_create_open) = create_signal(false);
    let (create_data_id, set_create_data_id) = create_signal(String::new());
    let (create_group, set_create_group) = create_signal(String::from("DEFAULT_GROUP"));
    let (create_ns, set_create_ns) = create_signal(String::from("public"));
    let (create_type, set_create_type) = create_signal(String::from("text"));
    let (create_content, set_create_content) = create_signal(String::new());
    let (creating, set_creating) = create_signal(false);
    // 创建编辑器初始化/销毁
    create_effect(move |_| {
        if create_open.get() {
            let mode = cm_mode(&create_type.get()).to_string();
            if let Some(w) = web_sys::window() {
                if let Ok(f) = js_sys::Reflect::get(&w, &JsValue::from_str("initCodeMirror")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        let _ = func.call2(&JsValue::NULL, &JsValue::from_str("createContentEditor"), &JsValue::from_str(&mode));
                    }
                }
            }
        } else {
            if let Some(w) = web_sys::window() {
                if let Ok(f) = js_sys::Reflect::get(&w, &JsValue::from_str("destroyCodeMirror")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        let _ = func.call1(&JsValue::NULL, &JsValue::from_str("createContentEditor"));
                    }
                }
            }
        }
    });
    // 切换类型时更新编辑器模式
    create_effect(move |_| {
        let _ = &create_type.get();
        if create_open.get() {
            let mode = cm_mode(&create_type.get()).to_string();
            if let Some(w) = web_sys::window() {
                if let Ok(f) = js_sys::Reflect::get(&w, &JsValue::from_str("initCodeMirror")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        let _ = func.call2(&JsValue::NULL, &JsValue::from_str("createContentEditor"), &JsValue::from_str(&mode));
                    }
                }
            }
        }
    });

    fn apply_ext(mut id: String, t: &str) -> String {
        if let Some(dot) = id.rfind('.') {
            id = id[..dot].to_string();
        }
        match t {
            "json" => format!("{}.json", id),
            "xml" => format!("{}.xml", id),
            "yaml" => format!("{}.yaml", id),
            "properties" => format!("{}.properties", id),
            "html" => format!("{}.html", id),
            _ => format!("{}.txt", id),
        }
    }

    let on_create = {
        let load_configs = load_configs.clone();
        move |_| {
            let data_id = create_data_id.get();
            let group = create_group.get();
            let ns_val = create_ns.get();
            let cfg_type = create_type.get();
            let content = create_content.get();

            if data_id.trim().is_empty() || content.trim().is_empty() {
                set_error.set(Some("请填写必填字段".to_string()));
                return;
            }
            let final_id = apply_ext(data_id, &cfg_type);
            set_creating.set(true);
            spawn_local(async move {
                let req = PublishConfigRequest {
                    data_id: final_id.clone(),
                    group: group.clone(),
                    content: content.clone(),
                    namespace: Some(ns_val.clone()),
                };
                match ApiClient::new().publish_config(req).await {
                    Ok(true) => {
                        set_info.set(Some("配置创建成功".to_string()));
                        set_error.set(None);
                        set_create_open.set(false);
                        set_create_data_id.set(String::new());
                        set_create_content.set(String::new());
                        load_configs(ns_val);
                    }
                    Ok(false) => set_error.set(Some("创建失败".to_string())),
                    Err(e) => set_error.set(Some(format!("创建失败: {}", e))),
                }
                set_creating.set(false);
            });
        }
    };

    // 编辑表单
    let (edit_open, set_edit_open) = create_signal(false);
    let (edit_original_id, set_edit_original_id) = create_signal(String::new());
    let (edit_original_group, set_edit_original_group) = create_signal(String::new());
    let (edit_original_ns, set_edit_original_ns) = create_signal(String::new());
    let (edit_data_id, set_edit_data_id) = create_signal(String::new());
    let (edit_group, set_edit_group) = create_signal(String::new());
    let (edit_ns, set_edit_ns) = create_signal(String::new());
    let (edit_type, set_edit_type) = create_signal(String::from("text"));
    let (edit_content, set_edit_content) = create_signal(String::new());
    let (updating, set_updating) = create_signal(false);
    // 编辑器初始化/销毁
    create_effect(move |_| {
        if edit_open.get() {
            let mode = cm_mode(&edit_type.get()).to_string();
            if let Some(w) = web_sys::window() {
                if let Ok(f) = js_sys::Reflect::get(&w, &JsValue::from_str("initCodeMirror")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        let _ = func.call2(&JsValue::NULL, &JsValue::from_str("editContentEditor"), &JsValue::from_str(&mode));
                    }
                }
            }
        } else {
            if let Some(w) = web_sys::window() {
                if let Ok(f) = js_sys::Reflect::get(&w, &JsValue::from_str("destroyCodeMirror")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        let _ = func.call1(&JsValue::NULL, &JsValue::from_str("editContentEditor"));
                    }
                }
            }
        }
    });
    // 切换类型时更新编辑器模式
    create_effect(move |_| {
        let _ = &edit_type.get();
        if edit_open.get() {
            let mode = cm_mode(&edit_type.get()).to_string();
            if let Some(w) = web_sys::window() {
                if let Ok(f) = js_sys::Reflect::get(&w, &JsValue::from_str("initCodeMirror")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        let _ = func.call2(&JsValue::NULL, &JsValue::from_str("editContentEditor"), &JsValue::from_str(&mode));
                    }
                }
            }
        }
    });

    let open_edit = move |c: ConfigItem| {
        set_edit_original_id.set(c.data_id.clone());
        set_edit_original_group.set(c.group.clone());
        set_edit_original_ns.set(c.namespace.clone());
        set_edit_data_id.set(c.data_id.clone());
        set_edit_group.set(c.group.clone());
        set_edit_ns.set(c.namespace.clone());
        // 简单推断类型
        let ty = if c.data_id.ends_with(".json") {"json"}
                 else if c.data_id.ends_with(".xml") {"xml"}
                 else if c.data_id.ends_with(".yaml") {"yaml"}
                 else if c.data_id.ends_with(".properties") {"properties"}
                 else if c.data_id.ends_with(".html") {"html"}
                 else {"text"};
        set_edit_type.set(ty.to_string());
        set_edit_content.set(c.content.clone());
        set_edit_open.set(true);
    };
    let open_view = move |c: ConfigItem| {
        // 推断语言
        let ty = if c.data_id.ends_with(".json") {"json"}
                 else if c.data_id.ends_with(".xml") {"xml"}
                 else if c.data_id.ends_with(".yaml") {"yaml"}
                 else if c.data_id.ends_with(".properties") {"properties"}
                 else if c.data_id.ends_with(".html") {"html"}
                 else {"text"};
        set_view_type.set(ty.to_string());
        set_view_content.set(c.content.clone());
        set_view_open.set(true);
        // 延迟高亮
        gloo_timers::callback::Timeout::new(50, move || {
            if let Some(w) = web_sys::window() {
                if let Ok(f) = js_sys::Reflect::get(&w, &JsValue::from_str("highlightCode")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        let _ = func.call1(&JsValue::NULL, &JsValue::from_str("viewCode"));
                    }
                }
            }
        }).forget();
    };

    // 历史记录
    let open_history = move |c: ConfigItem| {
        let url = format!("/nacos/v1/cs/configs/history?data_id={}&group={}&namespace={}", c.data_id, c.group, c.namespace);
        let set_history_items = set_history_items.clone();
        spawn_local(async move {
            let req = {
                if let Some(token) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|s| s.get_item("accessToken").ok().flatten()) {
                    Request::get(&url).header("Authorization", &format!("Bearer {}", token))
                } else {
                    Request::get(&url)
                }
            };
            match req.send().await {
                Ok(resp) => {
                    if resp.ok() {
                        #[derive(serde::Deserialize)]
                        struct HItem { version: i64, deleted: bool, content: String }
                        #[derive(serde::Deserialize)]
                        struct Wrap { code: i32, data: Option<Vec<HItem>> }
                        if let Ok(w) = resp.json::<Wrap>().await {
                            if let Some(list) = w.data {
                                let mapped = list.into_iter().map(|h| (h.version, h.deleted, h.content)).collect();
                                set_history_items.set(mapped);
                                // 默认选择最近两个版本
                                let items = history_items.get();
                                let mut vers: Vec<i64> = items.iter().map(|(v,_,_)| *v).collect();
                                vers.sort();
                                vers.reverse();
                                set_left_ver.set(vers.get(1).cloned());
                                set_right_ver.set(vers.get(0).cloned());
                            } else {
                                set_history_items.set(vec![]);
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        });
        set_history_open.set(true);
    };
    let do_rollback = move |c: ConfigItem, version: i64| {
        if !web_sys::window().and_then(|w| w.confirm_with_message(&format!("回滚到版本 {} ？", version)).ok()).unwrap_or(false) {
            return;
        }
        let ns = c.namespace.clone();
        spawn_local(async move {
            let url = "/nacos/v1/cs/configs/history/rollback";
            let body = serde_json::json!({
                "data_id": c.data_id,
                "group": c.group,
                "namespace": c.namespace,
                "version": version
            });
            let req = {
                if let Some(token) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|s| s.get_item("accessToken").ok().flatten()) {
                    Request::post(url).header("Authorization", &format!("Bearer {}", token)).header("Content-Type", "application/json")
                } else {
                    Request::post(url).header("Content-Type", "application/json")
                }
            };
            match req.body(body.to_string()) {
                Ok(req2) => {
                    match req2.send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                set_info.set(Some("回滚成功".to_string()));
                                set_error.set(None);
                                set_history_open.set(false);
                                load_configs(ns);
                            } else {
                                set_error.set(Some("回滚失败".to_string()));
                            }
                        }
                        Err(e) => set_error.set(Some(format!("回滚失败: {}", e))),
                    }
                }
                Err(e) => set_error.set(Some(format!("回滚失败: {}", e))),
            }
        });
    };

    // 历史 vs 历史 并排对比
    let on_sbs_compare = move |_| {
        if let (Some(lv), Some(rv)) = (left_ver.get(), right_ver.get()) {
            let left_content = history_items.get().iter().find(|(v,_,_)| *v == lv).map(|(_,_,c)| c.clone()).unwrap_or_default();
            let right_content = history_items.get().iter().find(|(v,_,_)| *v == rv).map(|(_,_,c)| c.clone()).unwrap_or_default();
            if let Some(w) = web_sys::window() {
                if let Ok(f) = js_sys::Reflect::get(&w, &JsValue::from_str("createSideBySideDiffHtml")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        if let Ok(res) = func.call2(&JsValue::NULL, &JsValue::from_str(&left_content), &JsValue::from_str(&right_content)) {
                            if let Some(html) = res.as_string() {
                                set_sbs_html.set(html);
                                set_sbs_open.set(true);
                            }
                        }
                    }
                }
            }
        }
    };

    // 生成 Diff（历史 vs 当前）
    let open_diff = move |c: ConfigItem, version: i64| {
        // 拿当前内容
        let current_req_ns = c.namespace.clone();
        let current_req_group = c.group.clone();
        let current_req_id = c.data_id.clone();
        // 找历史内容
        let maybe_hist = history_items.get().into_iter().find(|(v, _, _)| *v == version).map(|(_, _, content)| content).unwrap_or_default();
        spawn_local(async move {
            let url = format!("/nacos/v1/cs/configs?data_id={}&group={}&namespace={}", current_req_id, current_req_group, current_req_ns);
            let req = {
                if let Some(token) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|s| s.get_item("accessToken").ok().flatten()) {
                    Request::get(&url).header("Authorization", &format!("Bearer {}", token))
                } else {
                    Request::get(&url)
                }
            };
            let mut current_content = String::new();
            if let Ok(resp) = req.send().await {
                if resp.ok() {
                    #[derive(serde::Deserialize)]
                    struct Wrap { code: i32, data: Option<ConfigItem> }
                    if let Ok(w) = resp.json::<Wrap>().await {
                        if let Some(ci) = w.data {
                            current_content = ci.content;
                        }
                    }
                }
            }
            // 调用 window.createDiffHtml
            if let Some(w) = web_sys::window() {
                if let Ok(f) = js_sys::Reflect::get(&w, &JsValue::from_str("createDiffHtml")) {
                    if let Ok(func) = f.dyn_into::<Function>() {
                        if let Ok(res) = func.call2(&JsValue::NULL, &JsValue::from_str(&maybe_hist), &JsValue::from_str(&current_content)) {
                            if let Some(html) = res.as_string() {
                                set_diff_html.set(html);
                                set_diff_open.set(true);
                            }
                        }
                    }
                }
            }
        });
    };

    // 导出/导入
    let on_export = {
        let current_ns = current_ns.clone();
        move |_| {
            let ns = current_ns.get();
            let url = format!("/nacos/v1/cs/configs/export?namespace={}", ns);
            spawn_local(async move {
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
                        if let Ok(text) = resp.text().await {
                            if let Some(w) = web_sys::window() {
                                let blob = web_sys::Blob::new_with_str_sequence(&js_sys::Array::of1(&JsValue::from_str(&text))).unwrap();
                                let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
                                let document = w.document().unwrap();
                                let a = document.create_element("a").unwrap();
                                a.set_attribute("href", &url).unwrap();
                                a.set_attribute("download", "configs.json").unwrap();
                                let a: web_sys::HtmlElement = a.dyn_into().unwrap();
                                a.click();
                                let _ = web_sys::Url::revoke_object_url(&url);
                            }
                        }
                    }
                }
            });
        }
    };
    let on_import = move |e: leptos::ev::Event| {
        let target: web_sys::EventTarget = event_target(&e);
        let input: web_sys::HtmlInputElement = target.unchecked_into();
        if let Some(file) = input.files().and_then(|fl| fl.get(0)) {
            let fr = web_sys::FileReader::new().unwrap();
            let onload = Closure::wrap(Box::new(move |ev: web_sys::Event| {
                let fr: web_sys::FileReader = ev.target().unwrap().dyn_into().unwrap();
                if let Ok(Some(text)) = fr.result().map(|r| r.as_string()) {
                    spawn_local(async move {
                        let url = "/nacos/v1/cs/configs/import";
                        let req = {
                            if let Some(token) = web_sys::window()
                                .and_then(|w| w.local_storage().ok().flatten())
                                .and_then(|s| s.get_item("accessToken").ok().flatten()) {
                                Request::post(url).header("Authorization", &format!("Bearer {}", token)).header("Content-Type", "application/json")
                            } else {
                                Request::post(url).header("Content-Type", "application/json")
                            }
                        };
                        match req.body(text) {
                            Ok(req2) => {
                                match req2.send().await {
                                    Ok(resp) => {
                                        if resp.ok() {
                                            set_info.set(Some("导入完成".to_string()));
                                            set_error.set(None);
                                        } else {
                                            set_error.set(Some("导入失败".to_string()));
                                        }
                                    }
                                    Err(e) => set_error.set(Some(format!("导入失败: {}", e))),
                                }
                            }
                            Err(e) => set_error.set(Some(format!("导入失败: {}", e))),
                        }
                    });
                }
            }) as Box<dyn FnMut(_)>);
            fr.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            fr.read_as_text(&file).unwrap();
        }
    };
    let on_update = {
        let load_configs = load_configs.clone();
        move |_| {
            let orig_id = edit_original_id.get();
            let orig_group = edit_original_group.get();
            let orig_ns = edit_original_ns.get();
            let data_id = edit_data_id.get();
            let group = edit_group.get();
            let ns_val = edit_ns.get();
            let cfg_type = edit_type.get();
            let content = edit_content.get();
            if data_id.trim().is_empty() || content.trim().is_empty() {
                set_error.set(Some("请填写必填字段".to_string()));
                return;
            }
            let final_id = apply_ext(data_id, &cfg_type);
            set_updating.set(true);
            spawn_local(async move {
                // 若 key 变化，先删旧，再发新；否则直接发布覆盖
                let key_changed = !(orig_id == final_id && orig_group == group && orig_ns == ns_val);
                if key_changed {
                    let _ = ApiClient::new().remove_config(&orig_id, &orig_group, &orig_ns).await;
                }
                let req = PublishConfigRequest {
                    data_id: final_id.clone(),
                    group: group.clone(),
                    content: content.clone(),
                    namespace: Some(ns_val.clone()),
                };
                match ApiClient::new().publish_config(req).await {
                    Ok(true) => {
                        set_info.set(Some("配置更新成功".to_string()));
                        set_error.set(None);
                        set_edit_open.set(false);
                        load_configs(ns_val);
                    }
                    Ok(false) => set_error.set(Some("更新失败".to_string())),
                    Err(e) => set_error.set(Some(format!("更新失败: {}", e))),
                }
                set_updating.set(false);
            });
        }
    };

    // 删除
    let on_delete = {
        let load_configs = load_configs.clone();
        move |c: ConfigItem| {
            if !web_sys::window().and_then(|w| w.confirm_with_message(&format!("确定要删除配置 \"{}\" 吗？", c.data_id)).ok()).unwrap_or(false) {
                return;
            }
            spawn_local(async move {
                match ApiClient::new().remove_config(&c.data_id, &c.group, &c.namespace).await {
                    Ok(true) => {
                        set_info.set(Some("配置删除成功".to_string()));
                        set_error.set(None);
                        load_configs(c.namespace);
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
                <i class="bi bi-gear"></i> {" 配置管理"}
            </h2>
            {move || if let Some(m) = info.get() { view!{<div class="alert alert-success py-2">{m}</div>}.into_view() } else { view!{<></>}.into_view() }}
            {move || if let Some(m) = error.get() { view!{<div class="alert alert-danger py-2">{m}</div>}.into_view() } else { view!{<></>}.into_view() }}

            <div class="card mb-3">
                <div class="card-body">
                    <div class="row g-2 align-items-end">
                        <div class="col-md-3">
                            <label class="form-label">{"命名空间"}</label>
                            <select class="form-select"
                                    on:change=move |e| on_change_ns(event_target_value(&e))>
                                {let cur = current_ns.get();
                                 namespaces.get().into_iter().map(|ns| {
                                    let selected = cur == ns.namespace;
                                    view!{ <option value={ns.namespace.clone()} selected={selected}>{format!("{} - {}", ns.namespace, ns.namespace_show_name)}</option> }
                                 }).collect_view()
                                }
                            </select>
                        </div>
                        <div class="col-md-4">
                            <label class="form-label">{"搜索（按数据ID包含）"}</label>
                            <input class="form-control" placeholder="关键字"
                                   prop:value=search
                                   on:input=move |e| set_search.set(event_target_value(&e)) />
                        </div>
                        <div class="col-md-2">
                            <label class="form-label">{"每页"}</label>
                            <select class="form-select"
                                    prop:value=move || size.get().to_string()
                                    on:change=move |e| {
                                        if let Ok(n) = event_target_value(&e).parse::<u32>() {
                                            set_size.set(n);
                                            set_page.set(1);
                                            load_configs(current_ns.get());
                                        }
                                    }>
                                <option value="10">{"10"}</option>
                                <option value="20">{"20"}</option>
                                <option value="50">{"50"}</option>
                            </select>
                        </div>
                        <div class="col-md-3 text-end">
                            <button class="btn btn-outline-secondary me-2" on:click=move |_| load_configs(current_ns.get())>
                                <i class="bi bi-arrow-clockwise"></i> {" 刷新"}
                            </button>
                            <button class="btn btn-outline-secondary me-2" on:click=on_export>
                                <i class="bi bi-download"></i> {" 导出"}
                            </button>
                            <label class="btn btn-outline-secondary me-2 mb-0">
                                <i class="bi bi-upload"></i> {" 导入"}
                                <input type="file" accept=".json" style="display:none" on:change=on_import />
                            </label>
                            <button class="btn btn-primary" on:click=move |_| {
                                set_create_ns.set(current_ns.get());
                                set_create_open.set(true);
                            }>
                                <i class="bi bi-plus-lg"></i> {" 新增配置"}
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            {move || if create_open.get() {
                view!{
                    <div class="card mb-3">
                        <div class="card-header">{"新增配置"}</div>
                        <div class="card-body">
                            <div class="row g-2">
                                <div class="col-md-4">
                                    <label class="form-label">{"数据ID"}</label>
                                    <input class="form-control"
                                           placeholder="如 application"
                                           prop:value=create_data_id
                                           on:input=move |e| set_create_data_id.set(event_target_value(&e)) />
                                </div>
                                <div class="col-md-4">
                                    <label class="form-label">{"分组"}</label>
                                    <input class="form-control"
                                           prop:value=create_group
                                           on:input=move |e| set_create_group.set(event_target_value(&e)) />
                                </div>
                                <div class="col-md-4">
                                    <label class="form-label">{"命名空间"}</label>
                                    <select class="form-select"
                                            prop:value=create_ns
                                            on:change=move |e| set_create_ns.set(event_target_value(&e))>
                                        {namespaces.get().into_iter().map(|ns| {
                                            view!{ <option value={ns.namespace.clone()}>{format!("{} - {}", ns.namespace, ns.namespace_show_name)}</option> }
                                        }).collect_view()}
                                    </select>
                                </div>
                            </div>
                            <div class="row g-2 mt-2">
                                <div class="col-md-3">
                                    <label class="form-label">{"配置格式"}</label>
                                    <select class="form-select"
                                            prop:value=create_type
                                            on:change=move |e| set_create_type.set(event_target_value(&e))>
                                        <option value="text">{"TEXT"}</option>
                                        <option value="json">{"JSON"}</option>
                                        <option value="xml">{"XML"}</option>
                                        <option value="yaml">{"YAML"}</option>
                                        <option value="properties">{"Properties"}</option>
                                        <option value="html">{"HTML"}</option>
                                    </select>
                                </div>
                            </div>
                            <div class="mt-2">
                                <label class="form-label">{"配置内容"}</label>
                                <textarea id="createContentEditor" class="form-control" rows=10
                                          prop:value=create_content
                                          on:input=move |e| set_create_content.set(event_target_value(&e)) />
                            </div>
                            <div class="mt-3">
                                <button class="btn btn-primary me-2" on:click=on_create disabled=move || creating.get()>
                                    {move || if creating.get() { view!{<span class="spinner-border spinner-border-sm me-1"></span>}.into_view() } else { view!{<></>}.into_view() }}
                                    {"创建"}
                                </button>
                                <button class="btn btn-secondary" on:click=move |_| set_create_open.set(false)>{"取消"}</button>
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else { view!{<></>}.into_view() }}

            {move || if edit_open.get() {
                view!{
                    <div class="card mb-3">
                        <div class="card-header">{"编辑配置"}</div>
                        <div class="card-body">
                            <div class="row g-2">
                                <div class="col-md-4">
                                    <label class="form-label">{"数据ID"}</label>
                                    <input class="form-control"
                                           prop:value=edit_data_id
                                           on:input=move |e| set_edit_data_id.set(event_target_value(&e)) />
                                </div>
                                <div class="col-md-4">
                                    <label class="form-label">{"分组"}</label>
                                    <input class="form-control"
                                           prop:value=edit_group
                                           on:input=move |e| set_edit_group.set(event_target_value(&e)) />
                                </div>
                                <div class="col-md-4">
                                    <label class="form-label">{"命名空间"}</label>
                                    <select class="form-select"
                                            prop:value=edit_ns
                                            on:change=move |e| set_edit_ns.set(event_target_value(&e))>
                                        {namespaces.get().into_iter().map(|ns| {
                                            view!{ <option value={ns.namespace.clone()}>{format!("{} - {}", ns.namespace, ns.namespace_show_name)}</option> }
                                        }).collect_view()}
                                    </select>
                                </div>
                            </div>
                            <div class="row g-2 mt-2">
                                <div class="col-md-3">
                                    <label class="form-label">{"配置格式"}</label>
                                    <select class="form-select"
                                            prop:value=edit_type
                                            on:change=move |e| set_edit_type.set(event_target_value(&e))>
                                        <option value="text">{"TEXT"}</option>
                                        <option value="json">{"JSON"}</option>
                                        <option value="xml">{"XML"}</option>
                                        <option value="yaml">{"YAML"}</option>
                                        <option value="properties">{"Properties"}</option>
                                        <option value="html">{"HTML"}</option>
                                    </select>
                                </div>
                            </div>
                            <div class="mt-2">
                                <label class="form-label">{"配置内容"}</label>
                                <textarea id="editContentEditor" class="form-control" rows=10
                                          prop:value=edit_content
                                          on:input=move |e| set_edit_content.set(event_target_value(&e)) />
                            </div>
                            <div class="mt-3">
                                <button class="btn btn-primary me-2" on:click=on_update disabled=move || updating.get()>
            {move || if view_open.get() {
                let lang = hljs_lang(&view_type.get()).to_string();
                view!{
                    <div class="card mb-3">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <span>{"查看配置"}</span>
                            <button class="btn btn-sm btn-secondary" on:click=move |_| set_view_open.set(false)>{"关闭"}</button>
                        </div>
                        <div class="card-body">
                            <pre><code id="viewCode" class={format!("language-{}", lang)}>{view_content.get()}</code></pre>
                        </div>
                    </div>
                }.into_view()
            } else { view!{<></>}.into_view() }}
                                    {move || if updating.get() { view!{<span class="spinner-border spinner-border-sm me-1"></span>}.into_view() } else { view!{<></>}.into_view() }}
                                    {"保存"}
                                </button>
                                <button class="btn btn-secondary" on:click=move |_| set_edit_open.set(false)>{"取消"}</button>
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else { view!{<></>}.into_view() }}

            {move || if history_open.get() {
                view!{
                    <div class="card mb-3">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <span>{"配置历史"}</span>
                            <div class="d-flex align-items-center gap-2">
                                <div class="d-flex align-items-center">
                                    <label class="me-2 mb-0">{"左版本"}</label>
                                    <select class="form-select form-select-sm"
                                            on:change=move |e| {
                                                if let Ok(v) = event_target_value(&e).parse::<i64>() { set_left_ver.set(Some(v)); }
                                            }>
                                        {history_items.get().iter().map(|(v,_,_)| {
                                            view! { <option value={v.to_string()}>{v.to_string()}</option> }
                                        }).collect_view()}
                                    </select>
                                </div>
                                <div class="d-flex align-items-center">
                                    <label class="ms-2 me-2 mb-0">{"右版本"}</label>
                                    <select class="form-select form-select-sm"
                                            on:change=move |e| {
                                                if let Ok(v) = event_target_value(&e).parse::<i64>() { set_right_ver.set(Some(v)); }
                                            }>
                                        {history_items.get().iter().map(|(v,_,_)| {
                                            view! { <option value={v.to_string()}>{v.to_string()}</option> }
                                        }).collect_view()}
                                    </select>
                                </div>
                                <button class="btn btn-sm btn-outline-primary" on:click=on_sbs_compare>
                                    <i class="bi bi-columns"></i> {" 并排对比"}
                                </button>
                                <button class="btn btn-sm btn-secondary" on:click=move |_| set_history_open.set(false)>{"关闭"}</button>
                            </div>
                        </div>
                        <div class="card-body">
                            <div class="table-responsive">
                                <table class="table table-hover">
                                    <thead>
                                        <tr>
                                            <th>{"版本（时间戳）"}</th>
                                            <th>{"状态"}</th>
                                            <th class="text-end">{"操作"}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {history_items.get().into_iter().map(|(v, del, content)| {
                                            view!{
                                                <tr>
                                                    <td>{v}</td>
                                                    <td>{if del { "删除" } else { "正常" }}</td>
                                                    <td class="text-end">
                                                        <button class="btn btn-sm btn-outline-primary me-2"
                                                                on:click=move |_| {
                                                                    set_view_type.set("text".to_string());
                                                                    set_view_content.set(content.clone());
                                                                    set_view_open.set(true);
                                                                    gloo_timers::callback::Timeout::new(50, move || {
                                                                        if let Some(w) = web_sys::window() {
                                                                            if let Ok(f) = js_sys::Reflect::get(&w, &JsValue::from_str("highlightCode")) {
                                                                                if let Ok(func) = f.dyn_into::<Function>() {
                                                                                    let _ = func.call1(&JsValue::NULL, &JsValue::from_str("viewCode"));
                                                                                }
                                                                            }
                                                                        }
                                                                    }).forget();
                                                                }>
                                                            <i class="bi bi-eye"></i> {" 查看"}
                                                        </button>
                                                        <button class="btn btn-sm btn-outline-danger"
                                                                on:click=move |_| {
                                                                    if let Some(first) = configs.get().first().cloned() {
                                                                        do_rollback(first, v);
                                                                    }
                                                                }>
                                                            <i class="bi bi-arrow-counterclockwise"></i> {" 回滚"}
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else { view!{<></>}.into_view() }}

            {move || if sbs_open.get() {
                view!{
                    <div class="card mb-3">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <span>{"并排对比"}</span>
                            <button class="btn btn-sm btn-secondary" on:click=move |_| set_sbs_open.set(false)>{"关闭"}</button>
                        </div>
                        <div class="card-body">
                            <div inner_html={sbs_html.get()}></div>
                        </div>
                    </div>
                }.into_view()
            } else { view!{<></>}.into_view() }}

            <div class="card">
                <div class="card-body">
                    {move || if loading.get() {
                        view! { <Loading /> }.into_view()
                    } else if let Some(msg) = error.get() {
                        view! { <div class="alert alert-danger">{msg}</div> }.into_view()
                    } else {
                        let ns_list = namespaces.get();
                        let cfgs = configs.get();
                        view! {
                            <div class="table-responsive">
                                <table class="table table-hover">
                                    <thead>
                                        <tr>
                                            <th>{"数据ID"}</th>
                                            <th>{"分组"}</th>
                                            <th>{"命名空间"}</th>
                                            <th>{"更新时间"}</th>
                                            <th class="text-end">{"操作"}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {if cfgs.is_empty() {
                                            view! { <tr><td colspan="4" class="text-center text-muted">{"暂无配置"}</td></tr> }.into_view()
                                        } else {
                                            cfgs.into_iter().map(|c| {
                                                let ts = js_sys::Date::new(&JsValue::from_f64(c.update_time as f64))
                                                    .to_locale_string("zh-CN", &JsValue::UNDEFINED)
                                                    .as_string()
                                                    .unwrap_or_else(|| "".to_string());
                                                let c_for_edit = c.clone();
                                                let c_for_view = c.clone();
                                                let c_for_history = c.clone();
                                                let c_for_delete = c.clone();
                                                view! {
                                                    <tr>
                                                        <td><code>{c.data_id}</code></td>
                                                        <td>{c.group}</td>
                                                        <td>{c.namespace.clone()}</td>
                                                        <td>{ts}</td>
                                                        <td class="text-end">
                                                            <button class="btn btn-sm btn-outline-secondary me-2"
                                                                    on:click=move |_| open_view(c_for_view.clone())>
                                                                <i class="bi bi-eye"></i> {" 查看"}
                                                            </button>
                                                            <button class="btn btn-sm btn-outline-secondary me-2"
                                                                    on:click=move |_| open_history(c_for_history.clone())>
                                                                <i class="bi bi-clock-history"></i> {" 历史"}
                                                            </button>
                                                            <button class="btn btn-sm btn-outline-primary me-2"
                                                                    on:click=move |_| open_edit(c_for_edit.clone())>
                                                                <i class="bi bi-pencil"></i> {" 编辑"}
                                                            </button>
                                                            <button class="btn btn-sm btn-outline-danger"
                                                                    on:click=move |_| on_delete(c_for_delete.clone())>
                                                                <i class="bi bi-trash"></i> {" 删除"}
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()
                                        }}
                                    </tbody>
                                </table>
                            </div>
                            // 分页控件
                            <div class="d-flex justify-content-between align-items-center mt-3">
                                <div class="text-muted">
                                    {move || format!("共 {} 条", total_count.get())}
                                </div>
                                <nav aria-label="分页">
                                    <ul class="pagination mb-0">
                                        <li class={move || format!("page-item {}", if page.get() <= 1 { "disabled" } else { "" })}>
                                            <a class="page-link" href="#"
                                               on:click=move |e| { e.prevent_default(); if page.get() > 1 { set_page.set(page.get() - 1); load_configs(current_ns.get()); } }>
                                                {"«"}
                                            </a>
                                        </li>
                                        {move || {
                                            let tp = total_pages.get().max(1);
                                            let cur = page.get().min(tp);
                                            // 渲染最多 7 个页码窗口
                                            let start = if cur > 3 { cur - 3 } else { 1 };
                                            let end = (start + 6).min(tp);
                                            (start..=end).map(|p| {
                                                view!{
                                                    <li class={format!("page-item {}", if p == cur { "active" } else { "" })}>
                                                        <a class="page-link" href="#"
                                                           on:click=move |e| { e.prevent_default(); set_page.set(p); load_configs(current_ns.get()); }>
                                                            {p}
                                                        </a>
                                                    </li>
                                                }
                                            }).collect_view()
                                        }}
                                        <li class={move || format!("page-item {}", if page.get() >= total_pages.get().max(1) { "disabled" } else { "" })}>
                                            <a class="page-link" href="#"
                                               on:click=move |e| { e.prevent_default(); let tp = total_pages.get().max(1); if page.get() < tp { set_page.set(page.get() + 1); load_configs(current_ns.get()); } }>
                                                {"»"}
                                            </a>
                                        </li>
                                    </ul>
                                </nav>
                            </div>
                        }.into_view()
                    }}
                </div>
            </div>
        </div>
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn Configs() {}