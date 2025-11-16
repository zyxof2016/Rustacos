#[cfg(target_arch = "wasm32")]
use leptos::*;
use crate::frontend::components::Loading;

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Services() -> impl IntoView {
    view! {
        <div class="container mt-4">
            <h2 class="page-title">
                <i class="bi bi-server"></i> {" 服务管理"}
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
pub fn Services() {}