//! Frontmatter extraction
//!
//! Extracts YAML frontmatter from markdown documents with position tracking.

use doc_doctor_domain::MetadataSpan;

/// Information about extracted frontmatter
#[derive(Debug, Clone)]
pub struct FrontmatterSpan {
    /// The YAML content (without delimiters)
    pub content: String,
    /// Byte offset where YAML content starts
    pub start_offset: usize,
    /// Byte offset where YAML content ends
    pub end_offset: usize,
    /// Line number where frontmatter starts (1-indexed)
    pub start_line: usize,
    /// Line number where frontmatter ends (1-indexed)
    pub end_line: usize,
}

impl FrontmatterSpan {
    /// Convert to domain MetadataSpan
    pub fn to_metadata_span(&self) -> MetadataSpan {
        use doc_doctor_domain::SourcePosition;

        MetadataSpan {
            content: self.content.clone(),
            start: SourcePosition::new(self.start_line, 1, self.start_offset),
            end: SourcePosition::new(self.end_line, 1, self.end_offset),
        }
    }
}

/// Extract frontmatter from document content
///
/// # Arguments
/// * `content` - Full document content
///
/// # Returns
/// * `Some(FrontmatterSpan)` if valid frontmatter found
/// * `None` if no frontmatter or invalid delimiters
///
/// # Format
/// ```text
/// ---
/// yaml: content
/// here: values
/// ---
/// # Rest of document
/// ```
pub fn extract_frontmatter(content: &str) -> Option<FrontmatterSpan> {
    // Must start with "---"
    if !content.starts_with("---") {
        return None;
    }

    // Find the position after opening delimiter
    let after_opening = &content[3..];

    // Must have newline after opening ---
    let yaml_start = if after_opening.starts_with('\n') {
        4 // "---\n"
    } else if after_opening.starts_with("\r\n") {
        5 // "---\r\n"
    } else {
        return None; // Invalid: no newline after opening ---
    };

    let rest = &content[yaml_start..];

    // Find closing delimiter patterns
    let closing_patterns = [
        ("\n---\n", 4),      // Standard
        ("\n---\r\n", 5),    // Windows
        ("\n---", 4),        // End of file
    ];

    for (pattern, _delimiter_len) in closing_patterns {
        if let Some(pos) = rest.find(pattern) {
            let yaml_end = yaml_start + pos;
            let yaml_content = &content[yaml_start..yaml_end];

            // Count lines for position info
            let start_line = 2; // Line after opening ---
            let end_line = start_line + yaml_content.lines().count();

            return Some(FrontmatterSpan {
                content: yaml_content.to_string(),
                start_offset: yaml_start,
                end_offset: yaml_end,
                start_line,
                end_line,
            });
        }
    }

    // Check if document ends with --- (no trailing newline)
    if rest.ends_with("\n---") {
        let yaml_end = yaml_start + rest.len() - 4;
        let yaml_content = &content[yaml_start..yaml_end];

        let start_line = 2;
        let end_line = start_line + yaml_content.lines().count();

        return Some(FrontmatterSpan {
            content: yaml_content.to_string(),
            start_offset: yaml_start,
            end_offset: yaml_end,
            start_line,
            end_line,
        });
    }

    None
}

/// Extract raw frontmatter without parsing
///
/// Returns just the string content and offsets, useful for validation
/// or when you need the raw YAML without deserializing.
pub fn extract_raw_frontmatter(content: &str) -> Option<(&str, usize, usize)> {
    let span = extract_frontmatter(content)?;
    Some((&content[span.start_offset..span.end_offset], span.start_offset, span.end_offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_basic_frontmatter() {
        let content = "---\ntitle: Test\nrefinement: 0.5\n---\n# Content";

        let span = extract_frontmatter(content).unwrap();
        assert_eq!(span.content, "title: Test\nrefinement: 0.5");
        assert_eq!(span.start_offset, 4);
        assert_eq!(span.start_line, 2);
    }

    #[test]
    fn test_extract_windows_line_endings() {
        let content = "---\r\ntitle: Test\r\n---\r\nContent";

        let span = extract_frontmatter(content).unwrap();
        // Windows line endings include \r in the content
        assert!(span.content.contains("title: Test"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just markdown\n\nNo frontmatter here.";
        assert!(extract_frontmatter(content).is_none());
    }

    #[test]
    fn test_missing_newline_after_opening() {
        let content = "---title: Test\n---\n";
        assert!(extract_frontmatter(content).is_none());
    }

    #[test]
    fn test_frontmatter_at_end_of_file() {
        let content = "---\ntitle: Test\n---";

        let span = extract_frontmatter(content).unwrap();
        assert_eq!(span.content, "title: Test");
    }

    #[test]
    fn test_multiline_frontmatter() {
        let content = "---\ntitle: Test\nrefinement: 0.75\naudience: internal\nstubs:\n  - type: link\n    description: Citation\n---\n";

        let span = extract_frontmatter(content).unwrap();
        assert!(span.content.contains("title: Test"));
        assert!(span.content.contains("audience: internal"));
        assert!(span.content.contains("stubs:"));
    }

    #[test]
    fn test_to_metadata_span() {
        let content = "---\ntitle: Test\n---\n";

        let span = extract_frontmatter(content).unwrap();
        let metadata = span.to_metadata_span();

        assert_eq!(metadata.content, span.content);
        assert_eq!(metadata.start.offset, span.start_offset);
        assert_eq!(metadata.end.offset, span.end_offset);
    }

    #[test]
    fn test_raw_frontmatter() {
        let content = "---\ntitle: Test\n---\n";

        let (yaml, start, end) = extract_raw_frontmatter(content).unwrap();
        assert_eq!(yaml, "title: Test");
        assert_eq!(start, 4);
        assert!(end > start);
    }
}
