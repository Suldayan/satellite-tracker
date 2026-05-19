pub struct AzureConfig {
    pub satellite_id: String,
    pub min_lon: f64,
    pub max_lon: f64, 
    pub min_lat: f64,
    pub max_lat: f64,
    pub overview_level: u8,
    pub lookback_days: i64,
}

impl AzureConfig {
    pub fn from_env() -> Self {
        Self {
            satellite_id: std::env::var("SATELLITE_ID").unwrap_or_else(|_| "SENTINEL-2A".into()),
            min_lon: std::env::var("MIN_LON").unwrap_or_else(|_| "-122.95".into()).parse().unwrap(),
            max_lon: std::env::var("MAX_LON").unwrap_or_else(|_| "-122.65".into()).parse().unwrap(),
            min_lat: std::env::var("MIN_LAT").unwrap_or_else(|_| "49.05".into()).parse().unwrap(),
            max_lat: std::env::var("MAX_LAT").unwrap_or_else(|_| "49.35".into()).parse().unwrap(),
            overview_level: std::env::var("OVERVIEW_LEVEL").unwrap_or_else(|_| "1".into()).parse().unwrap(),
            lookback_days: std::env::var("LOOKBACK_DAYS").unwrap_or_else(|_| "5".into()).parse().unwrap(),
        }
    }
}