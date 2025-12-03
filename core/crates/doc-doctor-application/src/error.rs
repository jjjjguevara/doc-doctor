//! Application layer errors
//!
//! Errors that can occur during use case execution.

use doc_doctor_domain::{DomainError, ParseError};
use thiserror::Error;

/// Application layer error
#[derive(Error, Debug)]
pub enum ApplicationError {
    /// Document parsing failed
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    /// Domain validation error
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    /// Batch processing error
    #[error("Batch error: {message}")]
    Batch {
        message: String,
        /// Number of documents that failed
        failed_count: usize,
    },

    /// File system error
    #[error("File error: {0}")]
    FileSystem(String),

    /// Invalid glob pattern
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    /// Repository error
    #[error("Repository error: {0}")]
    Repository(String),
}

impl ApplicationError {
    /// Create a batch error
    pub fn batch(message: impl Into<String>, failed_count: usize) -> Self {
        Self::Batch {
            message: message.into(),
            failed_count,
        }
    }

    /// Create a file system error
    pub fn file_system(message: impl Into<String>) -> Self {
        Self::FileSystem(message.into())
    }

    /// Create an invalid pattern error
    pub fn invalid_pattern(message: impl Into<String>) -> Self {
        Self::InvalidPattern(message.into())
    }

    /// Create a repository error
    pub fn repository(message: impl Into<String>) -> Self {
        Self::Repository(message.into())
    }
}

/// Result type for application operations
pub type ApplicationResult<T> = Result<T, ApplicationError>;
