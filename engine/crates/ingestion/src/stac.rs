use serde::Deserialize;
use predictor::SatellitePassEvent;

const URL: &str = "https://planetarycomputer.microsoft.com/api/sas/v1/sign";
const STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1/search";

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
    #[serde(rename = "B04")]
    pub b04: StacAsset,
    #[serde(rename = "B08")]
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
        .get(URL)
        .query(&[("href", href)])
        .send()?
        .json()?;

    Ok(resp.href)
}

pub fn fetch_imagery(event: &SatellitePassEvent)
    -> Result<Option<(String, String)>, Box<dyn std::error::Error>>
{
    const DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";
    let client = reqwest::blocking::Client::new();

    let body = serde_json::json!({
        "collections": ["sentinel-2-l2a"],
        "bbox": [event.min_lon, event.min_lat, event.max_lon, event.max_lat],
        "datetime": format!("{}/{}",
            event.pass_start.format(DATE_FORMAT),
            event.pass_end.format(DATE_FORMAT)),
        "query": { "eo:cloud_cover": { "lt": 20 } }
    });

    let stac: StacResponse = client
        .post(STAC_URL)
        .json(&body)
        .send()?
        .json()?;

    let Some(scene) = stac.features.into_iter().next() else {
        println!("No scenes found for this pass (cloud cover or timing)");
        return Ok(None);
    };

    Ok(Some((
        sign_url(&client, &scene.assets.b04.href)?,
        sign_url(&client, &scene.assets.b08.href)?,
    )))
}