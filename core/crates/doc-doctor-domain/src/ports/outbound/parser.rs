//! Document Parser Port
//!
//! Format-agnostic trait for parsing document metadata.
//! Implementations can parse YAML frontmatter, LKO format, TOML, etc.

use crate::entities::L1Properties;
use crate::errors::DomainError;
use std::fmt;

/// Source position in a document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset from start of document
    pub offset: usize,
}

impl SourcePosition {
    /// Create a new source position
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }
}

impl fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Span of metadata in a document
#[derive(Debug, Clone)]
pub struct MetadataSpan {
    /// Raw metadata content
    pub content: String,
    /// Start position
    pub start: SourcePosition,
    /// End position
    pub end: SourcePosition,
}

/// Parse error with position information
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Error message
    pub message: String,
    /// Position where error occurred
    pub position: Option<SourcePosition>,
    /// Code snippet around the error
    pub snippet: Option<String>,
    /// Underlying domain error (if applicable)
    pub domain_error: Option<DomainError>,
}

impl ParseError {
    /// Create a new parse error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: None,
            snippet: None,
            domain_error: None,
        }
    }

    /// Add position information
    pub fn with_position(mut self, position: SourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Add code snippet
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Add underlying domain error
    pub fn with_domain_error(mut self, error: DomainError) -> Self {
        self.domain_error = Some(error);
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pos) = &self.position {
            write!(f, "[{}] {}", pos, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}

/// Error type for serialization operations
#[derive(Debug, Clone)]
pub struct SerializeError {
    /// Error message
    pub message: String,
}

impl SerializeError {
    /// Create a new serialize error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SerializeError {}

/// Document parser trait - format-agnostic interface
///
/// This is an outbound port: the domain requests parsing services
/// through this trait, which adapters implement.
///
/// # Example Implementations
/// - `YamlParser`: Parses YAML frontmatter in Markdown files
/// - `LkoParser`: Parses LKO (Living Knowledge Object) format
/// - `TomlParser`: Parses TOML frontmatter
pub trait DocumentParser: Send + Sync {
    /// Parse document content into L1 properties
    ///
    /// # Arguments
    /// * `content` - Raw document content
    ///
    /// # Returns
    /// Parsed L1 properties or parse error with position
    fn parse(&self, content: &str) -> Result<L1Properties, ParseError>;

    /// Extract raw metadata without full parsing
    ///
    /// Useful for validation or quick inspection without
    /// deserializing all fields.
    ///
    /// # Arguments
    /// * `content` - Raw document content
    ///
    /// # Returns
    /// Metadata span if found, None otherwise
    fn extract_metadata(&self, content: &str) -> Option<MetadataSpan>;

    /// Get supported format identifier
    ///
    /// # Returns
    /// Format identifier string (e.g., "yaml", "lko", "toml")
    fn format_id(&self) -> &'static str;
}

/// Document writer trait - format-agnostic interface for serialization
///
/// This is an outbound port: the domain requests serialization services
/// through this trait, which adapters implement.
///
/// # Design Note
///
/// This trait is separate from `DocumentParser` to allow implementations
/// that can parse but not serialize (e.g., read-only parsers).
pub trait DocumentWriter: Send + Sync {
    /// Serialize L1 properties back into document content
    ///
    /// Takes the original document content and updated properties,
    /// replaces the frontmatter with serialized properties, and
    /// returns the complete document.
    ///
    /// # Arguments
    /// * `original_content` - Original document content
    /// * `properties` - Updated L1 properties to serialize
    ///
    /// # Returns
    /// Updated document content with new frontmatter
    fn serialize_document(
        &self,
        original_content: &str,
        properties: &L1Properties,
    ) -> Result<String, SerializeError>;

    /// Serialize only the frontmatter portion
    ///
    /// # Arguments
    /// * `properties` - L1 properties to serialize
    ///
    /// # Returns
    /// Serialized frontmatter string (without delimiters)
    fn serialize_frontmatter(&self, properties: &L1Properties) -> Result<String, SerializeError>;
}
