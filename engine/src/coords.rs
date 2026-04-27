use std::f64::consts::PI;
use crate::types::{TemePosition, EcefPosition, GeodeticPosition, AzEl};

pub fn get_gmst(unix_ms: f64) -> f64 {
    let seconds_since_2000 = (unix_ms / 1000.0) - 946728000.0;
    let d = seconds_since_2000 / 86400.0;
    let gmst_hours = 18.697374558 + 24.06570982441908 * d;
    (gmst_hours % 24.0) * (PI / 12.0)
}

impl EcefPosition {
    pub fn from_teme(teme: &TemePosition, gmst_radians: f64) -> Self {
        let (sin_t, cos_t) = gmst_radians.sin_cos();
        Self {
            x: teme.x * cos_t + teme.y * sin_t,
            y: -teme.x * sin_t + teme.y * cos_t,
            z: teme.z, 
        }
    }
}

impl AzEl {
    pub fn from_ecef(sat_ecef: &EcefPosition, obs_geo: &GeodeticPosition) -> Self {
        let obs_ecef = obs_geo.to_ecef();
        
        let lat_rad = obs_geo.lat_deg.to_radians();
        let lon_rad = obs_geo.lon_deg.to_radians();

        // Slant Range Vector (Satellite - Observer)
        let dx = sat_ecef.x - obs_ecef.x;
        let dy = sat_ecef.y - obs_ecef.y;
        let dz = sat_ecef.z - obs_ecef.z;

        // Rotate into Topocentric ENU (East, North, Up)
        let east = -lon_rad.sin() * dx + lon_rad.cos() * dy;
        let north = -lat_rad.sin() * lon_rad.cos() * dx - lat_rad.sin() * lon_rad.sin() * dy + lat_rad.cos() * dz;
        let up = lat_rad.cos() * lon_rad.cos() * dx + lat_rad.cos() * lon_rad.sin() * dy + lat_rad.sin() * dz;

        let range_km = (east.powi(2) + north.powi(2) + up.powi(2)).sqrt();
        let elevation = (up / range_km).asin().to_degrees();
        
        let azimuth = east.atan2(north).to_degrees();
        let compass_azimuth = (azimuth + 360.0) % 360.0; 

        Self {
            azimuth: compass_azimuth,
            elevation,
            range_km,
        }
    }
}