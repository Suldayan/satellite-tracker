# sentinel_cog

Stream individual tiles from Sentinel-2 Cloud Optimized GeoTIFFs (COGs) via HTTP range requests — no full-band download required.

Sentinel-2 bands are ~200 MB each. This crate fetches only the tiles intersecting your bounding box, reducing a full-resolution Surrey, BC acquisition from ~200 MB to 23.9 MB (88% reduction) with no loss of spatial resolution.

## Install

```toml
[dependencies]
sentinel_cog = "0.1"
```

## Quick start

```rust
use sentinel_cog::{fetch_overview, fetch_overview_bbox, CogError};
use sentinel_types::BBox;

let client = reqwest::blocking::Client::new();
let url = "https://…/B04.tif"; // signed Sentinel-2 COG URL

// All tiles at overview level 1 (~20m resolution)
let raster = fetch_overview(&client, url, 1)?;

// Full 10m resolution, Surrey BC only — 23.9 MB vs ~200 MB full band
let bbox = BBox::surrey_bc();
let raster = fetch_overview_bbox(&client, url, 0, &bbox)?;
```

## Overview levels

Sentinel-2 COGs embed multiple resolution overviews. Not all levels exist for every tile — `fetch_overview_bbox` returns an error if the requested level is absent rather than silently falling back.

| Level | Resolution | Surrey bbox size | vs full band |
|-------|-----------|-----------------|--------------|
| 0 | 10 m | 23.9 MB | 88% reduction |
| 1 | 20 m | 9.82 MB | 95% reduction |
| Full band | 10 m | ~200 MB | baseline |

## How it works

Sentinel-2 COGs store image data in a tiled, multi-resolution structure. Rather than downloading the entire file, `sentinel_cog`:

1. Fetches the first 16 KB to read the TIFF header and locate the Image File Directory
2. Parses IFD tags to find tile byte offsets, image dimensions, and the embedded geotransform (tags 33550 + 33922)
3. Converts the bounding box from WGS84 lat/lon to the TIFF's native CRS (UTM) and filters to only intersecting tiles
4. Fetches each required tile as a separate HTTP range request
5. Decompresses (Zlib) and stitches tiles into a single contiguous raster
6. Fills non-fetched pixels with `NODATA` (`u16::MAX`) so downstream compute can distinguish missing data from valid zero-reflectance pixels

## Output

```rust
pub struct Raster {
    pub pixels: Vec<u16>,  // raw u16 reflectance values
    pub width: u32,
    pub height: u32,
}
```

Raw `u16` values are returned so callers can perform band math at full precision. See [`sentinel_ndvi`](https://crates.io/crates/sentinel_ndvi) for NDVI computation and GeoTIFF output.

## Error handling

```rust
match sentinel_cog::fetch_overview_bbox(&client, &url, 0, &bbox) {
    Err(CogError::Http(e))                        => // retry
    Err(CogError::MissingTag { tag, name })       => // unsupported TIFF
    Err(CogError::DecompressFailed { index, .. }) => // corrupt tile
    Err(CogError::InvalidHeader(_))               => // bad overview level
    Ok(raster)                                    => // use raster
}
```

## Data source

Signed URLs for Sentinel-2 L2A COGs are available free from [Microsoft Planetary Computer](https://planetarycomputer.microsoft.com/).

## License

MIT