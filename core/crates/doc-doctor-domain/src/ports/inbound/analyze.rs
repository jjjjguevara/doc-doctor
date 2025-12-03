//! Analyze Document Use Case Port
//!
//! Inbound port for document analysis.

use crate::calculations::StateDimensions;
use crate::entities::L1Properties;
use crate::errors::ValidationWarning;

/// Document analysis result
#[derive(Debug, Clone)]
pub struct DocumentAnalysis {
    /// Parsed L1 properties
    pub properties: L1Properties,
    /// Calculated L2 dimensions
    pub dimensions: StateDimensions,
    /// Validation warnings (non-fatal issues)
    pub warnings: Vec<ValidationWarning>,
}

impl DocumentAnalysis {
    /// Create a new analysis result
    pub fn new(properties: L1Properties, dimensions: StateDimensions) -> Self {
        Self {
            properties,
            dimensions,
            warnings: Vec::new(),
        }
    }

    /// Add warnings
    pub fn with_warnings(mut self, warnings: Vec<ValidationWarning>) -> Self {
        self.warnings = warnings;
        self
    }

    /// Check if the document is healthy (health >= 0.7)
    pub fn is_healthy(&self) -> bool {
        self.dimensions.health >= 0.7
    }

    /// Check if the document is useful for its audience
    pub fn is_useful(&self) -> bool {
        self.dimensions.usefulness.is_useful
    }
}

/// Analysis error
#[derive(Debug, Clone)]
pub struct AnalysisError {
    /// Error message
    pub message: String,
    /// Underlying cause (if any)
    pub cause: Option<String>,
}

impl AnalysisError {
    /// Create a new analysis error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
        }
    }

    /// Add cause
    pub fn with_cause(mut self, cause: impl Into<String>) -> Self {
        self.cause = Some(cause.into());
        self
    }
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(cause) = &self.cause {
            write!(f, ": {}", cause)?;
        }
        Ok(())
    }
}

impl std::error::Error for AnalysisError {}

/// Analyze document use case trait
///
/// This is an inbound port: external actors (CLI, MCP, WASM)
/// call this to analyze documents.
pub trait AnalyzeDocument {
    /// Analyze document content
    ///
    /// # Arguments
    /// * `content` - Raw document content
    ///
    /// # Returns
    /// Analysis result or error
    fn analyze(&self, content: &str) -> Result<DocumentAnalysis, AnalysisError>;
}
