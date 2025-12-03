//! Validate Document Use Case Port
//!
//! Inbound port for document validation.

use crate::ports::outbound::SourcePosition;

/// Schema validation error
#[derive(Debug, Clone)]
pub struct SchemaError {
    /// Error message
    pub message: String,
    /// JSON path to the error (e.g., "/stubs/0/type")
    pub path: Option<String>,
    /// Position in source document
    pub position: Option<SourcePosition>,
}

impl SchemaError {
    /// Create a new schema error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
            position: None,
        }
    }

    /// Add JSON path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Add position
    pub fn with_position(mut self, position: SourcePosition) -> Self {
        self.position = Some(position);
        self
    }
}

/// Schema validation warning
#[derive(Debug, Clone)]
pub struct SchemaWarning {
    /// Warning message
    pub message: String,
    /// JSON path
    pub path: Option<String>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl SchemaWarning {
    /// Create a new schema warning
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
            suggestion: None,
        }
    }

    /// Add JSON path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the document is valid
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<SchemaError>,
    /// Validation warnings
    pub warnings: Vec<SchemaWarning>,
}

impl ValidationResult {
    /// Create a valid result
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create an invalid result
    pub fn invalid(errors: Vec<SchemaError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add warnings
    pub fn with_warnings(mut self, warnings: Vec<SchemaWarning>) -> Self {
        self.warnings = warnings;
        self
    }
}

/// Validation error (use case level)
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error message
    pub message: String,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validate document use case trait
///
/// This is an inbound port for document validation against
/// J-Editorial schema.
pub trait ValidateDocument {
    /// Validate document content
    ///
    /// # Arguments
    /// * `content` - Raw document content
    /// * `strict` - If true, reject unknown fields
    ///
    /// # Returns
    /// Validation result or error
    fn validate(&self, content: &str, strict: bool) -> Result<ValidationResult, ValidationError>;
}
