use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SatellitePosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}