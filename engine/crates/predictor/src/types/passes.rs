use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PassWindow {
    pub start_ms: f64,
    pub end_ms: f64,
    pub max_elevation_deg: f64,
    pub max_el_time_ms: f64,
}
