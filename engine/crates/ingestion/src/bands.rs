use predictor::SatellitePassEvent;
use crate::fetch_imagery;
use crate::{compute_ndvi, ndvi_to_geotiff};
use std::time::Duration;
use image::ImageFormat;

pub fn download_bands(event: &SatellitePassEvent) 
    -> Result<(), Box<dyn std::error::Error>> 
{
    let Some((b04_url, b08_url)) = fetch_imagery(event)? else {
        println!("No imagery available for this pass");
        return Ok(());
    };

    let client = reqwest::blocking::Client::new();

    let b04_bytes = client.get(&b04_url)
        //.header("Range", "bytes=0-5000000")
        .send()?.bytes()?;

    let b08_bytes = client.get(&b08_url)
        //.header("Range", "bytes=0-5000000")
        .send()?.bytes()?;

    println!("B04: {} bytes", b04_bytes.len());
    println!("B08: {} bytes", b08_bytes.len());

    let (ndvi, w4, h4) = compute_ndvi(&b04_bytes, &b08_bytes)?; 

    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%SZ");
    let path = format!("ndvi_{}.tif", timestamp);
    ndvi_to_geotiff(&ndvi, w4, h4, &path)?;
    println!("Saved {}", path);

    Ok(())
}

/// Sentinel-2 COGs tag their 12-bit data as 15 bits-per-sample.
/// Neither `image` nor `tiff` accept this, so we patch the IFD tag
/// from 15 → 16 before handing the bytes to the decoder.
fn patch_bits_per_sample(bytes: &bytes::Bytes) -> Vec<u8> {
    let mut data = bytes.to_vec();
    if data.len() < 8 { return data; }

    let le = &data[0..2] == b"II"; // II = little-endian, MM = big-endian

    let read16 = |d: &[u8], off: usize| -> u16 {
        if le { u16::from_le_bytes([d[off], d[off+1]]) }
        else  { u16::from_be_bytes([d[off], d[off+1]]) }
    };

    let write16 = |d: &mut Vec<u8>, off: usize, v: u16| {
        let b = if le { v.to_le_bytes() } else { v.to_be_bytes() };
        d[off] = b[0]; d[off+1] = b[1];
    };

    // Offset to first IFD is at bytes 4-7
    let ifd_off = if le {
        u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize
    } else {
        u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize
    };

    let entry_count = read16(&data, ifd_off) as usize;

    for i in 0..entry_count {
        let off = ifd_off + 2 + i * 12;
        if off + 12 > data.len() { break; }

        let tag = read16(&data, off);
        if tag == 258 {
            // Tag 258 = BitsPerSample, type SHORT (3), count 1
            // Value is stored inline in bytes 8-9 of the entry
            let val = read16(&data, off + 8);
            if val == 15 {
                write16(&mut data, off + 8, 16);
            }
            break;
        }
    }
    data
}

pub fn decode_band(bytes: &bytes::Bytes) -> anyhow::Result<(Vec<u16>, u32, u32)> {
    let patched = patch_bits_per_sample(bytes);

    let img = image::load_from_memory_with_format(&patched, ImageFormat::Tiff)
        .map_err(|e| anyhow::anyhow!("TIFF decode failed: {e}"))?;

    let img16 = img.into_luma16();
    let (w, h) = img16.dimensions();
    let data = img16.into_raw();

    Ok((data, w, h))
}

pub fn handle_pass(event: SatellitePassEvent) {
    let delay = event.pass_end + chrono::Duration::hours(6);
    let wait  = (delay - chrono::Utc::now())
                    .to_std()
                    .unwrap_or(Duration::ZERO);
    std::thread::sleep(wait);

    if let Err(e) = download_bands(&event) {
        eprintln!("Ingestion failed for {}: {e}", event.satellite_id);
    }
}