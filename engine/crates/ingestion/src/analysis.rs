use crate::DifferenceMap;

pub fn calc_difference_map(
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