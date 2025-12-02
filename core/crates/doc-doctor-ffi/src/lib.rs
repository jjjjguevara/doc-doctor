//! Doc Doctor FFI Bindings
//!
//! Provides WASM and Node.js bindings for the core library.

mod wasm;
mod napi;

// Re-export for convenience
pub use wasm::*;
