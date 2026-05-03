//! # sentinel_orchestrator
//!
//! Predicts Sentinel-2 satellite passes for a ground observer and emits
//! [`SatellitePassEvent`]s over an `mpsc` channel for downstream processing.
//!
//! This crate knows about orbital mechanics (via [`predictor`]) and
//! geography (observer position, bbox). It has no knowledge of band fetching,
//! NDVI, or Azure.

mod error;
pub mod config;
mod tle;
pub mod predict;
mod runner;

pub use error::{OrchestratorError, OrchestratorResult};

pub use config::{OrchestratorConfig, BBox};
pub use tle::fetch_tle;
pub use predict::{predict_passes, ms_to_datetime};
pub use runner::predict_loop;