use chrono::Utc;
use sentinel_types::NdviRecord;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::SyncRunner, ImageExt},
};

fn make_record() -> NdviRecord {
    NdviRecord {
        captured_at: Utc::now(),
        satellite_id: "SENTINEL-2A".into(),
        min_lon: -122.95,
        max_lon: -122.65,
        min_lat: 49.05,
        max_lat: 49.35,
        mean_ndvi: 0.45,
        max_ndvi: 0.82,
        min_ndvi: -0.12,
        valid_pixels: 100_000,
        tif_path: "test.tif".into(),
    }
}

#[test]
fn test_ndvi_database_workflow() {
    let container = Postgres::default()
        .with_name("postgis/postgis")
        .with_tag("15-3.3")
        .start()
        .expect("PostGIS container failed to start — is Docker running?");

    let port = container.get_host_port_ipv4(5432).unwrap();
    let conn_str = format!(
        "host=localhost port={port} user=postgres password=postgres dbname=postgres"
    );

    let mut client = postgres::Client::connect(&conn_str, postgres::NoTls)
        .expect("Failed to connect to test database");

    client.batch_execute("
        CREATE EXTENSION IF NOT EXISTS postgis;

        CREATE TABLE IF NOT EXISTS ndvi_history (
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

        CREATE INDEX ndvi_history_bbox_idx ON ndvi_history USING GIST (bbox);
        CREATE INDEX ndvi_history_time_idx ON ndvi_history (captured_at);
    ").expect("Schema migration failed");

    unsafe {
        std::env::set_var("DATABASE_URL", &conn_str);
    }
    
    let record = make_record();
    let result = sentinel_db::insert_ndvi_result(&record);

    assert!(result.is_ok(), "Insert failed: {:?}", result.err());

    let row = client
        .query_one("SELECT mean_ndvi, max_ndvi, min_ndvi, valid_pixels, satellite_id FROM ndvi_history", &[])
        .expect("Query failed");

    let mean: f32 = row.get(0);
    let max: f32 = row.get(1);
    let min: f32 = row.get(2);
    let pixels: i32 = row.get(3);
    let sat_id: String = row.get(4);

    assert!((mean -  0.45).abs() < 1e-5, "mean_ndvi mismatch: {mean}");
    assert!((max -  0.82).abs() < 1e-5, "max_ndvi mismatch: {max}");
    assert!((min - -0.12).abs() < 1e-5, "min_ndvi mismatch: {min}");
    assert_eq!(pixels, 100_000);
    assert_eq!(sat_id, "SENTINEL-2A");

    let bbox_row = client
        .query_one("SELECT ST_AsText(bbox) FROM ndvi_history", &[])
        .expect("Bbox query failed");
    let wkt: String = bbox_row.get(0);
    assert!(wkt.contains("POLYGON"), "Expected a polygon bbox, got: {wkt}");

    unsafe {
        std::env::set_var("DATABASE_URL", "host=localhost port=1 user=nobody dbname=nobody");
    }
    let bad_result = sentinel_db::insert_ndvi_result(&record);
    assert!(bad_result.is_err(), "Expected error on bad connection");
}