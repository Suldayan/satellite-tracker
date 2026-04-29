use chrono::{DateTime, Utc, TimeZone};
use predictor::{SatellitePassEvent, Observer, ScanOptions};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn get_tle() -> Result<(String, String), reqwest::Error> {
    let body = reqwest::blocking::get(
        "https://celestrak.org/NORAD/elements/gp.php?CATNR=25544&FORMAT=TLE"
    )?.text()?;

    let lines: Vec<&str> = body.trim().lines().map(|l| l.trim()).collect();
    Ok((lines[1].to_string(), lines[2].to_string()))
}

pub fn ms_to_datetime(ms: f64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(ms as i64).unwrap()
}

pub fn predict_loop(tx: mpsc::Sender<SatellitePassEvent>, observer: Observer) {
    loop {
        let (tle1, tle2) = match get_tle() {
            Ok(t) => t,
            Err(e) => { eprintln!("TLE fetch failed: {e}"); return; }
        };

        let options = ScanOptions::new(
            chrono::Utc::now().timestamp_millis() as f64,
            24.0,
            10.0,
        );

        match predictor::passes(&tle1, &tle2, &observer, &options) {
            Ok(passes) => {
                for pass in passes {
                    let event = SatellitePassEvent {
                        satellite_id: "SENTINEL-2A".into(),
                        pass_start: ms_to_datetime(pass.start_ms),
                        pass_end: ms_to_datetime(pass.end_ms),
                        max_elevation_deg: pass.max_elevation_deg,
                        min_lon: -122.95, max_lon: -122.65,
                        min_lat: 49.05, max_lat: 49.35,
                    };

                    if tx.send(event).is_err() {
                        return;
                    }
                }
            }
            Err(e) => eprintln!("Prediction error: {e}"),
        }

        // Refresh TLE and re-scan every 12 hours
        thread::sleep(Duration::from_secs(60 * 60 * 12));
    }
}