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