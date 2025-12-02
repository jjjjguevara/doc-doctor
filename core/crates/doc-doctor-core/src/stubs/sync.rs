//! Stub-Anchor Synchronization
//!
//! Tracks sync status between frontmatter stubs and inline anchors.

use serde::{Deserialize, Serialize};
use super::Stub;

/// Sync status between stubs and inline anchors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Stubs with resolved anchors
    pub synced_stubs: Vec<SyncedStub>,

    /// Stubs without inline anchors
    pub orphaned_stubs: Vec<OrphanedStub>,

    /// Inline anchors without matching stubs
    pub orphaned_anchors: Vec<OrphanedAnchor>,

    /// Total stub count
    pub total_stubs: usize,

    /// Sync percentage (0.0-1.0)
    pub sync_ratio: f64,
}

impl SyncStatus {
    /// Create sync status from analysis results
    pub fn new(
        synced: Vec<SyncedStub>,
        orphaned_stubs: Vec<OrphanedStub>,
        orphaned_anchors: Vec<OrphanedAnchor>,
    ) -> Self {
        let total = synced.len() + orphaned_stubs.len();
        let sync_ratio = if total > 0 {
            synced.len() as f64 / total as f64
        } else {
            1.0
        };

        Self {
            synced_stubs: synced,
            orphaned_stubs,
            orphaned_anchors,
            total_stubs: total,
            sync_ratio,
        }
    }

    /// Check if all stubs are synced
    pub fn is_fully_synced(&self) -> bool {
        self.orphaned_stubs.is_empty() && self.orphaned_anchors.is_empty()
    }

    /// Check if there are any orphaned items
    pub fn has_orphans(&self) -> bool {
        !self.orphaned_stubs.is_empty() || !self.orphaned_anchors.is_empty()
    }
}

/// A stub with a resolved inline anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedStub {
    /// The stub
    pub stub: Stub,

    /// The anchor ID
    pub anchor_id: String,

    /// Line number of anchor in document (1-indexed)
    pub anchor_line: usize,
}

/// A stub without a corresponding inline anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanedStub {
    /// The stub
    pub stub: Stub,

    /// Expected anchor ID (if specified)
    pub expected_anchor: Option<String>,
}

/// An inline anchor without a corresponding stub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanedAnchor {
    /// The anchor ID (e.g., "^stub-abc123")
    pub anchor_id: String,

    /// Line number in document (1-indexed)
    pub line: usize,

    /// Column position (1-indexed)
    pub column: usize,
}

/// Parse anchor IDs from document content
pub fn find_anchors(content: &str, prefix: &str) -> Vec<(String, usize, usize)> {
    let mut anchors = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // Find all anchors matching the prefix pattern
        let mut search_start = 0;
        while let Some(pos) = line[search_start..].find(prefix) {
            let abs_pos = search_start + pos;

            // Extract the full anchor ID (until whitespace or end of line)
            let anchor_start = abs_pos;
            let rest = &line[anchor_start..];
            let anchor_end = rest
                .find(|c: char| c.is_whitespace())
                .unwrap_or(rest.len());

            let anchor_id = rest[..anchor_end].to_string();

            if !anchor_id.is_empty() {
                anchors.push((anchor_id, line_num, abs_pos + 1));
            }

            search_start = abs_pos + anchor_end;
        }
    }

    anchors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_anchors() {
        let content = r#"Some text ^stub-abc123

Another line with ^stub-def456 in the middle.

Multiple ^stub-111 and ^stub-222 on same line."#;

        let anchors = find_anchors(content, "^stub-");
        assert_eq!(anchors.len(), 4);
        assert_eq!(anchors[0].0, "^stub-abc123");
        assert_eq!(anchors[0].1, 1); // line 1
        assert_eq!(anchors[1].0, "^stub-def456");
        assert_eq!(anchors[1].1, 3); // line 3
    }

    #[test]
    fn test_sync_status() {
        let synced = vec![SyncedStub {
            stub: Stub::compact("link", "test"),
            anchor_id: "^stub-abc".to_string(),
            anchor_line: 10,
        }];
        let orphaned_stubs = vec![OrphanedStub {
            stub: Stub::compact("expand", "unlinked"),
            expected_anchor: None,
        }];
        let orphaned_anchors = vec![];

        let status = SyncStatus::new(synced, orphaned_stubs, orphaned_anchors);

        assert_eq!(status.total_stubs, 2);
        assert!((status.sync_ratio - 0.5).abs() < 0.01);
        assert!(!status.is_fully_synced());
        assert!(status.has_orphans());
    }
}
