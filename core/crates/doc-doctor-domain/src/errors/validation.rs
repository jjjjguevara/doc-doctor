//! Validation error details
//!
//! Additional validation-related types for domain errors.

use serde::{Deserialize, Serialize};

/// Validation warning (non-fatal issue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,

    /// Field that triggered the warning
    pub field: Option<String>,

    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ValidationWarning {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
            suggestion: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}
