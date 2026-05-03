use crate::error::TrackerError;

#[derive(Debug, Clone, Copy)]
pub struct TemePosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct EcefPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct GeodeticPosition {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub alt_m: f64,
}

impl GeodeticPosition {
    pub fn validate(&self) -> Result<(), TrackerError> {
        if !(-90.0..=90.0).contains(&self.lat_deg) {
            return Err(TrackerError::InvalidInput(format!(
                "latitude {:.4} is outside [-90, 90]", self.lat_deg
            )));
        }
        if !(-180.0..=180.0).contains(&self.lon_deg) {
            return Err(TrackerError::InvalidInput(format!(
                "longitude {:.4} is outside [-180, 180]", self.lon_deg
            )));
        }
        if self.alt_m < -500.0 {
            return Err(TrackerError::InvalidInput(format!(
                "altitude {:.1} m is unreasonably low (< -500 m)", self.alt_m
            )));
        }
        Ok(())
    }

    pub fn to_ecef(&self) -> EcefPosition {
        const A: f64 = 6_378.137;
        const F: f64 = 1.0 / 298.257_223_563;
        const E2: f64 = 2.0 * F - F * F;

        let lat = self.lat_deg.to_radians();
        let lon = self.lon_deg.to_radians();
        let alt_km = self.alt_m / 1_000.0;

        let n = A / (1.0 - E2 * lat.sin().powi(2)).sqrt();

        EcefPosition {
            x: (n + alt_km) * lat.cos() * lon.cos(),
            y: (n + alt_km) * lat.cos() * lon.sin(),
            z: (n * (1.0 - E2) + alt_km) * lat.sin(),
        }
    }
}
