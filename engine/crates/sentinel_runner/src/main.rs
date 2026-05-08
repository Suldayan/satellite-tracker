use std::sync::mpsc;
use std::thread;

use satellite_predictor::Observer;
use sentinel_orchestrator::{OrchestratorConfig, predict_loop};
use sentinel_pipeline::{handle_pass, PipelineResult};
use sentinel_types::{SatellitePassEvent, BBox};
use sentinel_events::Event;

const OVERVIEW_LEVEL: u8 = 1;

fn main() {
    env_logger::init();

    let config = OrchestratorConfig {
        norad_id: 40697,
        satellite_id: "SENTINEL-2A".into(),
        observer: Observer::new(49.18, -122.85, 60.0),
        bbox: BBox::surrey_bc(),
        horizon_hours: 24.0,
        min_elevation_deg: 10.0,
        tle_refresh_hours: 12.0,
    };

    let (tx, rx) = mpsc::channel::<Event>();

    let tx_orch = tx.clone();
    thread::spawn(move || predict_loop(tx_orch, config));

    sentinel_pipeline::set_sender(tx.clone());

    for event in rx {
        match event {
            Event::SatellitePass(pass) => {
                thread::spawn(move || handle_pass(pass, OVERVIEW_LEVEL));
            }

            Event::PipelineFinished(result) => {
                println!("Pipeline finished: {:?}", result);
            }
        }
    }
}
