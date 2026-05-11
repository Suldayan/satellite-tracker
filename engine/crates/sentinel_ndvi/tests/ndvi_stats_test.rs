#[cfg(test)]
mod tests {
    use sentinel_ndvi::compute_stats;

    #[test]
    fn stats_returns_none_for_all_nan() {
        let ndvi = vec![f32::NAN; 100];
        assert!(compute_stats(&ndvi).is_none());
    }

    #[test]
    fn stats_skips_nan_pixels() {
        let mut ndvi = vec![0.5_f32; 100];
        ndvi[0] = f32::NAN;
        ndvi[1] = f32::NAN;

        let stats = compute_stats(&ndvi).unwrap();
        assert_eq!(stats.valid_pixels, 98);
        assert!((stats.mean_ndvi - 0.5).abs() < 1e-5);
    }

    #[test]
    fn stats_mean_is_correct() {
        let ndvi = vec![0.2_f32, 0.4, 0.6, 0.8];
        let stats = compute_stats(&ndvi).unwrap();
        assert!((stats.mean_ndvi - 0.5).abs() < 1e-5);
    }

    #[test]
    fn stats_max_and_min_are_correct() {
        let ndvi = vec![0.1_f32, 0.5, 0.9, -0.3];
        let stats = compute_stats(&ndvi).unwrap();
        assert!((stats.max_ndvi - 0.9).abs() < 1e-5);
        assert!((stats.min_ndvi - (-0.3)).abs() < 1e-5);
    }

    #[test]
    fn stats_single_valid_pixel() {
        let mut ndvi = vec![f32::NAN; 99];
        ndvi.push(0.7);
        let stats = compute_stats(&ndvi).unwrap();
        assert_eq!(stats.valid_pixels, 1);
        assert!((stats.mean_ndvi - 0.7).abs() < 1e-5);
    }

    #[test]
    fn stats_all_valid_pixels_counted() {
        let ndvi = vec![0.4_f32; 1000];
        let stats = compute_stats(&ndvi).unwrap();
        assert_eq!(stats.valid_pixels, 1000);
    }
}