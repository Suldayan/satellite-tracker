use std::fmt;
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
            TrackerError::TleParse(msg)     => write!(f, "TLE parse error: {msg}"),
            TrackerError::PhysicsInit(msg)  => write!(f, "Physics init error: {msg}"),
            TrackerError::Propagation(msg)  => write!(f, "Propagation error: {msg}"),
            TrackerError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

impl From<TrackerError> for JsValue {
    fn from(e: TrackerError) -> Self {
        JsValue::from_str(&e.to_string())
    }
}