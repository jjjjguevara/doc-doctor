//! Doc Doctor Core Library
//!
//! Implements the J-Editorial Framework specifications with deterministic
//! L1/L2 calculations for document quality management.
//!
//! # Architecture
//!
//! - **L1 (Intrinsic)**: Properties stored in frontmatter (refinement, audience, stubs)
//! - **L2 (Extrinsic)**: Calculated dimensions (health, usefulness, vector physics)
//! - **L3 (Operational)**: Rule engine for automated workflows (future)
//!
//! # Example
//!
//! ```rust
//! use doc_doctor_core::{parse_document, dimensions};
//!
//! let content = r#"---
//! title: My Document
//! refinement: 0.75
//! audience: internal
//! stubs:
//!   - type: link
//!     description: "Citation needed"
//! ---
//! # Content here
//! "#;
//!
//! let doc = parse_document(content).unwrap();
//! let health = dimensions::calculate_health(doc.refinement.value(), &doc.stubs);
//! ```

pub mod error;
pub mod parser;
pub mod types;
pub mod dimensions;
pub mod stubs;

// Re-export commonly used types
pub use error::{DocDoctorError, Result, SourcePosition};
pub use types::{L1Properties, Audience, Refinement, Origin, Form};
pub use stubs::{Stub, StubForm, StubType, Priority};

/// Parse a markdown document and extract L1 properties from frontmatter.
///
/// # Arguments
/// * `content` - Full markdown document content including frontmatter
///
/// # Returns
/// * `Result<L1Properties>` - Parsed properties or error with position info
pub fn parse_document(content: &str) -> Result<L1Properties> {
    parser::parse_document(content)
}

/// Validate frontmatter YAML against J-Editorial schema.
///
/// # Arguments
/// * `content` - Full markdown document content including frontmatter
/// * `strict` - If true, reject unknown fields
///
/// # Returns
/// * `Result<Vec<ValidationWarning>>` - Validation result with warnings
pub fn validate_frontmatter(content: &str, strict: bool) -> Result<Vec<error::ValidationWarning>> {
    parser::validate_frontmatter(content, strict)
}
