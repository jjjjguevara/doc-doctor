//! Inbound Ports (Driving)
//!
//! These ports define use cases that external actors can invoke.
//! The application layer implements these traits.

mod analyze;
mod batch;
mod calculate;
mod validate;

pub use analyze::{AnalysisError, AnalyzeDocument, DocumentAnalysis};
pub use batch::{BatchDocumentResult, BatchError, BatchProcess, BatchResult};
pub use calculate::{CalculateDimensions, DefaultCalculator};
pub use validate::{SchemaError, SchemaWarning, ValidateDocument, ValidationError, ValidationResult};
