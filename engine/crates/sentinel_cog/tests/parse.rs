use sentinel_cog::{CogError};

// ── Byte-order detection ──────────────────────────────────────────────────────

// We test the internal parser directly by constructing minimal valid TIFF
// byte sequences. This avoids any HTTP dependency in unit tests.

/// Build a minimal little-endian TIFF header pointing to an IFD at offset 8.
fn le_header() -> Vec<u8> {
    let mut h = vec![
        0x49, 0x49, // "II" = little-endian
        0x2A, 0x00, // magic 42
        0x08, 0x00, 0x00, 0x00, // IFD offset = 8
    ];
    // Pad to 64 bytes so offset reads don't go out of bounds in tests
    h.resize(64, 0);
    h
}

/// Build a minimal big-endian TIFF header pointing to an IFD at offset 8.
fn be_header() -> Vec<u8> {
    let mut h = vec![
        0x4D, 0x4D, // "MM" = big-endian
        0x00, 0x2A, // magic 42
        0x00, 0x00, 0x00, 0x08, // IFD offset = 8
    ];
    h.resize(64, 0);
    h
}

#[test]
fn detects_little_endian_marker() {
    // "II" at bytes 0-1 means little-endian
    // We verify this by checking parse_subifds doesn't return an error
    // for a well-formed LE header — byte order detection is its first step.
    let header = le_header();
    let result = sentinel_cog::parse_subifds(&header);
    // Should not error on byte-order — may error on IFD contents since
    // our minimal header has no real IFD entries, but that's fine here.
    assert!(
        !matches!(result, Err(CogError::InvalidHeader(_))),
        "Should not fail byte-order detection for II header"
    );
}

#[test]
fn detects_big_endian_marker() {
    let header = be_header();
    let result = sentinel_cog::parse_subifds(&header);
    assert!(
        !matches!(result, Err(CogError::InvalidHeader(_))),
        "Should not fail byte-order detection for MM header"
    );
}

#[test]
fn rejects_invalid_byte_order_marker() {
    let mut header = le_header();
    header[0] = 0x00; // corrupt the marker
    header[1] = 0x00;

    let err = sentinel_cog::parse_subifds(&header).unwrap_err();
    assert!(
        matches!(err, CogError::InvalidHeader(_)),
        "Expected InvalidHeader, got {err}"
    );
}

// ── Tile geometry ─────────────────────────────────────────────────────────────

#[test]
fn tiles_across_calculation_is_correct() {
    // tiles_across = ceil(img_w / tile_w)
    // We test the formula directly since IfdInfo exposes tiles_across
    // and we can construct it via fetch_overview in integration tests.
    // For unit testing the math: 10980 / 256 = 42.89... → 43 tiles across
    let img_w:  u32 = 10_980;
    let tile_w: u32 = 256;
    let tiles_across = (img_w + tile_w - 1) / tile_w;
    assert_eq!(tiles_across, 43);
}

#[test]
fn tiles_across_exact_multiple() {
    // When img_w is an exact multiple of tile_w, no padding tile needed
    let img_w:  u32 = 1024;
    let tile_w: u32 = 256;
    let tiles_across = (img_w + tile_w - 1) / tile_w;
    assert_eq!(tiles_across, 4);
}

#[test]
fn tiles_across_single_tile() {
    let img_w:  u32 = 100;
    let tile_w: u32 = 256;
    let tiles_across = (img_w + tile_w - 1) / tile_w;
    assert_eq!(tiles_across, 1);
}

// ── Pixel stitching ───────────────────────────────────────────────────────────

#[test]
fn pixel_index_formula_is_correct() {
    // dst = y * img_w + x  — the formula used in decode_tiles
    // Verify it produces correct linear indices for a 4x4 image
    let img_w: u32 = 4;
    let expected: Vec<usize> = (0..16).collect();
    let mut indices = Vec::new();
    for y in 0..4u32 {
        for x in 0..4u32 {
            indices.push((y * img_w + x) as usize);
        }
    }
    assert_eq!(indices, expected);
}

// ── End-to-end (network required) ────────────────────────────────────────────

/// Fetch a real Sentinel-2 overview and verify its dimensions are plausible.
/// Run with `cargo test -- --ignored`.
#[test]
#[ignore]
fn fetch_real_sentinel2_overview() {
    // Sentinel-2A B04 band — Surrey, BC tile T10UEV
    // This URL may expire; replace with a fresh signed URL if needed.
    let url = std::env::var("SENTINEL_B04_URL")
        .expect("Set SENTINEL_B04_URL to a signed Sentinel-2 COG URL");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap();

    let raster = sentinel_cog::fetch_overview(&client, &url, 3).unwrap();

    // Level 3 overview of a 10980x10980 tile should be roughly 1372x1372
    assert!(raster.width  > 100, "Width {} seems too small", raster.width);
    assert!(raster.height > 100, "Height {} seems too small", raster.height);
    assert_eq!(
        raster.pixels.len(),
        (raster.width * raster.height) as usize,
        "Pixel buffer length doesn't match dimensions"
    );

    println!("Raster: {}x{} ({} pixels)", raster.width, raster.height, raster.pixels.len());
}