//! Batch Process Use Case Port
//!
//! Inbound port for batch document processing.

use crate::calculations::StateDimensions;
use crate::entities::L1Properties;
use std::path::PathBuf;

/// Single document result in batch processing
#[derive(Debug, Clone)]
pub struct BatchDocumentResult {
    /// Document path
    pub path: PathBuf,
    /// Parsed properties (if successful)
    pub properties: Option<L1Properties>,
    /// Calculated dimensions (if successful)
    pub dimensions: Option<StateDimensions>,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl BatchDocumentResult {
    /// Create a successful result
    pub fn success(path: PathBuf, properties: L1Properties, dimensions: StateDimensions) -> Self {
        Self {
            path,
            properties: Some(properties),
            dimensions: Some(dimensions),
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(path: PathBuf, error: impl Into<String>) -> Self {
        Self {
            path,
            properties: None,
            dimensions: None,
            error: Some(error.into()),
        }
    }

    /// Check if this result is successful
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
}

/// Batch processing result
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Individual document results
    pub documents: Vec<BatchDocumentResult>,
    /// Total documents processed
    pub total: usize,
    /// Successful documents
    pub succeeded: usize,
    /// Failed documents
    pub failed: usize,
}

impl BatchResult {
    /// Create a new batch result
    pub fn new(documents: Vec<BatchDocumentResult>) -> Self {
        let total = documents.len();
        let succeeded = documents.iter().filter(|d| d.is_success()).count();
        let failed = total - succeeded;

        Self {
            documents,
            total,
            succeeded,
            failed,
        }
    }

    /// Get successful results
    pub fn successes(&self) -> impl Iterator<Item = &BatchDocumentResult> {
        self.documents.iter().filter(|d| d.is_success())
    }

    /// Get failed results
    pub fn failures(&self) -> impl Iterator<Item = &BatchDocumentResult> {
        self.documents.iter().filter(|d| !d.is_success())
    }

    /// Calculate aggregate health score
    pub fn average_health(&self) -> Option<f64> {
        let healths: Vec<f64> = self.documents
            .iter()
            .filter_map(|d| d.dimensions.as_ref())
            .map(|dims| dims.health)
            .collect();

        if healths.is_empty() {
            None
        } else {
            Some(healths.iter().sum::<f64>() / healths.len() as f64)
        }
    }
}

/// Batch processing error
#[derive(Debug, Clone)]
pub struct BatchError {
    /// Error message
    pub message: String,
}

impl BatchError {
    /// Create a new batch error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BatchError {}

/// Batch process use case trait
///
/// This is an inbound port for processing multiple documents
/// matching a glob pattern.
pub trait BatchProcess {
    /// Process documents matching a pattern
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern (e.g., "**/*.md")
    ///
    /// # Returns
    /// Batch result with individual document results
    fn process(&self, pattern: &str) -> Result<BatchResult, BatchError>;
}
