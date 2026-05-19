mod config;
use config::AzureConfig;

use chrono::Utc;
use sentinel_types::SatellitePassEvent;
use sentinel_pipeline::ingest_pass;
use sentinel_db::insert_ndvi_result;

fn main() {
    env_logger::init();
    dotenvy::dotenv().ok();

    let config = AzureConfig::from_env();

    let event = SatellitePassEvent {
        satellite_id: config.satellite_id,
        pass_start: Utc::now() - chrono::Duration::days(config.lookback_days),
        pass_end: Utc::now(),
        max_elevation_deg: 0.0,
        min_lon: config.min_lon,
        max_lon: config.max_lon,
        min_lat: config.min_lat,
        max_lat: config.max_lat,
    };

    match ingest_pass(&event, config.overview_level) {
        Ok(Some(record)) => {
            log::info!("NDVI complete — mean: {:.3}, pixels: {}", record.mean_ndvi, record.valid_pixels);
            if let Err(e) = insert_ndvi_result(&record) {
                log::error!("DB insert failed: {e}");
            }
        }
        Ok(None) => log::info!("No imagery available for this window"),
        Err(e) => log::error!("Pipeline failed: {e}"),
    }
}