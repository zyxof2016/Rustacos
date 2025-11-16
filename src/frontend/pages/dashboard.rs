#[cfg(target_arch = "wasm32")]
use leptos::*;
use gloo_timers::callback::Interval;

#[cfg(target_arch = "wasm32")]
use crate::frontend::{
    components::{StatCard, Loading},
};

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Dashboard() -> impl IntoView {
    let (loading, set_loading) = create_signal(true);
    
    // 模拟加载
    create_effect(move |_| {
        set_loading.set(true);
        gloo_timers::callback::Timeout::new(1000, move || {
            set_loading.set(false);
        }).forget();
    });
    
    view! {
        <div class="container mt-4">
            <h2 class="page-title">
                <i class="bi bi-speedometer2"></i> {" 系统仪表盘"}
            </h2>
            
            <div class="row mb-4">
                <StatCard 
                    title="服务总数".to_string()
                    value="5".to_string()
                    icon="bi-server".to_string()
                    color="primary".to_string()
                />
                <StatCard 
                    title="实例总数".to_string()
                    value="12".to_string()
                    icon="bi-hdd-stack".to_string()
                    color="success".to_string()
                />
                <StatCard 
                    title="配置总数".to_string()
                    value="8".to_string()
                    icon="bi-gear".to_string()
                    color="info".to_string()
                />
                <StatCard 
                    title="命名空间总数".to_string()
                    value="1".to_string()
                    icon="bi-collection".to_string()
                    color="warning".to_string()
                />
            </div>
            
            <div class="row">
                <div class="col-md-6">
                    <div class="card">
                        <div class="card-header">
                            <h5 class="card-title mb-0">{"最近服务"}</h5>
                        </div>
                        <div class="card-body">
                            {move || {
                                if loading.get() {
                                    view! { <Loading /> }.into_view()
                                } else {
                                    view! {
                                        <div class="alert alert-info">
                                            {"请使用导航栏查看详细信息"}
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