#![deny(unused_must_use)]
#![doc = "Leptos-powered WASM GUI for dex parsing and analysis."]

mod ui;

pub use ui::App;

#[cfg(target_arch = "wasm32")]
use {
    leptos::{mount::mount_to_body, prelude::*},
    wasm_bindgen::prelude::*,
};

/// The page entry point. Not compiled into the test harness, which mounts the app itself.
#[cfg(all(target_arch = "wasm32", not(test)))]
#[wasm_bindgen(start)]
pub fn main_js() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    mount();
}

/// Mount [`App`] into the document body.
#[cfg(target_arch = "wasm32")]
pub fn mount() {
    mount_to_body(|| view! { <App/> });
}

#[cfg(all(test, target_arch = "wasm32"))]
mod smoke;
