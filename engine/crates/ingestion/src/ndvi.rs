use image::{GrayImage};
use std::path::Path;
use crate::bands::decode_band;

pub fn compute_ndvi(
    b04_bytes: &bytes::Bytes,
    b08_bytes: &bytes::Bytes,
) -> anyhow::Result<(Vec<f32>, u32, u32)> {
    let (b04, w4, h4) = decode_band(b04_bytes)?;
    let (b08, w8, h8) = decode_band(b08_bytes)?;

    if w4 != w8 || h4 != h8 {
        anyhow::bail!(
            "Band dimension mismatch: B04 = {}x{}, B08 = {}x{}",
            w4, h4, w8, h8
        );
    }

    let size = b04.len();
    if b08.len() != size {
        anyhow::bail!(
            "Band buffer length mismatch: B04 = {}, B08 = {}",
            size, b08.len()
        );
    }

    let mut output = Vec::with_capacity(size);
    for i in 0..size {
        let red = b04[i] as f32;
        let nir = b08[i] as f32;
        let ndvi = if (nir + red) == 0.0 {
            0.0
        } else {
            (nir - red) / (nir + red)
        };
        output.push(ndvi);
    }

    Ok((output, w4, h4))
}

pub fn ndvi_to_geotiff(ndvi: &[f32], w: u32, h: u32, path: &str) -> anyhow::Result<()> {
    let pixels: Vec<u8> = ndvi.iter()
        .map(|&v| ((v + 1.0) / 2.0 * 255.0).clamp(0.0, 255.0) as u8)
        .collect();

    let img = GrayImage::from_raw(w, h, pixels)
        .ok_or_else(|| anyhow::anyhow!("Buffer size mismatch: {}x{} != {} pixels", w, h, ndvi.len()))?;

    img.save_with_format(Path::new(path), image::ImageFormat::Tiff)
        .map_err(|e| anyhow::anyhow!("Failed to save TIFF: {e}"))?;

    Ok(())
}