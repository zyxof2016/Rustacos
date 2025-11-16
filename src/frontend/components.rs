#[cfg(target_arch = "wasm32")]
use leptos::*;
use leptos_router::*;

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="navbar navbar-expand-lg navbar-dark bg-dark">
            <div class="container">
                <A href="/" class="navbar-brand">
                    <i class="bi bi-hdd-network"></i> {" Rustacos"}
                </A>
                <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav">
                    <span class="navbar-toggler-icon"></span>
                </button>
                <div class="collapse navbar-collapse" id="navbarNav">
                    <ul class="navbar-nav ms-auto">
                        <li class="nav-item">
                            <A href="/" class="nav-link">
                                <i class="bi bi-speedometer2"></i> {" 仪表盘"}
                            </A>
                        </li>
                        <li class="nav-item">
                            <A href="/services" class="nav-link">
                                <i class="bi bi-server"></i> {" 服务管理"}
                            </A>
                        </li>
                        <li class="nav-item">
                            <A href="/configs" class="nav-link">
                                <i class="bi bi-gear"></i> {" 配置管理"}
                            </A>
                        </li>
                        <li class="nav-item">
                            <A href="/namespaces" class="nav-link">
                                <i class="bi bi-folder"></i> {" 命名空间"}
                            </A>
                        </li>
                    </ul>
                </div>
            </div>
        </nav>
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <div class="d-flex justify-content-center">
            <div class="spinner-border" role="status">
                <span class="visually-hidden">{"Loading..."}</span>
            </div>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Modal(
    show: RwSignal<bool>,
    title: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="modal" 
             class:modal-show=move || show.get()
             style:display=move || if show.get() { "block" } else { "none" }>
            <div class="modal-dialog">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title">{title}</h5>
                        <button type="button" 
                                class="btn-close"
                                on:click=move |_| show.set(false)></button>
                    </div>
                    <div class="modal-body">
                        {children()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Button(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
    #[prop(into, optional)] onclick: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    let class_val = class.unwrap_or_default();
    view! {
        <button 
            class=format!("btn {}", class_val)
            on:click=move |e| {
                if let Some(cb) = onclick {
                    cb.call(e);
                }
            }>
            {children()}
        </button>
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn TextInput(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(default = false)] required: bool,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    view! {
        <div class="mb-3">
            <label class="form-label">{label}</label>
            <input 
                type="text" 
                class="form-control"
                name=name
                prop:value=value
                on:input=move |e| {
                    value.set(event_target_value(&e));
                }
                placeholder=placeholder.unwrap_or_default()
                required=required
                disabled=disabled
            />
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn TextArea(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(default = 3)] rows: i32,
    #[prop(default = false)] required: bool,
) -> impl IntoView {
    view! {
        <div class="mb-3">
            <label class="form-label">{label}</label>
            <textarea 
                class="form-control"
                name=name
                prop:value=value
                on:input=move |e| {
                    value.set(event_target_value(&e));
                }
                placeholder=placeholder.unwrap_or_default()
                rows=rows
                required=required
            />
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Select(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<String>,
    options: Vec<(String, String)>, // (value, label)
    #[prop(into, optional)] onchange: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <div class="mb-3">
            <label class="form-label">{label}</label>
            <select 
                class="form-select"
                name=name
                prop:value=value
                on:change=move |e| {
                    if let Some(cb) = &onchange {
                        cb.call(event_target_value(&e));
                    }
                }>
                {options.iter().map(|(val, label)| {
                    view! {
                        <option value=val>{label}</option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn StatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] icon: String,
    #[prop(into)] color: String,
) -> impl IntoView {
    view! {
        <div class={format!("card stats-card border-left-4 border-{}", color)}>
            <div class="card-body">
                <div class="d-flex justify-content-between">
                    <div>
                        <h6 class="card-subtitle mb-2 text-muted">{title}</h6>
                        <h3 class="card-title mb-0">{value}</h3>
                    </div>
                    <div class="align-self-center">
                        <i class={format!("bi {} fs-2", icon)}></i>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn Navbar() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn Loading() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn Modal(
    show: RwSignal<bool>,
    title: String,
    children: Children,
) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn Button(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
    #[prop(into, optional)] onclick: Option<Callback<ev::MouseEvent>>,
) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn TextInput(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] name: Option<String>,
    value: RwSignal<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(default = false)] required: bool,
    #[prop(default = false)] disabled: bool,
) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn TextArea(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] name: Option<String>,
    value: RwSignal<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(default = 3)] rows: i32,
    #[prop(default = false)] required: bool,
) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn Select(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] name: Option<String>,
    value: RwSignal<String>,
    options: Vec<(String, String)>,
    #[prop(into, optional)] onchange: Option<Callback<String>>,
) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn StatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] icon: String,
    #[prop(into)] color: String,
) {}