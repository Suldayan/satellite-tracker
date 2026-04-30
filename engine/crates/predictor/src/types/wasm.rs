#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub struct AzEl {
    pub azimuth: f64,
    pub elevation: f64,
    pub range_km: f64,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub struct Observer {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub alt_m: f64,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Observer {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new(lat_deg: f64, lon_deg: f64, alt_m: f64) -> Self {
        Self { lat_deg, lon_deg, alt_m }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub struct ScanOptions {
    pub start_ms: f64,
    pub duration_hours: f64,
    pub min_elevation_deg: f64,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl ScanOptions {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new(start_ms: f64, duration_hours: f64, min_elevation_deg: f64) -> Self {
        Self { start_ms, duration_hours, min_elevation_deg }
    }
}
