//! WASM bindings for Doc Doctor
//!
//! These bindings allow the core library to be used from JavaScript/TypeScript.

use wasm_bindgen::prelude::*;
use doc_doctor_core::{parse_document, dimensions};

/// Main entry point for WASM bindings
#[wasm_bindgen]
pub struct DocDoctor;

#[wasm_bindgen]
impl DocDoctor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    /// Parse a markdown document and return L1 properties as JSON
    #[wasm_bindgen(js_name = parseDocument)]
    pub fn parse_document(&self, content: &str) -> Result<String, JsValue> {
        parse_document(content)
            .map(|props| serde_json::to_string(&props).unwrap())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Calculate health score
    #[wasm_bindgen(js_name = calculateHealth)]
    pub fn calculate_health(&self, refinement: f64, stubs_json: &str) -> Result<f64, JsValue> {
        let stubs: Vec<doc_doctor_core::Stub> = serde_json::from_str(stubs_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(dimensions::calculate_health(refinement, &stubs))
    }

    /// Calculate usefulness margin
    #[wasm_bindgen(js_name = calculateUsefulness)]
    pub fn calculate_usefulness(&self, refinement: f64, audience: &str) -> Result<f64, JsValue> {
        let audience: doc_doctor_core::Audience = audience.parse()
            .map_err(|e: doc_doctor_core::DocDoctorError| JsValue::from_str(&e.to_string()))?;
        let usefulness = dimensions::calculate_usefulness(refinement, audience);
        Ok(usefulness.margin)
    }

    /// Get version string
    #[wasm_bindgen(js_name = version)]
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

impl Default for DocDoctor {
    fn default() -> Self {
        Self::new()
    }
}
