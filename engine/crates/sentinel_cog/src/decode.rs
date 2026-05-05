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

/// Fetch tiles by `(original_index, offset, byte_count)` triples.
/// The original index is threaded through so `decode_tiles` can place
/// each tile at the correct position in the output raster.
pub fn fetch_tiles(
    client: &reqwest::blocking::Client,
    url: &str,
    tiles: &[(usize, u64, u64)],
) -> CogResult<Vec<(usize, Bytes)>> {
    tiles
        .iter()
        .map(|&(i, offset, len)| {
            debug!("Fetching tile {i}: bytes={offset}-{}", offset + len - 1);
            let bytes = fetch_range(client, url, offset, offset + len - 1)?;
            Ok((i, bytes))
        })
        .collect()
}

/// Decompress Zlib-encoded tiles and stitch them into a single [`Raster`].
/// Each tile is placed using its original grid index, so sparse/filtered
/// tile sets are positioned correctly within the full image canvas.
pub fn decode_tiles(tiles: Vec<(usize, Bytes)>, info: &IfdInfo, le: bool) -> CogResult<Raster> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let IfdInfo { img_w, img_h, tile_w, tile_h, tiles_across, .. } = *info;
    let mut pixels = vec![0u16; (img_w * img_h) as usize];

    for (original_index, tile_bytes) in &tiles {
        let mut raw = Vec::new();
        ZlibDecoder::new(tile_bytes.as_ref())
            .read_to_end(&mut raw)
            .map_err(|source| CogError::DecompressFailed { index: *original_index, source })?;

        let tile_pixels: Vec<u16> = raw
            .chunks_exact(2)
            .map(|c| {
                if le { u16::from_le_bytes([c[0], c[1]]) }
                else  { u16::from_be_bytes([c[0], c[1]]) }
            })
            .collect();

        let tile_col = (*original_index as u32) % tiles_across;
        let tile_row = (*original_index as u32) / tiles_across;
        let x_start  = tile_col * tile_w;
        let y_start  = tile_row * tile_h;

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

        debug!("Decoded tile {original_index} ({tile_col},{tile_row})");
    }

    Ok(Raster { pixels, width: img_w, height: img_h })
}

pub fn decode_tiles_region(
    tiles: Vec<(usize, Bytes)>,
    info: &IfdInfo,
    le: bool,
    min_col: u32,
    min_row: u32,
    out_w: u32,
    out_h: u32,
) -> CogResult<Raster> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let IfdInfo { tile_w, tile_h, tiles_across, .. } = *info;
    let mut pixels = vec![0u16; (out_w * out_h) as usize];

    for (original_index, tile_bytes) in &tiles {
        let mut raw = Vec::new();
        ZlibDecoder::new(tile_bytes.as_ref())
            .read_to_end(&mut raw)
            .map_err(|source| CogError::DecompressFailed { index: *original_index, source })?;

        let tile_pixels: Vec<u16> = raw
            .chunks_exact(2)
            .map(|c| if le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) })
            .collect();

        let tile_col = (*original_index as u32) % tiles_across - min_col;
        let tile_row = (*original_index as u32) / tiles_across - min_row;
        let x_start  = tile_col * tile_w;
        let y_start  = tile_row * tile_h;

        for ty in 0..tile_h {
            let y = y_start + ty;
            if y >= out_h { break; }
            for tx in 0..tile_w {
                let x = x_start + tx;
                if x >= out_w { break; }
                let src = (ty * tile_w + tx) as usize;
                let dst = (y * out_w + x) as usize;
                if src < tile_pixels.len() {
                    pixels[dst] = tile_pixels[src];
                }
            }
        }
    }

    Ok(Raster { pixels, width: out_w, height: out_h })
}