use predictor::{Observer, ScanOptions, passes};

const TLE1: &str = "1 25544U 98067A   26119.49027628  .00007115  00000+0  13705-3 0  9999";
const TLE2: &str = "2 25544  51.6319 181.1364 0007113   4.2135 355.8912 15.49020533564200";

#[test]
fn iss_produces_passes_over_surrey() {
    let (tle1, tle2) = orchestrator::get_tle()
        .expect("Failed to fetch ISS TLE from CelesTrak");

    let observer = Observer::new(49.19, -122.84, 60.0);
    let options  = ScanOptions::new(
        chrono::Utc::now().timestamp_millis() as f64,
        24.0,
        10.0,
    );

    let result = passes(&tle1, &tle2, &observer, &options);
    assert!(result.is_ok(), "passes() returned error: {:?}", result.err());

    let windows = result.unwrap();
    assert!(!windows.is_empty(), "Expected at least one ISS pass over Surrey in 24h");

    for w in &windows {
        assert!(w.max_elevation_deg > 10.0);
        assert!(w.end_ms > w.start_ms);
    }
}

#[test]
fn elevation_below_threshold_returns_no_passes() {
    let observer = Observer::new(49.19, -122.84, 60.0);
    // 89° threshold — nothing will ever pass this high
    let options = ScanOptions::new(1_745_280_000_000.0, 24.0, 89.0);

    let windows = passes(TLE1, TLE2, &observer, &options).unwrap();
    assert!(windows.is_empty(), "Expected no passes at 89° threshold");
}