#[cfg(target_arch = "wasm32")]
use leptos::*;
use crate::frontend::components::Loading;

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Configs() -> impl IntoView {
    view! {
        <div class="container mt-4">
            <h2 class="page-title">
                <i class="bi bi-gear"></i> {" 配置管理"}
            </h2>
            <div class="card">
                <div class="card-body">
                    <Loading />
                </div>
            </div>
        </div>
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn Configs() {}