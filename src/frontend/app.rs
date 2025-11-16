#[cfg(target_arch = "wasm32")]
use leptos::*;
use leptos_router::*;
use leptos_meta::*;

#[cfg(target_arch = "wasm32")]
use crate::frontend::{
    components::Navbar,
    pages::{Dashboard, Services, Configs, Namespaces},
};

#[cfg(target_arch = "wasm32")]
#[component]
pub fn App() -> impl IntoView {
    // 提供元数据
    provide_meta_context();

    view! {
        <Router>
            <div class="min-vh-100 bg-light">
                <Navbar />
                <main class="pb-5">
                    <Routes>
                        <Route path="/" view=Dashboard />
                        <Route path="/services" view=Services />
                        <Route path="/configs" view=Configs />
                        <Route path="/namespaces" view=Namespaces />
                        <Route path="/*any" view=NotFound />
                    </Routes>
                </main>
                <footer class="bg-dark text-white text-center py-3 mt-5">
                    <div class="container">
                        <p class="mb-0">{"© 2024 Rustacos - Nacos的Rust实现"}</p>
                        <small class="text-muted">{"使用Leptos构建的现代Web界面"}</small>
                    </div>
                </footer>
            </div>
        </Router>
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="container mt-5">
            <div class="row">
                <div class="col-12">
                    <div class="alert alert-warning">
                        <h4>{"404 - 页面未找到"}</h4>
                        <p>{"抱歉，您访问的页面不存在。"}</p>
                        <A href="/" class="btn btn-primary">
                            {"返回首页"}
                        </A>
                    </div>
                </div>
            </div>
        </div>
    }
}