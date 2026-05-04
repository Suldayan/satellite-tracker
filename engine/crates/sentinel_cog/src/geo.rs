use sentinel_types::BBox;
use crate::parse::{GeoTransform, IfdInfo};

const TYPE_DOUBLE: u16 = 12;

pub fn read_f64(data: &[u8], off: usize, le: bool) -> Option<f64> {
    let s = data.get(off..off + 8)?;
    Some(if le {
        f64::from_le_bytes(s.try_into().unwrap())
    } else {
        f64::from_be_bytes(s.try_into().unwrap())
    })
}

/// Convert WGS84 lat/lon degrees to UTM Zone 10N easting/northing (metres).
/// Uses the standard Transverse Mercator projection formula.
fn latlon_to_utm10n(lat_deg: f64, lon_deg: f64) -> (f64, f64) {
    const A: f64 = 6_378_137.0;
    const F: f64 = 1.0 / 298.257_223_563;
    const E2: f64 = 2.0 * F - F * F;
    const K0: f64 = 0.9996;
    const FALSE_EASTING: f64 = 500_000.0;
    const CENTRAL_MERIDIAN: f64 = -123.0_f64;

    let lat = lat_deg.to_radians();
    let lon = lon_deg.to_radians();
    let lon0 = CENTRAL_MERIDIAN.to_radians();

    let n = A / (1.0 - E2 * lat.sin().powi(2)).sqrt();
    let t = lat.tan().powi(2);
    let c = E2 / (1.0 - E2) * lat.cos().powi(2);
    let a_coef = (lon - lon0) * lat.cos();

    let e2 = E2;
    let e4 = e2 * e2;
    let e6 = e4 * e2;

    let m = A * (
        (1.0 - e2 / 4.0 - 3.0 * e4 / 64.0 - 5.0 * e6 / 256.0) * lat
        - (3.0 * e2 / 8.0 + 3.0 * e4 / 32.0 + 45.0 * e6 / 1024.0) * (2.0 * lat).sin()
        + (15.0 * e4 / 256.0 + 45.0 * e6 / 1024.0) * (4.0 * lat).sin()
        - (35.0 * e6 / 3072.0) * (6.0 * lat).sin()
    );

    let easting = FALSE_EASTING + K0 * n * (
        a_coef
        + (1.0 - t + c) * a_coef.powi(3) / 6.0
        + (5.0 - 18.0 * t + t * t + 72.0 * c - 58.0 * (e2 / (1.0 - e2))) * a_coef.powi(5) / 120.0
    );

    let northing = K0 * (
        m + n * lat.tan() * (
            a_coef.powi(2) / 2.0
            + (5.0 - t + 9.0 * c + 4.0 * c * c) * a_coef.powi(4) / 24.0
            + (61.0 - 58.0 * t + t * t + 600.0 * c - 330.0 * (e2 / (1.0 - e2))) * a_coef.powi(6) / 720.0
        )
    );

    (easting, northing)
}

/// Return only the tile offsets from `info` whose pixel extents intersect `bbox`.
///
/// Requires `info.geo` to be populated — call this after parsing an IFD that
/// contains tags 33550 (PixelScale) and 33922 (ModelTiepoint).
pub fn filter_tiles(info: &IfdInfo, bbox: &BBox) -> Vec<(u64, u64)> {
    let geo = match &info.geo {
        Some(g) => g,
        None => return info.tile_offsets.clone(),
    };

    let (utm_min_x, utm_min_y) = latlon_to_utm10n(bbox.min_lat, bbox.min_lon);
    let (utm_max_x, utm_max_y) = latlon_to_utm10n(bbox.max_lat, bbox.max_lon);

    let tiles_down = (info.img_h + info.tile_h - 1) / info.tile_h;

    info.tile_offsets
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            let tile_col = (*i as u32) % info.tiles_across;
            let tile_row = (*i as u32) / info.tiles_across;

            let tile_min_x = geo.origin_x + (tile_col * info.tile_w) as f64 * geo.pixel_x;
            let tile_max_x = tile_min_x + info.tile_w as f64 * geo.pixel_x;

            // pixel_y is negative (north-up), so min_y is the bottom of the tile
            let tile_max_y = geo.origin_y + (tile_row * info.tile_h) as f64 * geo.pixel_y;
            let tile_min_y = tile_max_y + info.tile_h as f64 * geo.pixel_y;

            let _ = tiles_down;

            tile_max_x > utm_min_x
                && tile_min_x < utm_max_x
                && tile_max_y > utm_min_y
                && tile_min_y < utm_max_y
        })
        .map(|(_, &offsets)| offsets)
        .collect()
}