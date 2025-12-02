//! Position tracking for YAML parsing

use crate::error::SourcePosition;

/// Tracks positions within a document for error reporting
#[derive(Debug)]
pub struct PositionTracker<'a> {
    content: &'a str,
    /// Offset where frontmatter content begins (after opening ---)
    frontmatter_offset: usize,
}

impl<'a> PositionTracker<'a> {
    pub fn new(content: &'a str, frontmatter_offset: usize) -> Self {
        Self {
            content,
            frontmatter_offset,
        }
    }

    /// Convert a byte offset within frontmatter to a document position
    pub fn frontmatter_position(&self, offset: usize) -> SourcePosition {
        let absolute_offset = self.frontmatter_offset + offset;
        SourcePosition::from_offset(self.content, absolute_offset)
    }

    /// Get position at start of frontmatter
    pub fn frontmatter_start(&self) -> SourcePosition {
        SourcePosition::from_offset(self.content, self.frontmatter_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmatter_position() {
        let content = "---\ntitle: Test\n---\n";
        let tracker = PositionTracker::new(content, 4); // After "---\n"

        let pos = tracker.frontmatter_position(0);
        assert_eq!(pos.line, 2); // Line 2 is where "title:" starts
        assert_eq!(pos.column, 1);
    }
}
