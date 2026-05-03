//! # sentinel_ndvi
//!
//! Pure-compute NDVI processing — no HTTP, no scheduling.
//!
//! Depends on [`sentinel_cog`] for the [`Raster`] type but has no knowledge
//! of how or where bands were fetched.
//!
//! ## Example
//!
//! ```no_run
//! use sentinel_ndvi::{compute_ndvi, write_f32_tiff, GeoRef, NdviError};
//! use sentinel_cog::Raster;
//!
//! # let b04 = Raster { pixels: vec![], width: 0, height: 0 };
//! # let b08 = Raster { pixels: vec![], width: 0, height: 0 };
//! let (ndvi, w, h) = compute_ndvi(&b04, &b08)?;
//! write_f32_tiff(&ndvi, w, h, "ndvi.tif", &GeoRef::utm10n_10m())?;
//! # Ok::<(), NdviError>(())
//! ```

mod error;
pub mod ndvi;
pub mod analysis;
mod geotiff;

pub use error::{NdviError, NdviResult};
pub use ndvi::{compute_ndvi, compute_ndvi_raw};
pub use analysis::{calc_difference_map, DifferenceMap};
pub use geotiff::{write_rgb_geotiff, write_f32_tiff, GeoRef};