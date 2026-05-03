use std::sync::mpsc;
use std::thread;
use satellite_predictor::Observer;
use sentinel_orchestrator::{BBox, OrchestratorConfig, predict_loop};
use sentinel_pipeline::handle_pass;
use sentinel_types::SatellitePassEvent;

fn main() {
    // Set RUST_LOG=info to see pipeline output
    env_logger::init();

    let config = OrchestratorConfig {
        norad_id:          40697,  // Sentinel-2A
        satellite_id:      "SENTINEL-2A".into(),
        observer:          Observer::new(49.18, -122.85, 60.0),
        bbox:              BBox::surrey_bc(),
        horizon_hours:     24.0,
        min_elevation_deg: 10.0,
        tle_refresh_hours: 12.0,
    };

    let (tx, rx) = mpsc::channel::<SatellitePassEvent>();

    // Predict passes on a background thread
    thread::spawn(move || predict_loop(tx, config));

    // Each pass gets its own thread so ingestion runs concurrently
    for event in rx {
        thread::spawn(move || handle_pass(event));
    }
}