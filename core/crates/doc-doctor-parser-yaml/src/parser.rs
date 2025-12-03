//! YAML Parser Implementation
//!
//! Implements the `DocumentParser` and `DocumentWriter` traits for YAML frontmatter.

use doc_doctor_domain::{
    DocumentParser, DocumentWriter, L1Properties, MetadataSpan, ParseError, SerializeError,
    SourcePosition,
};

use crate::error::{YamlParseError, YamlParseErrorKind};
use crate::frontmatter::{extract_frontmatter, FrontmatterSpan};
use crate::position::PositionTracker;

/// YAML frontmatter parser
///
/// Parses markdown documents with YAML frontmatter and extracts L1 properties.
#[derive(Debug, Clone, Default)]
pub struct YamlParser {
    /// Whether to reject unknown fields
    strict: bool,
}

impl YamlParser {
    /// Create a new YAML parser
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict parser that rejects unknown fields
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Set strict mode
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Parse frontmatter and return detailed result
    pub fn parse_detailed(&self, content: &str) -> Result<ParseResult, YamlParseError> {
        // Extract frontmatter
        let span = extract_frontmatter(content)
            .ok_or_else(YamlParseError::no_frontmatter)?;

        let tracker = PositionTracker::new(content, span.start_offset);

        // Check for unknown fields if strict
        let warnings = if self.strict {
            self.check_unknown_fields(&span.content, &tracker)?
        } else {
            Vec::new()
        };

        // Parse YAML to L1Properties
        let properties: L1Properties = serde_yaml::from_str(&span.content).map_err(|e| {
            self.convert_yaml_error(e, &span, &tracker)
        })?;

        Ok(ParseResult {
            properties,
            span,
            warnings,
        })
    }

    /// Check for unknown fields in YAML
    fn check_unknown_fields(
        &self,
        yaml_content: &str,
        _tracker: &PositionTracker,
    ) -> Result<Vec<ParseWarning>, YamlParseError> {
        let mut warnings = Vec::new();

        // Parse as generic YAML value
        let value: serde_yaml::Value = serde_yaml::from_str(yaml_content).map_err(|e| {
            YamlParseError::yaml_syntax(e.to_string())
        })?;

        // Known L1 property fields
        const KNOWN_FIELDS: &[&str] = &[
            "uid", "title", "created", "modified", "tags", "aliases",
            "refinement", "origin", "form", "audience", "stubs",
        ];

        // Known stub fields
        const KNOWN_STUB_FIELDS: &[&str] = &[
            "type", "description", "gap_id", "stub_form", "stub_origin",
            "priority", "urgency", "impact", "complexity",
            "inline_anchors", "assignees", "dependencies", "notes",
        ];

        if let serde_yaml::Value::Mapping(map) = &value {
            for (key, val) in map {
                if let serde_yaml::Value::String(key_str) = key {
                    if !KNOWN_FIELDS.contains(&key_str.as_str()) {
                        warnings.push(ParseWarning {
                            message: format!("Unknown field: {}", key_str),
                            field: Some(key_str.clone()),
                            position: None,
                            suggestion: Some("Check J-Editorial schema for valid fields".to_string()),
                        });
                    }

                    // Check stub fields
                    if key_str == "stubs" {
                        if let serde_yaml::Value::Sequence(stubs) = val {
                            for (i, stub) in stubs.iter().enumerate() {
                                if let serde_yaml::Value::Mapping(stub_map) = stub {
                                    for stub_key in stub_map.keys() {
                                        if let serde_yaml::Value::String(stub_key_str) = stub_key {
                                            if !KNOWN_STUB_FIELDS.contains(&stub_key_str.as_str()) {
                                                warnings.push(ParseWarning {
                                                    message: format!(
                                                        "Unknown stub field: {} (in stubs[{}])",
                                                        stub_key_str, i
                                                    ),
                                                    field: Some(format!("stubs[{}].{}", i, stub_key_str)),
                                                    position: None,
                                                    suggestion: None,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(warnings)
    }

    /// Convert serde_yaml error to our error type
    fn convert_yaml_error(
        &self,
        err: serde_yaml::Error,
        span: &FrontmatterSpan,
        tracker: &PositionTracker,
    ) -> YamlParseError {
        let position = err.location().map(|loc| {
            tracker.frontmatter_position(loc.index())
        });

        let snippet = err.location().map(|loc| {
            tracker.extract_snippet(span.start_offset + loc.index(), 30)
        });

        YamlParseError {
            kind: YamlParseErrorKind::YamlSyntax,
            message: err.to_string(),
            position,
            snippet,
            field: None,
            suggestion: Some("Check YAML syntax (indentation, colons, quotes)".to_string()),
        }
    }
}

impl DocumentParser for YamlParser {
    fn parse(&self, content: &str) -> Result<L1Properties, ParseError> {
        self.parse_detailed(content)
            .map(|r| r.properties)
            .map_err(|e| e.into())
    }

    fn extract_metadata(&self, content: &str) -> Option<MetadataSpan> {
        extract_frontmatter(content).map(|s| s.to_metadata_span())
    }

    fn format_id(&self) -> &'static str {
        "yaml"
    }
}

impl DocumentWriter for YamlParser {
    fn serialize_document(
        &self,
        original_content: &str,
        properties: &L1Properties,
    ) -> Result<String, SerializeError> {
        // Serialize frontmatter
        let yaml = self.serialize_frontmatter(properties)?;

        // Extract the current frontmatter span to find where to replace
        if let Some(span) = extract_frontmatter(original_content) {
            // Get content after frontmatter
            // Find the end of the closing delimiter
            let after_frontmatter_start = span.end_offset;

            // Find the closing delimiter and skip past it
            let rest_of_content = &original_content[after_frontmatter_start..];
            let body_start = if rest_of_content.starts_with("\n---\n") {
                after_frontmatter_start + 5
            } else if rest_of_content.starts_with("\n---\r\n") {
                after_frontmatter_start + 6
            } else if rest_of_content.starts_with("\n---") {
                // End of file case
                after_frontmatter_start + 4
            } else {
                // Just newline
                after_frontmatter_start
            };

            let body = if body_start < original_content.len() {
                &original_content[body_start..]
            } else {
                ""
            };

            Ok(format!("---\n{}\n---\n{}", yaml, body))
        } else {
            // No existing frontmatter - prepend new one
            Ok(format!("---\n{}\n---\n\n{}", yaml, original_content))
        }
    }

    fn serialize_frontmatter(&self, properties: &L1Properties) -> Result<String, SerializeError> {
        serde_yaml::to_string(properties).map_err(|e| SerializeError::new(e.to_string()))
    }
}

/// Detailed parse result
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Parsed L1 properties
    pub properties: L1Properties,
    /// Frontmatter span information
    pub span: FrontmatterSpan,
    /// Validation warnings
    pub warnings: Vec<ParseWarning>,
}

/// Parse warning (non-fatal)
#[derive(Debug, Clone)]
pub struct ParseWarning {
    /// Warning message
    pub message: String,
    /// Field that triggered the warning
    pub field: Option<String>,
    /// Position in source
    pub position: Option<SourcePosition>,
    /// Suggestion for fixing
    pub suggestion: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_doctor_domain::Audience;

    #[test]
    fn test_parse_basic_document() {
        let parser = YamlParser::new();
        let content = r#"---
title: My Document
refinement: 0.75
audience: internal
---
# Content here"#;

        let props = parser.parse(content).unwrap();
        assert_eq!(props.title.as_deref(), Some("My Document"));
        assert_eq!(props.refinement.value(), 0.75);
        assert_eq!(props.audience, Audience::Internal);
    }

    #[test]
    fn test_parse_with_stubs() {
        let parser = YamlParser::new();
        let content = r#"---
title: Document with Stubs
refinement: 0.6
stubs:
  - type: link
    description: "Citation needed"
  - type: expand
    description: "Add more detail"
    priority: high
---
# Content"#;

        let props = parser.parse(content).unwrap();
        assert_eq!(props.stubs.len(), 2);
        assert_eq!(props.stubs[0].stub_type.as_str(), "link");
        assert_eq!(props.stubs[1].stub_type.as_str(), "expand");
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let parser = YamlParser::new();
        let content = "# Just markdown\nNo frontmatter.";

        let err = parser.parse(content).unwrap_err();
        assert!(err.message.contains("frontmatter"));
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let parser = YamlParser::new();
        let content = "---\ntitle: [invalid yaml\n---\n";

        let err = parser.parse(content).unwrap_err();
        assert!(!err.message.is_empty());
    }

    #[test]
    fn test_strict_mode_unknown_field() {
        let parser = YamlParser::strict();
        let content = "---\ntitle: Test\ncustom_field: value\n---\n";

        let result = parser.parse_detailed(content).unwrap();
        assert!(result.warnings.iter().any(|w| w.field.as_deref() == Some("custom_field")));
    }

    #[test]
    fn test_format_id() {
        let parser = YamlParser::new();
        assert_eq!(parser.format_id(), "yaml");
    }

    #[test]
    fn test_extract_metadata() {
        let parser = YamlParser::new();
        let content = "---\ntitle: Test\n---\n";

        let span = parser.extract_metadata(content).unwrap();
        assert_eq!(span.content, "title: Test");
    }

    #[test]
    fn test_parse_detailed() {
        let parser = YamlParser::new();
        let content = "---\ntitle: Test\nrefinement: 0.5\n---\n";

        let result = parser.parse_detailed(content).unwrap();
        assert_eq!(result.properties.title.as_deref(), Some("Test"));
        assert_eq!(result.span.start_line, 2);
    }

    #[test]
    fn test_parse_all_audiences() {
        let parser = YamlParser::new();

        for (audience_str, expected) in [
            ("personal", Audience::Personal),
            ("internal", Audience::Internal),
            ("trusted", Audience::Trusted),
            ("public", Audience::Public),
        ] {
            let content = format!("---\naudience: {}\n---\n", audience_str);
            let props = parser.parse(&content).unwrap();
            assert_eq!(props.audience, expected);
        }
    }

    #[test]
    fn test_parse_default_values() {
        let parser = YamlParser::new();
        let content = "---\ntitle: Minimal\n---\n";

        let props = parser.parse(content).unwrap();
        assert_eq!(props.refinement.value(), 0.0);
        assert_eq!(props.audience, Audience::Personal);
        assert!(props.stubs.is_empty());
    }
}
