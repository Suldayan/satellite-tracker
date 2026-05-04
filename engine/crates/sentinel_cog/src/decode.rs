use bytes::Bytes;
use log::debug;
use crate::error::{CogError, CogResult};
use crate::fetch::fetch_range;
use crate::parse::IfdInfo;

#[derive(Debug, Clone)]
pub struct Raster {
    pub pixels: Vec<u16>,
    pub width:  u32,
    pub height: u32,
}

/// Fetch every tile referenced by `info` and return the raw compressed bytes.
pub fn fetch_tiles(
    client: &reqwest::blocking::Client,
    url: &str,
    info: &IfdInfo,
) -> CogResult<Vec<Bytes>> {
    info.tile_offsets
        .iter()
        .enumerate()
        .map(|(i, &(offset, len))| {
            debug!("Fetching tile {i}: bytes={offset}-{}", offset + len - 1);
            fetch_range(client, url, offset, offset + len - 1)
        })
        .collect()
}

/// Decompress Zlib-encoded tiles and stitch them into a single [`Raster`].
/// `le` must match the TIFF file's byte order (from [`crate::parse::is_little_endian`]).
pub fn decode_tiles(tiles: Vec<Bytes>, info: &IfdInfo, le: bool) -> CogResult<Raster> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let IfdInfo { img_w, img_h, tile_w, tile_h, tiles_across, .. } = *info;
    let mut pixels = vec![0u16; (img_w * img_h) as usize];

    for (i, tile_bytes) in tiles.iter().enumerate() {
        let mut raw = Vec::new();
        ZlibDecoder::new(tile_bytes.as_ref())
            .read_to_end(&mut raw)
            .map_err(|source| CogError::DecompressFailed { index: i, source })?;

        let tile_pixels: Vec<u16> = raw
            .chunks_exact(2)
            .map(|c| {
                if le { u16::from_le_bytes([c[0], c[1]]) }
                else { u16::from_be_bytes([c[0], c[1]]) }
            })
            .collect();

        let tile_col = (i as u32) % tiles_across;
        let tile_row = (i as u32) / tiles_across;
        let x_start = tile_col * tile_w;
        let y_start = tile_row * tile_h;

        for ty in 0..tile_h {
            let y = y_start + ty;
            if y >= img_h { break; }
            for tx in 0..tile_w {
                let x = x_start + tx;
                if x >= img_w { break; }
                let src = (ty * tile_w + tx) as usize;
                let dst = (y  * img_w  + x)  as usize;
                if src < tile_pixels.len() {
                    pixels[dst] = tile_pixels[src];
                }
            }
        }

        debug!("Decoded tile {i} ({tile_col},{tile_row})");
    }

    Ok(Raster { pixels, width: img_w, height: img_h })
}