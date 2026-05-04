//! # sentinel_cog
//!
//! Stream individual tiles from Sentinel-2 Cloud Optimized GeoTIFFs (COGs)
//! over HTTP range requests — no full-band download required.
//!
//! ## Quick start
//!
//! ```no_run
//! use sentinel_cog::{fetch_overview, fetch_overview_bbox, CogError};
//! use sentinel_types::BBox;
//!
//! let client = reqwest::blocking::Client::new();
//!
//! // All tiles at overview level 3 (~300m resolution)
//! let raster = fetch_overview(&client, "https://…/B04.tif", 3)?;
//!
//! // Only tiles intersecting Surrey at full resolution
//! let bbox = BBox { min_lon: -122.95, max_lon: -122.65, min_lat: 49.05, max_lat: 49.35 };
//! let raster = fetch_overview_bbox(&client, "https://…/B04.tif", 0, &bbox)?;
//! # Ok::<(), CogError>(())
//! ```

mod decode;
mod error;
mod fetch;
pub mod parse;
mod geo;

pub use decode::Raster;
pub use error::{CogError, CogResult};
pub use parse::{IfdInfo, GeoTransform, is_little_endian, parse_subifds, parse_ifd_bytes};
use geo::{filter_tiles};

use sentinel_types::BBox;

pub fn fetch_overview(
    client: &reqwest::blocking::Client,
    url: &str,
    overview_level: usize,
) -> CogResult<Raster> {
    let (info, le) = fetch_ifd(client, url, overview_level)?;
    let tiles = decode::fetch_tiles(client, url, &info)?;
    decode::decode_tiles(tiles, &info, le)
}

/// Fetch only the tiles intersecting `bbox`, at the given overview level.
///
/// Requires the TIFF to have embedded georeferencing (tags 33550 + 33922).
/// Falls back to fetching all tiles if the tags are absent.
pub fn fetch_overview_bbox(
    client: &reqwest::blocking::Client,
    url: &str,
    overview_level: usize,
    bbox: &BBox,
) -> CogResult<Raster> {
    let (mut info, le) = fetch_ifd(client, url, overview_level)?;
    info.tile_offsets = geo::filter_tiles(&info, bbox);
    let tiles = decode::fetch_tiles(client, url, &info)?;
    decode::decode_tiles(tiles, &info, le)
}

fn fetch_ifd(
    client: &reqwest::blocking::Client,
    url: &str,
    overview_level: usize,
) -> CogResult<(IfdInfo, bool)> {
    let header = fetch::fetch_header(client, url)?;
    let le = is_little_endian(&header)?;
    let subifd_offsets = parse_subifds(&header)?;

    let ifd_offset = subifd_offsets
        .get(overview_level)
        .copied()
        .unwrap_or_else(|| *subifd_offsets.last().unwrap());

    let ifd_bytes = fetch::fetch_ifd_block(client, url, ifd_offset)?;
    let info = parse_ifd_bytes(client, url, &ifd_bytes, le)?;
    Ok((info, le))
}