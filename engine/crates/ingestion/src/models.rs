use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SignedResponse {
    pub href: String,
}

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

#[derive(Debug, Clone, Serialize)]
pub struct DifferenceMap {
    pub mean_change: f32,
    pub max_decline: f32,
    pub max_growth: f32,
}
