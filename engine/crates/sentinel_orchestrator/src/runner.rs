use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use log::{info, warn, error};
use crate::config::OrchestratorConfig;
use sentinel_types::SatellitePassEvent;
use crate::tle::fetch_tle;
use crate::predict::predict_passes;

/// Run the prediction loop, sending [`SatellitePassEvent`]s over `tx`.
///
/// Refreshes the TLE and re-scans every `config.tle_refresh_hours`. Retries
/// on TLE fetch failure with a 60-second backoff rather than giving up.
/// Returns when the receiver is dropped (channel closed).
pub fn predict_loop(tx: mpsc::Sender<SatellitePassEvent>, config: OrchestratorConfig) {
    loop {
        let tle = loop {
            match fetch_tle(config.norad_id) {
                Ok(t) => break t,
                Err(e) => {
                    warn!("TLE fetch failed for NORAD {}: {e} — retrying in 60s", config.norad_id);
                    thread::sleep(Duration::from_secs(60));
                }
            }
        };

        info!("TLE refreshed for {} (NORAD {})", config.satellite_id, config.norad_id);

        match predict_passes(&tle, &config) {
            Ok(events) => {
                info!("{} pass(es) predicted", events.len());
                for event in events {
                    if tx.send(event).is_err() {
                        info!("Receiver dropped — shutting down predict_loop");
                        return;
                    }
                }
            }
            Err(e) => error!("Pass prediction failed: {e}"),
        }

        thread::sleep(Duration::from_secs_f64(
            config.tle_refresh_hours * 3_600.0,
        ));
    }
}