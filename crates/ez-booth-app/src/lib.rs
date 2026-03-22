use leptos::*;
use wasm_bindgen::prelude::*;

/// Entry point for the WASM application
#[wasm_bindgen(start)]
pub fn main() {
    // Set up console error panic hook for better debugging
    console_error_panic_hook::set_once();
    
    // Mount the application
    leptos::mount_to_body(|| view! { <ez_booth_ui::App /> });
}

