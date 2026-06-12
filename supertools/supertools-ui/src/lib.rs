use leptos::*;

pub mod app;
pub mod components;

#[path = "../../supertools-cli/src/diff_parser.rs"]
pub mod diff_parser;

use app::App;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
