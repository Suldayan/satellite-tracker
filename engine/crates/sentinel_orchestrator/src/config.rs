use satellite_predictor::Observer;

/// Geographic bounding box for the region of interest.
#[derive(Debug, Clone, Copy)]
pub struct BBox {
    pub min_lon: f64,
    pub max_lon: f64,
    pub min_lat: f64,
    pub max_lat: f64,
}

impl BBox {
    /// Surrey, BC — the default region of interest.
    pub fn surrey_bc() -> Self {
        Self {
            min_lon: -122.95,
            max_lon: -122.65,
            min_lat:   49.05,
            max_lat:   49.35,
        }
    }
}

/// Full configuration for the orchestrator loop.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// NORAD catalog number — Sentinel-2A = 40697, Sentinel-2B = 42063.
    pub norad_id: u32,
    /// Human-readable label attached to every [`SatellitePassEvent`].
    pub satellite_id: String,
    /// Ground observer position used for pass prediction.
    pub observer: Observer,
    /// Region of interest attached to every emitted event.
    pub bbox: BBox,
    /// How many hours ahead to scan for passes.
    pub horizon_hours: f64,
    /// Minimum elevation in degrees for a pass to be considered visible.
    pub min_elevation_deg: f64,
    /// How often to refresh the TLE and re-scan, in hours.
    pub tle_refresh_hours: f64,
}