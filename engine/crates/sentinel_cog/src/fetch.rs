use bytes::Bytes;
use log::debug;
use crate::error::{CogError, CogResult};
use reqwest::blocking::Client;

/// How many bytes to grab for the initial TIFF header + IFD scan.
/// 16 KiB is enough for all Sentinel-2 COG headers encountered in practice.
const HEADER_FETCH_SIZE: u64 = 16_383;

/// Fetch the TIFF header via a single HTTP range request.
///
/// Returns the raw bytes starting at offset 0. The caller is responsible
/// for parsing byte-order and locating the first IFD.
pub fn fetch_header(client: &Client, url: &str) -> CogResult<Bytes> {
    debug!("Fetching COG header ({} bytes) from {url}", HEADER_FETCH_SIZE + 1);
    fetch_range(client, url, 0, HEADER_FETCH_SIZE)
}

/// Fetch an arbitrary byte range `[start, end]` (inclusive) from `url`.
pub fn fetch_range(
    client: &Client,
    url: &str,
    start: u64,
    end: u64,
) -> CogResult<Bytes> {
    debug!("HTTP range request bytes={start}-{end} from {url}");
    let bytes = client
        .get(url)
        .header("Range", format!("bytes={start}-{end}"))
        .send()
        .map_err(CogError::Http)?
        .error_for_status()
        .map_err(CogError::Http)?
        .bytes()
        .map_err(CogError::Http)?;

    debug!("Received {} bytes", bytes.len());
    Ok(bytes)
}

/// Fetch the 4 KiB block starting at `offset` — enough to read a full IFD.
pub fn fetch_ifd_block(
    client: &Client,
    url: &str,
    offset: u32,
) -> CogResult<Bytes> {
    fetch_range(client, url, offset as u64, offset as u64 + 4_095)
}