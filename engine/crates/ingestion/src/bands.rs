use predictor::SatellitePassEvent;
use crate::fetch_imagery;
use std::io::Cursor;
use tiff::decoder::{Decoder, DecodingResult};
use crate::{compute_ndvi, ndvi_to_geotiff};
use std::time::Duration;

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

pub fn decode_band(bytes: &bytes::Bytes) -> anyhow::Result<(Vec<u16>, u32, u32)> {
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