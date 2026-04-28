use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use predictor::{SatellitePassEvent, Observer, ScanOptions};

fn main() {
    let (tx, rx) = mpsc::channel::<SatellitePassEvent>();

    let observer = Observer::new(49.19, -122.84, 60.0); 

    thread::spawn(move || {
        predict_loop(tx, observer);
    });

    while let Ok(event) = rx.recv() {
        println!("Pass received: {} at {}", event.satellite_id, event.pass_start);
        
        thread::spawn(move || {
            ingestion::handle_pass(event);
        });
    }
}

fn predict_loop(
    tx: mpsc::Sender<SatellitePassEvent>,
    observer: Observer,
) {
    let tle1 = "..."; // eventually fetched from CelesTrak
    let tle2 = "...";

    loop {
        let options = ScanOptions::new(
            chrono::Utc::now().timestamp_millis() as f64,
            24.0,  
            10.0,  
        );

        match predictor::passes(tle1, tle2, &observer, &options) {
            Ok(passes) => {
                for pass in passes {
                    let event = SatellitePassEvent {
                        satellite_id:      "SENTINEL-2A".into(),
                        pass_start:        pass.start_ms_to_datetime(),
                        pass_end:          pass.end_ms_to_datetime(),
                        max_elevation_deg: pass.max_elevation_deg,
                        min_lon: -122.95, max_lon: -122.65,
                        min_lat:  49.05,  max_lat:  49.35,
                    };

                    if tx.send(event).is_err() {
                        return; 
                    }
                }
            }
            Err(e) => eprintln!("Prediction error: {e}"),
        }
        
        thread::sleep(Duration::from_secs(60 * 60 * 12));
    }
}