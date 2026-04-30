pub mod stac;
pub mod bands;
pub mod analysis;
pub mod ndvi;
pub mod difference_map;

pub use ndvi::{compute_ndvi, ndvi_to_geotiff};
pub use difference_map::DifferenceMap;
pub use analysis::calc_difference_map;
pub use bands::{decode_band, handle_pass};
pub use stac::{StacResponse, StacFeature, StacAssets, StacAsset, fetch_imagery};
