mod models;

use predictor::SatellitePassEvent;
use std::time::Duration;
use reqwest;
use models::{SignedResponse, StacResponse, DifferenceMap};
use std::io::Cursor;
use tiff::decoder::{Decoder, DecodingResult};
use std::fs::File;
use tiff::encoder::{TiffEncoder, colortype};
use chrono;

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

fn sign_url(client: &reqwest::blocking::Client, href: &str) 
    -> Result<String, Box<dyn std::error::Error>> 
{
    let resp: SignedResponse = client
        .get("https://planetarycomputer.microsoft.com/api/sas/v1/sign")
        .query(&[("href", href)])
        .send()?
        .json()?;

    Ok(resp.href)
}

fn fetch_imagery(event: &SatellitePassEvent) 
    -> Result<Option<(String, String)>, Box<dyn std::error::Error>> 
{
    let client = reqwest::blocking::Client::new();

    let datetime = format!(
        "{}/{}",
        event.pass_start.format("%Y-%m-%dT%H:%M:%SZ"),
        event.pass_end.format("%Y-%m-%dT%H:%M:%SZ"),
    );

    let body = serde_json::json!({
        "collections": ["sentinel-2-l2a"],
        "bbox": [event.min_lon, event.min_lat, event.max_lon, event.max_lat],
        "datetime": datetime,
        "query": { "eo:cloud_cover": { "lt": 20 } }
    });

    let stac: StacResponse = client
        .post("https://planetarycomputer.microsoft.com/api/stac/v1/search")
        .json(&body)
        .send()?
        .json()?;

    if stac.features.is_empty() {
        println!("No scenes found for this pass (cloud cover or timing)");
        return Ok(None);
    }

    // Take the first (best) scene
    let scene = &stac.features[0];
    let b04_url = sign_url(&client, &scene.assets.b04.href)?;
    let b08_url = sign_url(&client, &scene.assets.b08.href)?;

    Ok(Some((b04_url, b08_url)))
}

fn download_bands(event: &SatellitePassEvent) 
    -> Result<(), Box<dyn std::error::Error>> 
{
    let Some((b04_url, b08_url)) = fetch_imagery(event)? else {
        println!("No imagery available for this pass");
        return Ok(());
    };

    let client = reqwest::blocking::Client::new();

    let b04_bytes = client.get(&b04_url)
        .header("Range", "bytes=0-5000000")
        .send()?.bytes()?;

    let b08_bytes = client.get(&b08_url)
        .header("Range", "bytes=0-5000000")
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

fn compute_ndvi(
    b04_bytes: &bytes::Bytes,
    b08_bytes: &bytes::Bytes,
) -> anyhow::Result<(Vec<f32>, u32, u32)> {
    // Decode both bands
    let (b04, w4, h4) = decode_band(b04_bytes)?;
    let (b08, w8, h8) = decode_band(b08_bytes)?;

    // Dimension check
    if w4 != w8 || h4 != h8 {
        anyhow::bail!(
            "Band dimension mismatch: B04 = {}x{}, B08 = {}x{}",
            w4,
            h4,
            w8,
            h8
        );
    }

    let size = b04.len();
    if b08.len() != size {
        anyhow::bail!(
            "Band buffer length mismatch: B04 = {}, B08 = {}",
            size,
            b08.len()
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

fn ndvi_to_geotiff(ndvi: &[f32], w: u32, h: u32, path: &str) -> anyhow::Result<()> {
    let pixels: Vec<u8> = ndvi.iter()
        .map(|&v| ((v + 1.0) / 2.0 * 255.0).clamp(0.0, 255.0) as u8)
        .collect();

    let file = File::create(path)?;
    let mut encoder = TiffEncoder::new(file)?;
    let image = encoder.new_image::<colortype::Gray8>(w, h)?;
    image.write_data(&pixels)?;

    Ok(())
}

fn calc_difference_map(
    past: &[f32],
    present: &[f32],
) -> anyhow::Result<DifferenceMap> {
    if past.len() != present.len() {
        anyhow::bail!(
            "Buffer length mismatch: past={}, present={}",
            past.len(), present.len()
        );
    }

    let size = present.len();
    let mut total_change = 0.0_f32;
    let mut max_decline = 0.0_f32; 
    let mut max_growth = 0.0_f32; 

    for i in 0..size {
        let diff = present[i] - past[i];
        total_change += diff;

        if diff < max_decline { max_decline = diff; }
        if diff > max_growth { max_growth  = diff; }
    }

    Ok(DifferenceMap {
        mean_change: total_change / size as f32,
        max_decline,
        max_growth,
    })
}

fn decode_band(bytes: &bytes::Bytes) -> anyhow::Result<(Vec<u16>, u32, u32)> {
    let cursor = Cursor::new(bytes);
    let mut decoder = Decoder::new(cursor)?;  

    let (width, height) = decoder.dimensions()?; 

    let result = decoder.read_image()?;

    let data = match result {
        DecodingResult::U16(buf) => buf,
        other => anyhow::bail!("Unexpected sample type: {:?}", other),
    };

    Ok((data, width, height))
}

