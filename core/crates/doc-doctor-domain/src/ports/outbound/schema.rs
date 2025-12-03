//! Schema Provider Port
//!
//! Provides JSON Schema definitions for validation.

/// Schema provider trait
///
/// Provides JSON Schema definitions for frontmatter and stub validation.
/// This allows different implementations to provide schema files from
/// various sources (embedded, file system, network).
pub trait SchemaProvider: Send + Sync {
    /// Get JSON Schema for frontmatter validation
    ///
    /// # Returns
    /// JSON Schema as a string
    fn frontmatter_schema(&self) -> &str;

    /// Get JSON Schema for stubs array validation
    ///
    /// # Returns
    /// JSON Schema as a string
    fn stubs_schema(&self) -> &str;

    /// Get schema version
    ///
    /// # Returns
    /// Schema version string (e.g., "1.0.0")
    fn version(&self) -> &str;
}

/// Default embedded schema provider
pub struct EmbeddedSchemaProvider;

impl SchemaProvider for EmbeddedSchemaProvider {
    fn frontmatter_schema(&self) -> &str {
        include_str!("../../schemas/frontmatter.json")
    }

    fn stubs_schema(&self) -> &str {
        include_str!("../../schemas/stubs.json")
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}
