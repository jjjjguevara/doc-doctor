//! Position tracking for YAML parsing
//!
//! Converts byte offsets to line/column positions for error reporting.

use doc_doctor_domain::SourcePosition;

/// Tracks positions within a document for error reporting
#[derive(Debug, Clone)]
pub struct PositionTracker<'a> {
    /// Full document content
    content: &'a str,
    /// Byte offset where frontmatter content begins (after opening ---)
    frontmatter_offset: usize,
    /// Precomputed line starts for efficient lookups
    line_starts: Vec<usize>,
}

impl<'a> PositionTracker<'a> {
    /// Create a new position tracker
    pub fn new(content: &'a str, frontmatter_offset: usize) -> Self {
        let line_starts = Self::compute_line_starts(content);
        Self {
            content,
            frontmatter_offset,
            line_starts,
        }
    }

    /// Compute line start offsets for the content
    fn compute_line_starts(content: &str) -> Vec<usize> {
        let mut starts = vec![0];
        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                starts.push(i + 1);
            }
        }
        starts
    }

    /// Convert a byte offset within frontmatter to a document position
    pub fn frontmatter_position(&self, offset: usize) -> SourcePosition {
        let absolute_offset = self.frontmatter_offset + offset;
        self.position_from_offset(absolute_offset)
    }

    /// Get position at start of frontmatter
    pub fn frontmatter_start(&self) -> SourcePosition {
        self.position_from_offset(self.frontmatter_offset)
    }

    /// Convert an absolute byte offset to a position
    pub fn position_from_offset(&self, offset: usize) -> SourcePosition {
        // Binary search for the line
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };

        let line_start = self.line_starts[line_idx];
        let column = offset - line_start + 1;

        SourcePosition {
            line: line_idx + 1, // 1-indexed
            column,
            offset,
        }
    }

    /// Extract a snippet around a position
    pub fn extract_snippet(&self, offset: usize, context: usize) -> String {
        let start = offset.saturating_sub(context);
        let end = (offset + context).min(self.content.len());

        // Find valid UTF-8 char boundaries
        let start = self.floor_char_boundary(start);
        let end = self.ceil_char_boundary(end);

        // Extend to line boundaries for better context
        let snippet = &self.content[start..end];
        snippet.replace('\n', "\\n")
    }

    /// Find the largest valid char boundary <= index
    fn floor_char_boundary(&self, index: usize) -> usize {
        if index >= self.content.len() {
            return self.content.len();
        }
        let mut i = index;
        while i > 0 && !self.content.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    /// Find the smallest valid char boundary >= index
    fn ceil_char_boundary(&self, index: usize) -> usize {
        if index >= self.content.len() {
            return self.content.len();
        }
        let mut i = index;
        while i < self.content.len() && !self.content.is_char_boundary(i) {
            i += 1;
        }
        i
    }

    /// Get the full line containing an offset
    pub fn get_line(&self, offset: usize) -> &str {
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };

        let line_start = self.line_starts[line_idx];
        let line_end = self
            .line_starts
            .get(line_idx + 1)
            .map(|&s| s.saturating_sub(1))
            .unwrap_or(self.content.len());

        &self.content[line_start..line_end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_from_offset() {
        let content = "---\ntitle: Test\nrefinement: 0.5\n---\n";
        let tracker = PositionTracker::new(content, 0);

        // Start of file
        let pos = tracker.position_from_offset(0);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);

        // Start of line 2 (after "---\n")
        let pos = tracker.position_from_offset(4);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);

        // Middle of "title"
        let pos = tracker.position_from_offset(6);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 3);
    }

    #[test]
    fn test_frontmatter_position() {
        let content = "---\ntitle: Test\n---\n";
        let tracker = PositionTracker::new(content, 4); // After "---\n"

        // Position 0 in frontmatter = absolute position 4
        let pos = tracker.frontmatter_position(0);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);

        // Position 6 in frontmatter ("Test" start)
        let pos = tracker.frontmatter_position(7);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 8);
    }

    #[test]
    fn test_get_line() {
        let content = "line1\nline2\nline3";
        let tracker = PositionTracker::new(content, 0);

        assert_eq!(tracker.get_line(0), "line1");
        assert_eq!(tracker.get_line(3), "line1");
        assert_eq!(tracker.get_line(6), "line2");
        assert_eq!(tracker.get_line(12), "line3");
    }

    #[test]
    fn test_extract_snippet() {
        let content = "---\ntitle: Test Document\n---\n";
        let tracker = PositionTracker::new(content, 0);

        // Position 4 is start of "title"
        let snippet = tracker.extract_snippet(4, 10);
        assert!(snippet.contains("title"));
    }
}
