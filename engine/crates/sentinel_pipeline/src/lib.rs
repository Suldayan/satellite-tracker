//! # sentinel_pipeline
//!
//! End-to-end orchestration: STAC query → band fetch → NDVI compute → GeoTIFF.
//!
//! This crate is the glue layer. It knows about satellites and scheduling;
//! the actual TIFF parsing lives in [`sentinel_cog`] and the compute in
//! [`sentinel_ndvi`].

mod error;
pub mod stac;
pub mod pass;

pub use error::{PipelineError, PipelineResult};
pub use pass::{ingest_pass, handle_pass};
pub use stac::SceneUrls;