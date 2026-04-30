use serde::Deserialize;
use predictor::SatellitePassEvent;

#[derive(Debug, Clone, Deserialize)]
pub struct StacResponse {
    pub features: Vec<StacFeature>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StacFeature {
    pub assets: StacAssets,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StacAssets {
    pub b04: StacAsset,
    pub b08: StacAsset,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StacAsset {
    pub href: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignedResponse {
    pub href: String,
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

pub fn fetch_imagery(event: &SatellitePassEvent) 
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