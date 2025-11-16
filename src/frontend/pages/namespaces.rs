#[cfg(target_arch = "wasm32")]
use leptos::*;
use web_sys::console;

#[cfg(target_arch = "wasm32")]
use crate::frontend::components::Loading;

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Namespaces() -> impl IntoView {
    let (loading, set_loading) = create_signal(true);
    let (namespaces, set_namespaces) = create_signal(vec!["public".to_string()]);
    
    // 模拟加载
    create_effect(move |_| {
        set_loading.set(true);
        
        // 模拟异步加载
        gloo_timers::callback::Timeout::new(1000, move || {
            set_loading.set(false);
            console::log_1(&"命名空间加载完成".into());
        }).forget();
    });
    
    view! {
        <div class="container mt-4">
            <h2 class="page-title">
                <i class="bi bi-folder"></i> {" 命名空间管理"}
            </h2>
            
            <div class="card">
                <div class="card-body">
                    {move || {
                        if loading.get() {
                            view! { <Loading /> }.into_view()
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
                                                <th>{"操作"}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {ns_list.iter().map(|ns| {
                                                let ns_clone = ns.clone();
                                                let ns_clone_edit = ns.clone();
                                                let ns_clone_delete = ns.clone();
                                                view! {
                                                    <tr>
                                                        <td>{ns_clone}</td>
                                                        <td>{"Public Namespace"}</td>
                                                        <td>{"默认命名空间"}</td>
                                                        <td>
                                                            <button 
                                                                class="btn btn-sm btn-outline-primary me-1"
                                                                on:click=move |_| {
                                                                    console::log_1(&format!("编辑命名空间: {}", ns_clone_edit).into());
                                                                }>
                                                                <i class="bi bi-pencil"></i>
                                                            </button>
                                                            <button 
                                                                class="btn btn-sm btn-outline-danger"
                                                                on:click=move |_| {
                                                                    console::log_1(&format!("删除命名空间: {}", ns_clone_delete).into());
                                                                    let result = web_sys::window()
                                                                        .unwrap()
                                                                        .confirm_with_message(&format!("确定要删除命名空间 \"{}\" 吗？", ns_clone_delete));
                                                                    if result.unwrap_or(false) {
                                                                        console::log_1(&format!("确认删除: {}", ns_clone_delete).into());
                                                                    }
                                                                }>
                                                                <i class="bi bi-trash"></i>
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()
                                        }
                                    </tbody>
                                    </table>
                                </div>
                            }.into_view()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn Namespaces() {}