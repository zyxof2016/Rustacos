#[cfg(target_arch = "wasm32")]
use leptos::*;
#[cfg(target_arch = "wasm32")]
use rustacos::frontend::App;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init().expect("Failed to initialize console_log");
    
    // 设置文档标题
    document().set_title("Rustacos - Nacos的Rust实现");
    
    leptos::mount_to_body(App)
}

// wasm 目标下提供空 main 以满足 bin 要求（入口由 #[wasm_bindgen(start)] 触发）
#[cfg(target_arch = "wasm32")]
pub fn main() {}

// 非 wasm 构建时提供空 main，避免构建失败
#[cfg(not(target_arch = "wasm32"))]
pub fn main() {}