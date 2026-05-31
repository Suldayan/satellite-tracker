# sentinel_ndvi

Pure-compute NDVI processing for Sentinel-2 rasters. Takes raw band data, returns NDVI values, summary statistics, and GeoTIFF output — no HTTP, no scheduling, no I/O beyond writing the result.

## Install

```toml
[dependencies]
sentinel_ndvi = "0.1"
```

## Quick start

```rust
use sentinel_ndvi::{compute_ndvi, compute_stats, write_f32_tiff, GeoRef};

// b04 and b08 are Raster values from sentinel_cog
let (ndvi, width, height) = compute_ndvi(&b04, &b08)?;
let stats = compute_stats(&ndvi).expect("No valid pixels");

println!("Mean NDVI: {:.3}", stats.mean_ndvi);
println!("Valid pixels: {}", stats.valid_pixels);

// Write a Float32 GeoTIFF with DEFLATE compression
write_f32_tiff(&ndvi, width, height, "ndvi.tif", &GeoRef::utm10n_10m())?;
```

## NDVI

NDVI (Normalized Difference Vegetation Index) measures vegetation density:

```
NDVI = (NIR - Red) / (NIR + Red)
```

| NDVI range | Meaning |
|------------|---------|
| < 0 | Water, shadow, cloud |
| ~0 | Bare soil, rock, urban |
| 0.1 – 0.3 | Sparse or dry vegetation |
| 0.3 – 0.6 | Moderate vegetation |
| > 0.6 | Dense, healthy canopy |

Pixels where either input band is `NODATA` (`u16::MAX` from `sentinel_cog`) are written as `f32::NAN` and excluded from all statistics. QGIS renders NAN pixels as transparent in Float32 GeoTIFFs.

## Summary statistics

```rust
let stats = compute_stats(&ndvi).unwrap();

println!("{:.3}", stats.mean_ndvi);    // mean of valid pixels
println!("{:.3}", stats.max_ndvi);     // maximum valid value
println!("{:.3}", stats.min_ndvi);     // minimum valid value
println!("{}", stats.valid_pixels);    // count of non-NAN pixels
```

## GeoTIFF output

Output is a single-band Float32 GeoTIFF with DEFLATE compression. Load in QGIS with *Singleband pseudocolor* + *RdYlGn* colormap at 2–98% percentile stretch for best results.

```rust
// Built-in: UTM Zone 10N, 10m pixels — Surrey/BC Sentinel-2 tiles
write_f32_tiff(&ndvi, w, h, "ndvi.tif", &GeoRef::utm10n_10m())?;

// Custom georeferencing for any tile
let georef = GeoRef {
    pixel_size_x: 10.0,
    pixel_size_y: -10.0,
    origin_x: 499_980.0,
    origin_y: 5_500_020.0,
    prj_wkt: "…",
};
```

## Change detection

```rust
use sentinel_ndvi::{calc_difference_map, DifferenceMap};

let diff: DifferenceMap = calc_difference_map(&past_ndvi, &present_ndvi)?;

println!("Mean change: {:.3}", diff.mean_change);
println!("Max decline: {:.3}", diff.max_decline);
println!("Max growth: {:.3}", diff.max_growth);
println!("Valid pixels: {}", diff.valid_pixels);
```

NAN pixels are skipped in difference map calculations.

## Error handling

```rust
match compute_ndvi(&b04, &b08) {
    Err(NdviError::DimensionMismatch { b04, b08 }) => // bands misaligned
    Err(NdviError::LengthMismatch { past, present }) => // diff map inputs differ
    Ok((ndvi, w, h)) => // proceed
}
```

## Used with

[`sentinel_cog`](https://crates.io/crates/sentinel_cog) — fetch Sentinel-2 band rasters via HTTP range requests.

## License

MIT