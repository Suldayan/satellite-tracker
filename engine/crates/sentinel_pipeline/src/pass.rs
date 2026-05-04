use std::time::Duration;
use log::{info, error};
use sentinel_ndvi::{compute_ndvi, write_f32_tiff, GeoRef};
use sentinel_types::{SatellitePassEvent, BBox};
use crate::error::{PipelineError, PipelineResult};
use crate::stac::fetch_scene_urls;

/// Fetch bands, compute NDVI, and write a Float32 GeoTIFF.
///
/// Returns the output path on success, or `Ok(None)` when no imagery is
/// available for this pass.
pub fn ingest_pass(event: &SatellitePassEvent) -> PipelineResult<Option<String>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(PipelineError::Http)?;

    const FMT: &str = "%Y-%m-%dT%H:%M:%SZ";
    let urls = fetch_scene_urls(
        &client,
        event.min_lon, event.min_lat,
        event.max_lon, event.max_lat,
        &event.pass_start.format(FMT).to_string(),
        &event.pass_end.format(FMT).to_string(),
    )?;

    let Some(urls) = urls else {
        info!("No imagery for pass on {}", event.satellite_id);
        return Ok(None);
    };

    let bbox = BBox {
        min_lon: event.min_lon,
        max_lon: event.max_lon,
        min_lat: event.min_lat,
        max_lat: event.max_lat,
    };

    let b04 = sentinel_cog::fetch_overview_bbox(&client, &urls.b04, 1, &bbox)?;
    let b08 = sentinel_cog::fetch_overview_bbox(&client, &urls.b08, 1, &bbox)?;

    info!("Bands fetched: {}×{}", b04.width, b04.height);

    let (ndvi, w, h) = compute_ndvi(&b04, &b08)?;

    let out_dir = std::path::Path::new("output/ndvi");
    std::fs::create_dir_all(out_dir)
        .map_err(PipelineError::Io)?;

    let filename = format!(
        "ndvi_{}.tif",
        chrono::Utc::now().format("%Y-%m-%dT%H-%M-%SZ")
    );

    let path = out_dir.join(filename);
    let path_str = path.to_string_lossy().to_string();

    write_f32_tiff(&ndvi, w, h, &path_str, &GeoRef::utm10n_10m())?;
    info!("Saved {path_str}");

    Ok(Some(path_str))
}

/// Block until 6 hours after pass end, then run [`ingest_pass`].
pub fn handle_pass(event: SatellitePassEvent) {
    let ready_at = event.pass_end + chrono::Duration::hours(6);
    let wait = (ready_at - chrono::Utc::now())
        .to_std()
        .unwrap_or(Duration::ZERO);

    info!(
        "Pass {} ends {}; waiting {:?} before ingestion",
        event.satellite_id, event.pass_end, wait
    );
    std::thread::sleep(wait);

    match ingest_pass(&event) {
        Ok(Some(path)) => info!("Ingestion complete: {path}"),
        Ok(None) => info!("No imagery available, skipping"),
        Err(e) => error!("Ingestion failed for {}: {e}", event.satellite_id),
    }
}