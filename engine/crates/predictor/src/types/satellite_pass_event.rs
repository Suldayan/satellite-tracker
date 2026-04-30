pub struct SatellitePassEvent {
    pub satellite_id: String,
    pub pass_start: chrono::DateTime<chrono::Utc>,
    pub pass_end: chrono::DateTime<chrono::Utc>,
    pub max_elevation_deg: f64,
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}