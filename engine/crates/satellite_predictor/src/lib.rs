mod coords;
mod error;
mod propagate;
pub mod types;

#[cfg(feature = "wasm")]
pub mod api;

#[cfg(feature = "wasm")]
pub use api::{predict_look_angles, find_passes};

pub use types::{AzEl, Observer, ScanOptions};
pub use propagate::{passes, look_angles};