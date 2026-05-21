use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{Container, runners::SyncRunner, ImageExt},
};
use sentinel_orchestrator::AzureConfig;

fn start_postgis() -> (Container<Postgres>, String) {
    let container = Postgres::default()
        .with_name("postgis/postgis")
        .with_tag("15-3.3")
        .start()
        .expect("PostGIS container failed to start — is Docker running?");

    let port = container.get_host_port_ipv4(5432).unwrap();
    let conn_str = format!(
        "host=localhost port={port} user=postgres password=postgres dbname=postgres"
    );
    (container, conn_str)
}

fn setup_schema(client: &mut postgres::Client) {
    client.batch_execute("
        CREATE EXTENSION IF NOT EXISTS postgis;
        CREATE TABLE ndvi_history (
            id SERIAL PRIMARY KEY,
            captured_at TIMESTAMPTZ NOT NULL,
            satellite_id TEXT NOT NULL,
            min_lon DOUBLE PRECISION NOT NULL,
            max_lon DOUBLE PRECISION NOT NULL,
            min_lat DOUBLE PRECISION NOT NULL,
            max_lat DOUBLE PRECISION NOT NULL,
            mean_ndvi REAL NOT NULL,
            max_ndvi REAL NOT NULL,
            min_ndvi REAL NOT NULL,
            valid_pixels INTEGER NOT NULL,
            tif_path TEXT NOT NULL,
            bbox GEOMETRY(POLYGON, 4326)
        );
    ").expect("Schema migration failed");
}

fn run_pipeline_at_level(overview_level: u8) -> (f32, i32) {
    let (_container, conn_str) = start_postgis();
    let mut client = postgres::Client::connect(&conn_str, postgres::NoTls).unwrap();
    setup_schema(&mut client);

    sentinel_orchestrator::run_with(AzureConfig::for_test(overview_level, conn_str.clone()))
        .unwrap_or_else(|e| panic!("Pipeline failed at level {overview_level}: {e}"));

    let row = client
        .query_one("SELECT mean_ndvi, valid_pixels FROM ndvi_history", &[])
        .unwrap();

    (row.get(0), row.get(1))
}

#[test]
#[ignore]
fn pipeline_overview_level_1() {
    let (mean, pixels) = run_pipeline_at_level(1);
    assert!(mean > -1.0 && mean < 1.0, "NDVI out of range: {mean}");
    assert!(pixels > 0, "Expected valid pixels at level 1");
    println!("Level 1 — mean NDVI: {mean:.3}, valid pixels: {pixels}");
}

#[test]
#[ignore]
fn pipeline_overview_level_2() {
    let (mean, pixels) = run_pipeline_at_level(2);
    assert!(mean > -1.0 && mean < 1.0, "NDVI out of range: {mean}");
    assert!(pixels > 0, "Expected valid pixels at level 2");
    println!("Level 2 — mean NDVI: {mean:.3}, valid pixels: {pixels}");
}

#[test]
#[ignore]
fn pipeline_overview_level_3() {
    let (mean, pixels) = run_pipeline_at_level(3);
    assert!(mean > -1.0 && mean < 1.0, "NDVI out of range: {mean}");
    assert!(pixels > 0, "Expected valid pixels at level 3");
    println!("Level 3 — mean NDVI: {mean:.3}, valid pixels: {pixels}");
}

#[test]
#[ignore]
fn overview_levels_resolve_distinct_ifds() {
    let results: Vec<(f32, i32)> = (1u8..=3)
        .map(run_pipeline_at_level)
        .collect();

    assert!(
        results[0].1 > results[1].1,
        "Level 1 should have more pixels than level 2: {} vs {}",
        results[0].1, results[1].1,
    );
    assert!(
        results[1].1 > results[2].1,
        "Level 2 should have more pixels than level 3: {} vs {}",
        results[1].1, results[2].1,
    );
}