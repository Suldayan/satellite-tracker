use std::fmt;

#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum TrackerError {
    TleParse(String),
    PhysicsInit(String),
    Propagation(String),
    InvalidInput(String),
}

impl fmt::Display for TrackerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackerError::TleParse(m)     => write!(f, "TLE parse error: {m}"),
            TrackerError::PhysicsInit(m)  => write!(f, "Physics init error: {m}"),
            TrackerError::Propagation(m)  => write!(f, "Propagation error: {m}"),
            TrackerError::InvalidInput(m) => write!(f, "Invalid input: {m}"),
        }
    }
}

#[cfg(feature = "wasm")]
impl From<TrackerError> for JsValue {
    fn from(e: TrackerError) -> Self {
        JsValue::from_str(&e.to_string())
    }
}