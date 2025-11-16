use leptos::*;
use leptos_router::*;
use leptos_meta::*;
use crate::frontend::app::App;

#[wasm_bindgen::prelude::wasm_bindgen_start]

#[wasm_bindgen_start]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init().expect("Failed to initialize console_log");
    
    // 设置文档标题
    document().set_title(&"Rustacos - Nacos的Rust实现".into());
    
    leptos::mount_to_body(App)
}