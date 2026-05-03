use sentinel_cog::Raster;
use crate::error::{NdviError, NdviResult};

/// Compute per-pixel NDVI from two aligned raster bands.
///
/// Returns `(ndvi_values, width, height)`.
///
/// # Errors
///
/// Returns [`NdviError::DimensionMismatch`] when the bands differ in size.
pub fn compute_ndvi(b04: &Raster, b08: &Raster) -> NdviResult<(Vec<f32>, u32, u32)> {
    if b04.width != b08.width || b04.height != b08.height {
        return Err(NdviError::DimensionMismatch {
            b04: b04.pixels.len(),
            b08: b08.pixels.len(),
        });
    }
    Ok((compute_ndvi_raw(&b04.pixels, &b08.pixels), b04.width, b04.height))
}

/// Compute NDVI from raw pixel slices.
///
/// Output values are in `[-1.0, 1.0]`. Pixels where `NIR + Red == 0`
/// (e.g. sensor fill values) are clamped to `0.0`.
pub fn compute_ndvi_raw(b04: &[u16], b08: &[u16]) -> Vec<f32> {
    b04.iter()
        .zip(b08.iter())
        .map(|(&red, &nir)| {
            let r = red as f32;
            let n = nir as f32;
            let denom = n + r;
            if denom == 0.0 { 0.0 } else { (n - r) / denom }
        })
        .collect()
}