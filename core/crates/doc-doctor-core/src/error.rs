//! Error types with source position tracking
//!
//! All errors include line:column positions for precise error reporting.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type alias for doc-doctor operations
pub type Result<T> = std::result::Result<T, DocDoctorError>;

/// Source position in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePosition {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset from start (0-indexed)
    pub offset: usize,
}

impl SourcePosition {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }

    /// Create position from byte offset and content
    pub fn from_offset(content: &str, offset: usize) -> Self {
        let mut line = 1;
        let mut column = 1;
        let mut current_offset = 0;

        for ch in content.chars() {
            if current_offset >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
            current_offset += ch.len_utf8();
        }

        Self { line, column, offset }
    }
}

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Validation warning (non-fatal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub message: String,
    pub field: Option<String>,
    pub position: Option<SourcePosition>,
    pub suggestion: Option<String>,
}

/// Main error type for doc-doctor operations
#[derive(Error, Debug)]
pub enum DocDoctorError {
    /// YAML parsing error
    #[error("YAML parse error at {}: {message}", position.map(|p| p.to_string()).unwrap_or_else(|| "unknown".to_string()))]
    YamlParse {
        message: String,
        position: Option<SourcePosition>,
        source_snippet: Option<String>,
    },

    /// No frontmatter found
    #[error("No frontmatter found in document")]
    NoFrontmatter,

    /// Invalid frontmatter delimiters
    #[error("Invalid frontmatter: {message}")]
    InvalidFrontmatter {
        message: String,
        position: Option<SourcePosition>,
    },

    /// Field validation error
    #[error("Validation error for field '{field}': {message}")]
    Validation {
        message: String,
        field: String,
        position: Option<SourcePosition>,
        expected: Option<String>,
        actual: Option<String>,
    },

    /// Refinement value out of range
    #[error("Refinement value {value} is out of range (must be 0.0-1.0)")]
    RefinementOutOfRange {
        value: f64,
        position: Option<SourcePosition>,
    },

    /// Unknown audience value
    #[error("Unknown audience '{audience}' (expected: personal, internal, trusted, public)")]
    UnknownAudience {
        audience: String,
        position: Option<SourcePosition>,
    },

    /// Unknown stub type
    #[error("Unknown stub type '{stub_type}'")]
    UnknownStubType {
        stub_type: String,
        known_types: Vec<String>,
        position: Option<SourcePosition>,
    },

    /// Invalid stub form
    #[error("Invalid stub_form '{form}' (expected: transient, persistent, blocking, structural)")]
    InvalidStubForm {
        form: String,
        position: Option<SourcePosition>,
    },

    /// Invalid priority
    #[error("Invalid priority '{priority}' (expected: low, medium, high, critical)")]
    InvalidPriority {
        priority: String,
        position: Option<SourcePosition>,
    },

    /// Missing required field
    #[error("Missing required field '{field}'")]
    MissingField {
        field: String,
        position: Option<SourcePosition>,
    },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl DocDoctorError {
    /// Get the source position if available
    pub fn position(&self) -> Option<SourcePosition> {
        match self {
            Self::YamlParse { position, .. } => *position,
            Self::InvalidFrontmatter { position, .. } => *position,
            Self::Validation { position, .. } => *position,
            Self::RefinementOutOfRange { position, .. } => *position,
            Self::UnknownAudience { position, .. } => *position,
            Self::UnknownStubType { position, .. } => *position,
            Self::InvalidStubForm { position, .. } => *position,
            Self::InvalidPriority { position, .. } => *position,
            Self::MissingField { position, .. } => *position,
            _ => None,
        }
    }

    /// Create a YAML parse error with position
    pub fn yaml_parse(message: impl Into<String>, position: Option<SourcePosition>, snippet: Option<String>) -> Self {
        Self::YamlParse {
            message: message.into(),
            position,
            source_snippet: snippet,
        }
    }

    /// Create a validation error
    pub fn validation(
        message: impl Into<String>,
        field: impl Into<String>,
        position: Option<SourcePosition>,
    ) -> Self {
        Self::Validation {
            message: message.into(),
            field: field.into(),
            position,
            expected: None,
            actual: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_from_offset() {
        let content = "line1\nline2\nline3";

        // Start of file
        let pos = SourcePosition::from_offset(content, 0);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);

        // Start of line 2
        let pos = SourcePosition::from_offset(content, 6);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);

        // Middle of line 2
        let pos = SourcePosition::from_offset(content, 8);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 3);
    }

    #[test]
    fn test_position_display() {
        let pos = SourcePosition::new(10, 5, 100);
        assert_eq!(pos.to_string(), "10:5");
    }
}
