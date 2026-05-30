# satellite-tracker

A satellite remote sensing pipeline written in Rust. Automatically fetches Sentinel-2 imagery when a satellite passes over a defined region, computes NDVI vegetation indices, and persists time-series statistics to PostgreSQL/PostGIS without downloading full 200 MB band files.

![NDVI output — Surrey, BC](docs/surrey_ndvi.png)

---

## What it does

Sentinel-2 satellites capture multispectral imagery of the Earth every 5 days. Each band file is ~200 MB. This pipeline streams only the tiles covering your area of interest via HTTP range requests, computes NDVI, writes a georeferenced GeoTIFF, and stores summary statistics in a PostGIS database for time-series analysis.

**The result:** 88% less data transferred at full 10m resolution, 95% less at 20m — without sacrificing spatial coverage of the target region.

---

## Architecture

```
sentinel_types         ← shared data types (BBox, SatellitePassEvent, NdviRecord)
sentinel_cog           ← COG tile fetching via HTTP range requests
sentinel_ndvi          ← NDVI compute, stats, GeoTIFF output
sentinel_pipeline      ← STAC query, band fetch, end-to-end ingest
sentinel_db            ← PostgreSQL/PostGIS insert with retry logic
sentinel_orchestrator  ← entry point, loads config, wires pipeline + db
```

```
Config (env vars)
      ↓
sentinel_orchestrator
      ↓ SatellitePassEvent
sentinel_pipeline  →  STAC query → COG fetch → NDVI compute → GeoTIFF
      ↓ NdviRecord
sentinel_db        →  INSERT INTO ndvi_history (PostGIS)
```

---

## Stack

- **Rust** — entire pipeline, zero runtime overhead
- **Sentinel-2 L2A** — imagery source via [Microsoft Planetary Computer](https://planetarycomputer.microsoft.com/)
- **PostgreSQL + PostGIS** — time-series vegetation statistics
- **Docker** — multi-stage Alpine build, production-ready container
- **QGIS** — visualisation and validation

---

## Published Crates

### `sentinel_cog`
Streams Cloud Optimized GeoTIFF tiles via HTTP range requests. Parses the TIFF IFD at the byte level, reads the embedded geotransform (tags 33550 + 33922), and filters to only the tiles intersecting a bounding box before fetching.

```rust
// ~500 KB at overview level 3
let raster = fetch_overview(&client, &url, 3)?;

// 23.9 MB at full 10m resolution, Surrey bbox only (vs ~200 MB full band)
let raster = fetch_overview_bbox(&client, &url, 0, &BBox::surrey_bc())?;
```

→ [crates.io/crates/sentinel_cog](https://crates.io/crates/sentinel_cog)

---

### `sentinel_ndvi`
Pure-compute NDVI processing. No HTTP, no I/O beyond writing the result.

```rust
let (ndvi, w, h) = compute_ndvi(&b04, &b08)?;
let stats = compute_stats(&ndvi)?;
write_f32_tiff(&ndvi, w, h, "ndvi.tif", &GeoRef::utm10n_10m())?;

// Change detection between two dates
let diff = calc_difference_map(&past_ndvi, &present_ndvi)?;
println!("Mean change: {:.3}", diff.mean_change);
```

→ [crates.io/crates/sentinel_ndvi](https://crates.io/crates/sentinel_ndvi)

---

## NDVI Output

NDVI measures vegetation density using near-infrared and red reflectance:

```
NDVI = (NIR − Red) / (NIR + Red)
```

Output is a Float32 GeoTIFF with DEFLATE compression. Load in QGIS with *Singleband pseudocolor* + *RdYlGn* colormap at 2–98% percentile stretch.

| Overview | Dimensions | File size | vs full band |
|----------|-----------|-----------|--------------|
| 0 (10m) | 3584 × 5120 | 23.9 MB | 88% reduction |
| 1 (20m) | 2048 × 3072 | 9.82 MB | 95% reduction |
| Full band | 10980 × 10980 | ~200 MB | baseline |

---

## Running locally

```bash
git clone https://github.com/Suldayan/satellite-tracker
cd satellite-tracker/engine
```

Create a `.env` file:

```env
DATABASE_URL=host=localhost user=postgres password=secret dbname=gisdb
SATELLITE_ID=SENTINEL-2A
MIN_LON=-122.95
MAX_LON=-122.65
MIN_LAT=49.05
MAX_LAT=49.35
OVERVIEW_LEVEL=1
LOOKBACK_DAYS=5
RUST_LOG=info
```

Run the pipeline:

```bash
cargo run --bin sentinel_orchestrator
```

Or run via Docker:

```bash
docker build -t sentinel-orchestrator .
docker run --env-file .env sentinel-orchestrator
```

---

## Running tests

Unit and integration tests:

```bash
cargo test --workspace
```

End-to-end ignored tests (requires network):

```bash
cargo test --workspace -- --ignored --nocapture
```

Database tests (requires Docker):

```bash
cargo test --package sentinel_db
```

---

## PostGIS schema

```sql
CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE ndvi_history (
    id           SERIAL PRIMARY KEY,
    captured_at  TIMESTAMPTZ NOT NULL,
    satellite_id TEXT NOT NULL,
    min_lon      DOUBLE PRECISION NOT NULL,
    max_lon      DOUBLE PRECISION NOT NULL,
    min_lat      DOUBLE PRECISION NOT NULL,
    max_lat      DOUBLE PRECISION NOT NULL,
    mean_ndvi    REAL NOT NULL,
    max_ndvi     REAL NOT NULL,
    min_ndvi     REAL NOT NULL,
    valid_pixels INTEGER NOT NULL,
    tif_path     TEXT NOT NULL,
    bbox         GEOMETRY(POLYGON, 4326)
);

CREATE INDEX ndvi_history_bbox_idx ON ndvi_history USING GIST (bbox);
CREATE INDEX ndvi_history_time_idx ON ndvi_history (captured_at);
```

Example time-series query:

```sql
SELECT DATE_TRUNC('month', captured_at), AVG(mean_ndvi)
FROM ndvi_history
GROUP BY 1
ORDER BY 1;
```
---

## License

MIT
