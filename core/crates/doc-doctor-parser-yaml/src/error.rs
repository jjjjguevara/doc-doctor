//! Parser error types
//!
//! Rich error types with position information for diagnostics.

use doc_doctor_domain::SourcePosition;
use thiserror::Error;

/// YAML parse error kind
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YamlParseErrorKind {
    /// No frontmatter found in document
    NoFrontmatter,

    /// Invalid frontmatter delimiters
    InvalidDelimiters,

    /// YAML syntax error
    YamlSyntax,

    /// Type mismatch (e.g., expected number, got string)
    TypeMismatch,

    /// Unknown field (in strict mode)
    UnknownField,

    /// Missing required field
    MissingField,

    /// Value out of range
    OutOfRange,

    /// Invalid enum value
    InvalidEnumValue,
}

/// YAML parsing error with position information
#[derive(Error, Debug, Clone)]
#[error("{message}")]
pub struct YamlParseError {
    /// Error kind
    pub kind: YamlParseErrorKind,

    /// Human-readable error message
    pub message: String,

    /// Position in source document
    pub position: Option<SourcePosition>,

    /// Source snippet around the error
    pub snippet: Option<String>,

    /// Field that caused the error
    pub field: Option<String>,

    /// Suggestion for fixing the error
    pub suggestion: Option<String>,
}

impl YamlParseError {
    /// Create a new error
    pub fn new(kind: YamlParseErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            position: None,
            snippet: None,
            field: None,
            suggestion: None,
        }
    }

    /// Create a "no frontmatter" error
    pub fn no_frontmatter() -> Self {
        Self::new(
            YamlParseErrorKind::NoFrontmatter,
            "No frontmatter found in document. Expected document to start with '---'",
        )
        .with_suggestion("Add YAML frontmatter at the start: ---\\ntitle: My Document\\n---")
    }

    /// Create an "invalid delimiters" error
    pub fn invalid_delimiters(message: impl Into<String>) -> Self {
        Self::new(YamlParseErrorKind::InvalidDelimiters, message)
    }

    /// Create a YAML syntax error
    pub fn yaml_syntax(message: impl Into<String>) -> Self {
        Self::new(YamlParseErrorKind::YamlSyntax, message)
    }

    /// Create a type mismatch error
    pub fn type_mismatch(field: impl Into<String>, expected: &str, actual: &str) -> Self {
        let field = field.into();
        Self::new(
            YamlParseErrorKind::TypeMismatch,
            format!("Type mismatch for field '{}': expected {}, got {}", field, expected, actual),
        )
        .with_field(field)
    }

    /// Create an unknown field error
    pub fn unknown_field(field: impl Into<String>) -> Self {
        let field = field.into();
        Self::new(
            YamlParseErrorKind::UnknownField,
            format!("Unknown field '{}'", field),
        )
        .with_field(field)
    }

    /// Create a missing field error
    pub fn missing_field(field: impl Into<String>) -> Self {
        let field = field.into();
        Self::new(
            YamlParseErrorKind::MissingField,
            format!("Missing required field '{}'", field),
        )
        .with_field(field)
    }

    /// Create an out of range error
    pub fn out_of_range(field: impl Into<String>, value: f64, min: f64, max: f64) -> Self {
        let field = field.into();
        Self::new(
            YamlParseErrorKind::OutOfRange,
            format!(
                "Value {} for field '{}' is out of range (must be {}-{})",
                value, field, min, max
            ),
        )
        .with_field(field)
    }

    /// Create an invalid enum value error
    pub fn invalid_enum(field: impl Into<String>, value: &str, valid: &[&str]) -> Self {
        let field = field.into();
        Self::new(
            YamlParseErrorKind::InvalidEnumValue,
            format!(
                "Invalid value '{}' for field '{}'. Valid values: {}",
                value,
                field,
                valid.join(", ")
            ),
        )
        .with_field(field)
    }

    /// Add position information
    pub fn with_position(mut self, position: SourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Add source snippet
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Add field name
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl From<YamlParseError> for doc_doctor_domain::ParseError {
    fn from(err: YamlParseError) -> Self {
        let mut parse_err = doc_doctor_domain::ParseError::new(err.message);

        if let Some(pos) = err.position {
            parse_err = parse_err.with_position(pos);
        }

        if let Some(snippet) = err.snippet {
            parse_err = parse_err.with_snippet(snippet);
        }

        parse_err
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_frontmatter_error() {
        let err = YamlParseError::no_frontmatter();
        assert_eq!(err.kind, YamlParseErrorKind::NoFrontmatter);
        assert!(err.suggestion.is_some());
    }

    #[test]
    fn test_type_mismatch_error() {
        let err = YamlParseError::type_mismatch("refinement", "number", "string");
        assert_eq!(err.kind, YamlParseErrorKind::TypeMismatch);
        assert_eq!(err.field.as_deref(), Some("refinement"));
    }

    #[test]
    fn test_error_with_position() {
        let pos = SourcePosition { line: 5, column: 10, offset: 50 };
        let err = YamlParseError::yaml_syntax("Unexpected token")
            .with_position(pos);
        assert_eq!(err.position, Some(pos));
    }

    #[test]
    fn test_convert_to_domain_error() {
        let err = YamlParseError::yaml_syntax("Test error")
            .with_snippet("some code");

        let domain_err: doc_doctor_domain::ParseError = err.into();
        assert_eq!(domain_err.message, "Test error");
        assert_eq!(domain_err.snippet.as_deref(), Some("some code"));
    }
}
