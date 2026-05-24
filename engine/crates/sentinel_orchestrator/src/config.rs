use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")] 
pub struct Config {
    pub database_url: String, 

    #[serde(default = "default_satellite")]
    pub satellite_id: String,

    #[serde(default = "default_min_lon")]
    pub min_lon: f64,

    #[serde(default = "default_max_lon")]
    pub max_lon: f64,

    #[serde(default = "default_min_lat")]
    pub min_lat: f64,

    #[serde(default = "default_max_lat")]
    pub max_lat: f64,

    #[serde(default = "default_overview_level")]
    pub overview_level: u8,

    #[serde(default = "default_lookback_days")]
    pub lookback_days: i64,
}

fn default_satellite() -> String { "SENTINEL-2A".into() }
fn default_min_lon() -> f64 { -122.95 }
fn default_max_lon() -> f64 { -122.65 }
fn default_min_lat() -> f64 { 49.05 }
fn default_max_lat() -> f64 { 49.35 }
fn default_overview_level() -> u8 { 1 }
fn default_lookback_days() -> i64 { 5 }

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Self>()
    }

    pub fn for_test(overview_level: u8, database_url: String) -> Self {
        Self {
            database_url,
            satellite_id: "SENTINEL-2".into(),
            min_lon: -123.95,
            max_lon: -122.65,
            min_lat: 49.05,
            max_lat: 49.35,
            overview_level,
            lookback_days: 400,
        }
    }
}