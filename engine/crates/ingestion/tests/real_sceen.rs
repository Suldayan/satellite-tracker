use ingestion::stac::fetch_imagery;
use predictor::SatellitePassEvent;
use chrono::DateTime;

const T2: &str = "2026-04-24T19:09:09.024000Z";
const T1: &str = "2026-04-09T19:09:11.000000Z";

#[test]
#[ignore]
fn fetch_real_surrey_scene() {
    let event = SatellitePassEvent {
        satellite_id: "SENTINEL-2A".into(),
        pass_start: T1.parse::<DateTime<chrono::Utc>>().unwrap(),
        pass_end: T2.parse::<DateTime<chrono::Utc>>().unwrap(),
        max_elevation_deg: 45.0,
        min_lon: -122.95, max_lon: -122.65,
        min_lat: 49.05, max_lat: 49.35,
    };

    let result = fetch_imagery(&event);
    assert!(result.is_ok(), "STAC query failed: {:?}", result.err());

    let urls = result.unwrap();
    assert!(urls.is_some(), "No scenes found — try a different date");

    let (b04_url, b08_url) = urls.unwrap();
    println!("B04: {}", b04_url);
    println!("B08: {}", b08_url);
}

#[test]
#[ignore]
fn produces_ndvi_geotiff_for_surrey() {
    let event = SatellitePassEvent {
        satellite_id: "SENTINEL-2A".into(),
        pass_start: T1.parse::<DateTime<chrono::Utc>>().unwrap(),
        pass_end: T2.parse::<DateTime<chrono::Utc>>().unwrap(),
        max_elevation_deg: 45.0,
        min_lon: -122.95, max_lon: -122.65,
        min_lat: 49.05, max_lat: 49.35,
    };

    // Bypass the 6-hour sleep entirely and call download directly
    let result = ingestion::bands::download_bands(&event);
    assert!(result.is_ok(), "Download failed: {:?}", result.err());

    // Check the file actually landed on disk
    let files: Vec<_> = std::fs::read_dir(".")
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("ndvi_"))
        .collect();

    assert!(!files.is_empty(), "No NDVI GeoTIFF written to disk");
    println!("Output: {:?}", files[0].file_name());
}