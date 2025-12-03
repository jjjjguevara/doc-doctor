//! YAML Frontmatter Parser
//!
//! Implements the `DocumentParser` trait from the domain layer for YAML-based
//! markdown frontmatter parsing.
//!
//! # Features
//!
//! - Extracts YAML frontmatter from markdown documents
//! - Provides accurate line/column positions for error reporting
//! - Supports both compact (`- link: "desc"`) and structured stub syntax
//! - Format identifier: "yaml"
//!
//! # Example
//!
//! ```rust
//! use doc_doctor_parser_yaml::YamlParser;
//! use doc_doctor_domain::DocumentParser;
//!
//! let parser = YamlParser::new();
//! let content = r#"---
//! title: My Document
//! refinement: 0.75
//! ---
//! # Content
//! "#;
//!
//! let props = parser.parse(content).unwrap();
//! assert_eq!(props.title.as_deref(), Some("My Document"));
//! ```

mod error;
mod frontmatter;
mod parser;
mod position;

pub use error::{YamlParseError, YamlParseErrorKind};
pub use frontmatter::{extract_frontmatter, FrontmatterSpan};
pub use parser::YamlParser;
pub use position::PositionTracker;
