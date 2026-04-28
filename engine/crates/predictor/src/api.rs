use wasm_bindgen::prelude::*;
use crate::{
    propagate,
    types::{AzEl, Observer, ScanOptions},
};

#[wasm_bindgen]
pub fn predict_look_angles(
    tle1: &str,
    tle2: &str,
    unix_ms: f64,
    observer: &Observer,
) -> Result<AzEl, JsValue> {
    propagate::look_angles(tle1, tle2, unix_ms, observer)
        .map_err(JsValue::from)
}

#[wasm_bindgen]
pub fn find_passes(
    tle1: &str,
    tle2: &str,
    observer: &Observer,
    options: &ScanOptions,
) -> Result<JsValue, JsValue> {
    let passes = propagate::passes(tle1, tle2, observer, options)
        .map_err(JsValue::from)?;

    serde_wasm_bindgen::to_value(&passes)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
