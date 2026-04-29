use ingestion::calc_difference_map;

#[cfg(test)]
pub fn compute_ndvi_raw(b04: &[u16], b08: &[u16]) -> Vec<f32> {
    b04.iter().zip(b08.iter()).map(|(&r, &n)| {
        let red = r as f32;
        let nir = n as f32;
        if (nir + red) == 0.0 { 0.0 } else { (nir - red) / (nir + red) }
    }).collect()
}

#[test]
fn ndvi_of_pure_vegetation_is_near_one() {
    // B04 (red) very low, B08 (NIR) very high = healthy vegetation
    let b04 = vec![500u16;  100];
    let b08 = vec![4000u16; 100];

    let ndvi = compute_ndvi_raw(&b04, &b08);
    let mean = ndvi.iter().sum::<f32>() / ndvi.len() as f32;

    assert!(mean > 0.7, "Expected high NDVI for vegetation, got {mean:.3}");
}

#[test]
fn ndvi_of_bare_soil_is_near_zero() {
    // Both bands roughly equal = bare soil
    let b04 = vec![2000u16; 100];
    let b08 = vec![2200u16; 100];

    let ndvi = compute_ndvi_raw(&b04, &b08);
    let mean = ndvi.iter().sum::<f32>() / ndvi.len() as f32;

    assert!(mean < 0.1, "Expected low NDVI for bare soil, got {mean:.3}");
}

#[test]
fn difference_map_detects_vegetation_decline() {
    // Past: healthy vegetation
    let past = vec![0.6_f32; 100];
    // Present: stressed vegetation
    let present = vec![0.3_f32; 100];

    let diff = calc_difference_map(&past, &present).unwrap();

    assert!(diff.mean_change < 0.0,  "Expected negative mean change");
    assert!(diff.max_decline < 0.0,  "Expected a decline");
    assert_eq!(diff.max_growth, 0.0, "Expected no growth");
}

#[test]
fn difference_map_errors_on_mismatched_lengths() {
    let past = vec![0.5_f32; 100];
    let present = vec![0.5_f32; 50]; 

    assert!(calc_difference_map(&past, &present).is_err());
}