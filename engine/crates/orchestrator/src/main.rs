use orchestrator::predict_loop;
use predictor::{Observer, SatellitePassEvent};

fn main() {
    let (tx, rx) = std::sync::mpsc::channel::<SatellitePassEvent>();
    let observer = Observer::new(49.19, -122.84, 60.0);

    std::thread::spawn(move || {
        predict_loop(tx, observer);
    });

    while let Ok(event) = rx.recv() {
        println!("Pass received: {} at {}", event.satellite_id, event.pass_start);
        std::thread::spawn(move || {
            ingestion::bands::handle_pass(event);
        });
    }
}

