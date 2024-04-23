pub mod app;
pub mod error_template;
#[cfg(feature = "ssr")]
pub mod fileserv;
pub mod frontend;
#[cfg(feature = "ssr")]
pub mod handlers;
pub mod models;
pub mod server;
#[cfg(feature = "ssr")]
pub mod state;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}

// pub const MODEL: &str = "llama3:8b";
// pub const MODEL: &str = "tinyllama:latest";
pub const MODEL: &str = "mistral:7b";
