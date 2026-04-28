use predictor::SatellitePassEvent;
use std::time::Duration;

pub fn handle_pass(event: SatellitePassEvent) {
    // Wait for imagery to be processed and uploaded
    // (real-world delay between capture and availability)
    let delay = event.pass_end + chrono::Duration::hours(6);
    let wait  = (delay - chrono::Utc::now()).to_std()
                    .unwrap_or(Duration::ZERO);

    std::thread::sleep(wait);

    // Then query Planetary Computer STAC API
    fetch_imagery(&event);
}

fn fetch_imagery(event: &SatellitePassEvent) {
    // STAC API call...
}