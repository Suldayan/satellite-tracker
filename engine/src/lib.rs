use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_satellite_status(name: &str) -> String {
    format!("Satellite {} is being tracked in Rust!", name)
}