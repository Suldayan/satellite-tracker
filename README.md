# satellite-tracker

A satellite tracking and remote sensing pipeline written in Rust. Predict the next time any satellite passes over your location, then automatically fetch and process Sentinel-2 imagery when it does — producing NDVI vegetation maps without downloading full 200 MB band files.

![NDVI output — Surrey, BC](docs/surrey_ndvi.png)

---

## What it does

The project started as a WASM-powered satellite tracker — enter your coordinates, get the next visible pass. It evolved into a full remote sensing pipeline when I wanted to do something useful with the imagery Sentinel-2 captures on those passes.

The two halves are:

**Satellite tracking:** Given a TLE (orbital elements) and an observer location, predict every visible pass over the next 24 hours. Runs in the browser via WebAssembly or natively as part of the ingestion pipeline.

**NDVI pipeline:** When a pass occurs, automatically query Microsoft Planetary Computer for the corresponding Sentinel-2 scene, stream only the tiles covering your area of interest via HTTP range requests, compute NDVI, and write a georeferenced GeoTIFF. A full Sentinel-2 band is ~200 MB; this pipeline fetches ~2–5 MB for a city-scale bbox at full 10 m resolution.

---

## Architecture

```
satellite_predictor    ← orbital mechanics, SGP4 propagation, WASM + native
sentinel_types         ← shared types (BBox, SatellitePassEvent)
sentinel_cog           ← COG tile fetching via HTTP range requests
sentinel_ndvi          ← NDVI compute, difference maps, GeoTIFF output
sentinel_orchestrator  ← TLE refresh, pass prediction loop, event emission
sentinel_pipeline      ← STAC query, band fetch, NDVI write
sentinel_runner        ← binary, wires everything together
```

```
TLE from Celestrak
      ↓
sentinel_orchestrator  (predicts passes, emits SatellitePassEvent)
      ↓  mpsc channel
sentinel_pipeline      (queries STAC → fetches COG tiles → computes NDVI)
      ↓
Float32 GeoTIFF on disk
```

---

## Stack

- **Rust** — core pipeline, orbital mechanics, COG parsing
- **WebAssembly** — satellite tracker runs in the browser via `wasm-bindgen`
- **HTML / CSS / JS** — frontend tracker UI
- **Sentinel-2 L2A** — imagery source via [Microsoft Planetary Computer](https://planetarycomputer.microsoft.com/)
- **QGIS** — visualisation and validation

---

## Crates

### `sentinel_cog`

Streams Cloud Optimized GeoTIFF tiles via HTTP range requests. Parses the TIFF IFD at the byte level to locate tile offsets, reads the embedded geotransform (tags 33550 + 33922), and filters to only the tiles that intersect a given bounding box before fetching.

```rust
// ~500 KB at overview resolution
let raster = fetch_overview(&client, &url, 3)?;

// ~2–5 MB at full 10 m resolution, Surrey only
let raster = fetch_overview_bbox(&client, &url, 0, &BBox::surrey_bc())?;
```

→ [crates.io/crates/sentinel_cog](https://crates.io/crates/sentinel_cog)

---

### `sentinel_ndvi`

Pure-compute NDVI processing. No HTTP, no I/O beyond writing the result.

```rust
let (ndvi, w, h) = compute_ndvi(&b04, &b08)?;
write_f32_tiff(&ndvi, w, h, "ndvi.tif", &GeoRef::utm10n_10m())?;

// Change detection between two dates
let diff = calc_difference_map(&past_ndvi, &present_ndvi)?;
println!("Mean change: {:.3}", diff.mean_change);
```

→ [crates.io/crates/sentinel_ndvi](https://crates.io/crates/sentinel_ndvi)

---

## NDVI output

NDVI (Normalized Difference Vegetation Index) measures vegetation density:

```
NDVI = (NIR − Red) / (NIR + Red)
```

Output is a Float32 GeoTIFF. Load in QGIS with *Singleband pseudocolor* + *RdYlGn* colormap at 2–98% percentile stretch.

| Color | NDVI | Meaning |
|-------|------|---------|
| Deep green | > 0.6 | Dense, healthy vegetation |
| Yellow-green | 0.2 – 0.4 | Moderate vegetation |
| Orange | ~0 | Bare soil, urban |
| Red | < 0 | Water, shadow |

---

## Running locally

```bash
git clone https://github.com/your-username/satellite-tracker
cd satellite-tracker/engine
cargo build --workspace
cargo run --bin sentinel_runner
```

Set `RUST_LOG=info` to see pipeline output.

To run the ignored integration tests (requires network):

```bash
cargo test --workspace -- --ignored --nocapture
```

## Planned

- SCL (Scene Classification Layer) masking to remove cloud and shadow pixels before computing NDVI
- PostGIS time series storage for multi-date change detection queries
- Azure Functions deployment for fully automated ingestion

---

## License

MIT
