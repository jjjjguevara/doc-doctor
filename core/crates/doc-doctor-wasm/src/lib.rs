//! Doc-Doctor WASM Bindings
//!
//! WASM bindings for the Doc-Doctor library, enabling use from JavaScript/TypeScript.
//!
//! # Architecture
//!
//! This crate wires together:
//! - `doc-doctor-domain`: Pure domain logic (entities, calculations)
//! - `doc-doctor-application`: Use case implementations
//! - `doc-doctor-parser-yaml`: YAML frontmatter parser
//!
//! # Usage (JavaScript/TypeScript)
//!
//! ```javascript
//! import init, { DocDoctor } from '@doc-doctor/wasm';
//!
//! await init();
//! const dd = new DocDoctor();
//!
//! // Parse a document
//! const result = dd.parseDocument(content);
//! const props = JSON.parse(result);
//!
//! // Calculate health
//! const health = dd.calculateHealth(0.8, '[]');
//! ```

mod bindings;
mod types;

pub use bindings::DocDoctor;

use wasm_bindgen::prelude::*;

/// Initialize the WASM module
///
/// Sets up panic hook for better error messages in console.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
