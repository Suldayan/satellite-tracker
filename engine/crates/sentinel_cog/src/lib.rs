//! # sentinel_cog
//!
//! Stream individual tiles from Sentinel-2 Cloud Optimized GeoTIFFs (COGs)
//! over HTTP range requests — no full-band download required.
//!
//! ## Quick start
//!
//! ```no_run
//! use sentinel_cog::{fetch_overview, CogError};
//!
//! let client = reqwest::blocking::Client::new();
//! let raster = fetch_overview(&client, "https://…/B04.tif", 3)?;
//! println!("{}x{}", raster.width, raster.height);
//! # Ok::<(), CogError>(())
//! ```

mod decode;
mod error;
mod fetch;
pub mod parse;

pub use decode::Raster;
pub use error::{CogError, CogResult};
pub use parse::{IfdInfo, is_little_endian, parse_subifds, parse_ifd_bytes};

/// Fetch a single overview level from a Sentinel-2 COG band.
///
/// `overview_level` is 0-based where 0 = full resolution. Level 3 gives
/// roughly 300 m resolution and downloads ~500 KB per band.
///
/// # Errors
///
/// Returns [`CogError`] on HTTP failure, invalid TIFF structure, or
/// decompression error.
pub fn fetch_overview(
    client:         &reqwest::blocking::Client,
    url:            &str,
    overview_level: usize,
) -> CogResult<Raster> {
    let header = fetch::fetch_header(client, url)?;
    let le     = is_little_endian(&header)?;

    let subifd_offsets = parse_subifds(&header)?;

    // SubIFDs are ordered: [full-res, overview-0, overview-1, …]
    // overview_level 0 = full resolution (first SubIFD)
    let ifd_offset = subifd_offsets
        .get(overview_level)
        .copied()
        .unwrap_or_else(|| *subifd_offsets.last().unwrap());

    let ifd_bytes = fetch::fetch_ifd_block(client, url, ifd_offset)?;
    let info      = parse_ifd_bytes(client, url, &ifd_bytes, le)?;
    let tiles     = decode::fetch_tiles(client, url, &info)?;

    decode::decode_tiles(tiles, &info, le)
}