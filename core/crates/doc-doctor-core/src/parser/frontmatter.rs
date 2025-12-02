//! Frontmatter extraction and parsing

use crate::error::{DocDoctorError, Result, ValidationWarning};
use crate::types::L1Properties;
use super::PositionTracker;

/// Extract frontmatter YAML from document content
///
/// # Returns
/// * `Some((yaml_content, start_offset, end_offset))` if frontmatter exists
/// * `None` if no valid frontmatter delimiters found
pub fn extract_frontmatter(content: &str) -> Option<(&str, usize, usize)> {
    // Must start with "---"
    if !content.starts_with("---") {
        return None;
    }

    // Find the closing "---"
    let after_opening = &content[3..];

    // Skip optional newline after opening delimiter
    let yaml_start = if after_opening.starts_with('\n') {
        4
    } else if after_opening.starts_with("\r\n") {
        5
    } else {
        return None; // Invalid: no newline after opening ---
    };

    // Find closing delimiter
    let rest = &content[yaml_start..];
    let closing_patterns = ["\n---\n", "\n---\r\n", "\n---"];

    for pattern in closing_patterns {
        if let Some(pos) = rest.find(pattern) {
            let yaml_end = yaml_start + pos;
            return Some((&content[yaml_start..yaml_end], yaml_start, yaml_end));
        }
    }

    // Check if document ends with ---
    if rest.ends_with("\n---") {
        let yaml_end = yaml_start + rest.len() - 4;
        return Some((&content[yaml_start..yaml_end], yaml_start, yaml_end));
    }

    None
}

/// Parse a markdown document and extract L1 properties
pub fn parse_document(content: &str) -> Result<L1Properties> {
    let (yaml_content, start_offset, _end_offset) = extract_frontmatter(content)
        .ok_or(DocDoctorError::NoFrontmatter)?;

    let tracker = PositionTracker::new(content, start_offset);

    // Parse YAML
    let props: L1Properties = serde_yaml::from_str(yaml_content).map_err(|e| {
        let position = e.location().map(|loc| {
            tracker.frontmatter_position(loc.index())
        });

        DocDoctorError::YamlParse {
            message: e.to_string(),
            position,
            source_snippet: extract_snippet(yaml_content, e.location().map(|l| l.index())),
        }
    })?;

    Ok(props)
}

/// Validate frontmatter without full parsing
pub fn validate_frontmatter(content: &str, strict: bool) -> Result<Vec<ValidationWarning>> {
    let mut warnings = Vec::new();

    let (yaml_content, start_offset, _) = extract_frontmatter(content)
        .ok_or(DocDoctorError::NoFrontmatter)?;

    let _tracker = PositionTracker::new(content, start_offset);

    // Try to parse as generic YAML first
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content).map_err(|e| {
        DocDoctorError::YamlParse {
            message: e.to_string(),
            position: None,
            source_snippet: None,
        }
    })?;

    // Check for unknown fields if strict mode
    if strict {
        if let serde_yaml::Value::Mapping(map) = &yaml_value {
            let known_fields = [
                "uid", "title", "created", "modified", "tags", "aliases",
                "refinement", "origin", "form", "audience", "stubs",
            ];

            for key in map.keys() {
                if let serde_yaml::Value::String(key_str) = key {
                    if !known_fields.contains(&key_str.as_str()) {
                        warnings.push(ValidationWarning {
                            message: format!("Unknown field: {}", key_str),
                            field: Some(key_str.clone()),
                            position: None,
                            suggestion: None,
                        });
                    }
                }
            }
        }
    }

    // Try to parse as L1Properties to catch type errors
    let _props: L1Properties = serde_yaml::from_str(yaml_content).map_err(|e| {
        DocDoctorError::YamlParse {
            message: e.to_string(),
            position: None,
            source_snippet: None,
        }
    })?;

    Ok(warnings)
}

/// Extract a snippet around an error position
fn extract_snippet(content: &str, offset: Option<usize>) -> Option<String> {
    let offset = offset?;
    let start = offset.saturating_sub(20);
    let end = (offset + 20).min(content.len());

    let snippet = &content[start..end];
    Some(snippet.replace('\n', "\\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter() {
        let content = "---\ntitle: Test\nrefinement: 0.5\n---\n# Content";

        let result = extract_frontmatter(content);
        assert!(result.is_some());

        let (yaml, start, _end) = result.unwrap();
        // The YAML content between the --- delimiters
        assert_eq!(yaml, "title: Test\nrefinement: 0.5");
        assert_eq!(start, 4);
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just markdown\n\nNo frontmatter here.";
        assert!(extract_frontmatter(content).is_none());
    }

    #[test]
    fn test_parse_document() {
        // Note: Currently using structured stub syntax.
        // Compact syntax (- link: "Citation needed") requires custom deserializer.
        let content = r#"---
title: My Document
refinement: 0.75
audience: internal
stubs:
  - type: link
    description: "Citation needed"
---
# Content here"#;

        let props = parse_document(content).unwrap();
        assert_eq!(props.title.as_deref(), Some("My Document"));
        assert_eq!(props.refinement.value(), 0.75);
        assert_eq!(props.stubs.len(), 1);
        assert_eq!(props.stubs[0].stub_type.as_str(), "link");
    }

    #[test]
    fn test_parse_error_position() {
        let content = "---\ntitle: Test\nrefinement: invalid\n---\n";

        let err = parse_document(content).unwrap_err();
        assert!(matches!(err, DocDoctorError::YamlParse { .. }));
    }

    #[test]
    fn test_validate_strict() {
        let content = "---\ntitle: Test\ncustom_field: value\n---\n";

        let warnings = validate_frontmatter(content, true).unwrap();
        assert!(warnings.iter().any(|w| w.field.as_deref() == Some("custom_field")));
    }
}
