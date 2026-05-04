use satellite_predictor::Observer;
use sentinel_types::BBox;

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub norad_id: u32,
    pub satellite_id: String,
    pub observer: Observer,
    pub bbox: BBox,
    pub horizon_hours: f64,
    pub min_elevation_deg: f64,
    pub tle_refresh_hours: f64,
}