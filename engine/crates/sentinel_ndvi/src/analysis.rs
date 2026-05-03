use serde::Serialize;
use crate::error::{NdviError, NdviResult};

/// Summary statistics describing vegetation change between two NDVI snapshots.
#[derive(Debug, Clone, Serialize)]
pub struct DifferenceMap {
    /// Mean per-pixel NDVI change (positive = growth, negative = decline).
    pub mean_change: f32,
    /// Largest single-pixel decline observed (will be ≤ 0.0).
    pub max_decline: f32,
    /// Largest single-pixel growth observed (will be ≥ 0.0).
    pub max_growth: f32,
}

/// Compute a [`DifferenceMap`] between two temporally aligned NDVI grids.
///
/// Both slices must be the same length (same spatial extent and resolution).
///
/// # Errors
///
/// Returns [`NdviError::LengthMismatch`] when slice lengths differ.
pub fn calc_difference_map(past: &[f32], present: &[f32]) -> NdviResult<DifferenceMap> {
    if past.len() != present.len() {
        return Err(NdviError::LengthMismatch {
            past:    past.len(),
            present: present.len(),
        });
    }

    let mut total   = 0.0_f32;
    let mut decline = 0.0_f32;
    let mut growth  = 0.0_f32;

    for (p, q) in past.iter().zip(present.iter()) {
        let diff = q - p;
        total   += diff;
        if diff < decline { decline = diff; }
        if diff > growth  { growth  = diff; }
    }

    Ok(DifferenceMap {
        mean_change: total / past.len() as f32,
        max_decline: decline,
        max_growth:  growth,
    })
}