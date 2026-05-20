use std::fs::File;
use tiff::encoder::{colortype::Gray32Float, compression::Deflate, TiffEncoder};
use crate::error::{NdviError, NdviResult};

/// Georeferencing parameters written to the `.tfw` world file.
///
/// All values must be in the coordinate system defined by `prj_wkt`.
pub struct GeoRef {
    pub pixel_size_x: f64,
    /// Negative — rows step southward (top-left origin convention).
    pub pixel_size_y: f64,
    pub origin_x: f64,
    pub origin_y: f64,
    pub prj_wkt: &'static str,
}

impl GeoRef {
    /// UTM Zone 10N (EPSG:32610) at 10 m/pixel — Sentinel-2's native CRS for Surrey, BC.
    ///
    /// Origin is the top-left corner of the standard 100 km tile grid, not a pixel centre.
    pub fn utm10n_10m() -> Self {
        Self {
            pixel_size_x: 10.0,
            pixel_size_y: -10.0,
            origin_x: 499_980.0,
            origin_y: 5_500_020.0,
            prj_wkt: r#"PROJCS["WGS 84 / UTM zone 10N",GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Transverse_Mercator"],PARAMETER["latitude_of_origin",0],PARAMETER["central_meridian",-123],PARAMETER["scale_factor",0.9996],PARAMETER["false_easting",500000],PARAMETER["false_northing",0],UNIT["metre",1]]"#,
        }
    }
}

/// Write a single-band Float32 GeoTIFF with Deflate compression.
///
/// Invalid/masked pixels are stored as `NaN`. In QGIS, load with
/// *Singleband pseudocolor* → *RdYlGn* ramp → 2–98% percentile stretch.
pub fn write_f32_tiff(
    ndvi: &[f32],
    width: u32,
    height: u32,
    path: &str,
    georef: &GeoRef,
) -> NdviResult<()> {
    let file = File::create(path)?;
    let mut enc = TiffEncoder::new(file).map_err(tiff_err)?;

    // `write_image` has no compression parameter — use `new_image_with_compression`
    // explicitly, otherwise the tiff crate defaults to uncompressed output.
    let img = enc
        .new_image_with_compression::<Gray32Float, _>(width, height, Deflate::default())
        .map_err(tiff_err)?;

    img.write_data(ndvi).map_err(tiff_err)?;

    write_sidecars(path, georef)
}

fn write_sidecars(tif_path: &str, georef: &GeoRef) -> NdviResult<()> {
    let tfw = format!(
        "{}\n0.0\n0.0\n{}\n{}\n{}\n",
        georef.pixel_size_x, georef.pixel_size_y, georef.origin_x, georef.origin_y,
    );
    std::fs::write(tif_path.replace(".tif", ".tfw"), tfw)?;
    std::fs::write(tif_path.replace(".tif", ".prj"), georef.prj_wkt)?;
    Ok(())
}

/// Wrap a `tiff` crate error into [`NdviError::Io`].
///
/// The `tiff` crate uses its own error type, so this keeps error handling
/// uniform without pulling tiff's error type into the public API.
fn tiff_err(e: impl std::fmt::Display) -> NdviError {
    NdviError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}