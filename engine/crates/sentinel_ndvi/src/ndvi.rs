use sentinel_cog::Raster;
use crate::error::{NdviError, NdviResult};

#[derive(Debug, Clone)]
pub struct NdviStats {
    pub mean_ndvi: f32,
    pub max_ndvi: f32,
    pub min_ndvi: f32,
    pub valid_pixels: usize,
}

pub fn compute_ndvi(b04: &Raster, b08: &Raster) -> NdviResult<(Vec<f32>, u32, u32)> {
    if b04.width != b08.width || b04.height != b08.height {
        return Err(NdviError::DimensionMismatch {
            b04: b04.pixels.len(),
            b08: b08.pixels.len(),
        });
    }
    Ok((compute_ndvi_raw(&b04.pixels, &b08.pixels), b04.width, b04.height))
}

/// Compute NDVI from raw u16 pixel slices.
///
/// Pixels where either band is [`sentinel_cog::NODATA`] (`u16::MAX`) are
/// written as `f32::NAN` — QGIS renders these as transparent, preventing
/// nodata from appearing as bare soil (NDVI 0.0) in the output.
///
/// All other pixels where NIR + Red == 0 are clamped to 0.0.
pub fn compute_ndvi_raw(b04: &[u16], b08: &[u16]) -> Vec<f32> {
    b04.iter()
        .zip(b08.iter())
        .map(|(&red, &nir)| {
            if red == sentinel_cog::NODATA || nir == sentinel_cog::NODATA {
                return f32::NAN;
            }
            let r = red as f32;
            let n = nir as f32;
            let denom = n + r;
            if denom == 0.0 { 0.0 } else { (n - r) / denom }
        })
        .collect()
}

/// Compute summary statistics from an NDVI slice, skipping NAN pixels.
///
/// Returns `None` if there are no valid pixels at all.
pub fn compute_stats(ndvi: &[f32]) -> Option<NdviStats> {
    let valid: Vec<f32> = ndvi.iter().copied().filter(|v| !v.is_nan()).collect();

    if valid.is_empty() {
        return None;
    }

    let mean = valid.iter().sum::<f32>() / valid.len() as f32;
    let max = valid.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min = valid.iter().cloned().fold(f32::INFINITY, f32::min);

    Some(NdviStats {
        mean_ndvi: mean,
        max_ndvi: max,
        min_ndvi: min,
        valid_pixels: valid.len(),
    })
}