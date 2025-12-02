//! Document and Frontmatter Parsing
//!
//! Extracts L1 properties from markdown documents with YAML frontmatter.

mod frontmatter;
mod yaml_strict;
mod position;

pub use frontmatter::{parse_document, validate_frontmatter, extract_frontmatter};
pub use position::PositionTracker;
