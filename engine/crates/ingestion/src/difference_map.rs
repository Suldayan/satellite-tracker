use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DifferenceMap {
    pub mean_change: f32,
    pub max_decline: f32,
    pub max_growth: f32,
}