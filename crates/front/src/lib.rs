//! Entry point of the browser bundle: wakes up the islands on the page.

#[allow(unused_imports)]
use site::*;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_islands();
}
