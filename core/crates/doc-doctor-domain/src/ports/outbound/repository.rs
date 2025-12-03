//! Document Repository Port
//!
//! Abstraction over document storage and retrieval.

use std::path::{Path, PathBuf};
use std::fmt;

/// Repository error
#[derive(Debug, Clone)]
pub struct RepositoryError {
    /// Error kind
    pub kind: RepositoryErrorKind,
    /// Error message
    pub message: String,
    /// Path that caused the error (if applicable)
    pub path: Option<PathBuf>,
}

/// Repository error kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryErrorKind {
    /// File or directory not found
    NotFound,
    /// Permission denied
    PermissionDenied,
    /// Invalid path or pattern
    InvalidPath,
    /// I/O error
    IoError,
    /// Other error
    Other,
}

impl RepositoryError {
    /// Create a new repository error
    pub fn new(kind: RepositoryErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            path: None,
        }
    }

    /// Add path information
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Create a not found error
    pub fn not_found(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            kind: RepositoryErrorKind::NotFound,
            message: format!("File not found: {}", path.display()),
            path: Some(path),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            kind: RepositoryErrorKind::PermissionDenied,
            message: format!("Permission denied: {}", path.display()),
            path: Some(path),
        }
    }
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{} ({})", self.message, path.display())
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for RepositoryError {}

/// Document repository trait
///
/// This is an outbound port for document persistence.
/// Adapters implement this to provide file system, database,
/// or network-based storage.
pub trait DocumentRepository: Send + Sync {
    /// Read document content from path
    ///
    /// # Arguments
    /// * `path` - Path to the document
    ///
    /// # Returns
    /// Document content as string or error
    fn read(&self, path: &Path) -> Result<String, RepositoryError>;

    /// Write document content to path
    ///
    /// # Arguments
    /// * `path` - Path to write to
    /// * `content` - Content to write
    ///
    /// # Returns
    /// Ok on success, error on failure
    fn write(&self, path: &Path, content: &str) -> Result<(), RepositoryError>;

    /// List documents matching a glob pattern
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern (e.g., "**/*.md")
    ///
    /// # Returns
    /// List of matching paths
    fn list(&self, pattern: &str) -> Result<Vec<PathBuf>, RepositoryError>;

    /// Check if a document exists
    ///
    /// # Arguments
    /// * `path` - Path to check
    ///
    /// # Returns
    /// true if exists, false otherwise
    fn exists(&self, path: &Path) -> bool;

    /// Get document metadata (modification time, size, etc.)
    ///
    /// # Arguments
    /// * `path` - Path to the document
    ///
    /// # Returns
    /// Document metadata or error
    fn metadata(&self, path: &Path) -> Result<DocumentMetadata, RepositoryError>;
}

/// Document metadata from the repository
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    /// File size in bytes
    pub size: u64,
    /// Last modification time (Unix timestamp)
    pub modified: Option<i64>,
    /// Creation time (Unix timestamp)
    pub created: Option<i64>,
    /// Whether this is a directory
    pub is_directory: bool,
}
