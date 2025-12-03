//! L1 Intrinsic Properties
//!
//! Core document metadata stored in frontmatter.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Audience, Refinement};
use crate::stubs::Stub;

/// L1 Intrinsic Properties - Core document metadata
///
/// These properties are stored in YAML frontmatter and are
/// context-independent (portable across systems).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1Properties {
    // === Foundational Properties ===
    /// Unique identifier for the document
    #[serde(default)]
    pub uid: Option<String>,

    /// Document title
    #[serde(default)]
    pub title: Option<String>,

    /// Creation timestamp
    #[serde(default)]
    pub created: Option<DateTime<Utc>>,

    /// Last modification timestamp
    #[serde(default)]
    pub modified: Option<DateTime<Utc>>,

    /// Tags/categories
    #[serde(default)]
    pub tags: Vec<String>,

    /// Alternative names/titles
    #[serde(default)]
    pub aliases: Vec<String>,

    // === Core 5 Properties ===
    /// Refinement score (0.0-1.0)
    #[serde(default)]
    pub refinement: Refinement,

    /// Document origin/source
    #[serde(default)]
    pub origin: Origin,

    /// Document form/type
    #[serde(default)]
    pub form: Form,

    /// Target audience
    #[serde(default)]
    pub audience: Audience,

    /// Stubs (gaps, todos, issues)
    #[serde(default)]
    pub stubs: Vec<Stub>,
}

impl Default for L1Properties {
    fn default() -> Self {
        Self {
            uid: None,
            title: None,
            created: None,
            modified: None,
            tags: Vec::new(),
            aliases: Vec::new(),
            refinement: Refinement::default(),
            origin: Origin::default(),
            form: Form::default(),
            audience: Audience::default(),
            stubs: Vec::new(),
        }
    }
}

impl L1Properties {
    /// Create new L1Properties with only the title set
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            ..Default::default()
        }
    }

    /// Get the number of stubs
    pub fn stub_count(&self) -> usize {
        self.stubs.len()
    }

    /// Check if there are any blocking stubs
    pub fn has_blocking_stubs(&self) -> bool {
        self.stubs.iter().any(|s| s.is_blocking())
    }
}

/// Document origin - how the content was created
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// Specific inquiro or problem to solve
    #[default]
    Question,

    /// Workflow, system, or operational necessity
    Requirement,

    /// Spontaneous realization or discovery
    Insight,

    /// Discussion, collaboration, or interactive exchange
    Dialogue,

    /// Exploratory interest or voluntary learning
    Curiosity,

    /// Derived from other artifacts
    Derivative,

    /// Hypothesis testing or prototype development
    Experimental,
}

impl Origin {
    pub const fn display_name(&self) -> &'static str {
        match self {
            Origin::Question => "Question",
            Origin::Requirement => "Requirement",
            Origin::Insight => "Insight",
            Origin::Dialogue => "Dialogue",
            Origin::Curiosity => "Curiosity",
            Origin::Derivative => "Derivative",
            Origin::Experimental => "Experimental",
        }
    }
}

/// Document form - the structural type of content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// Standard document/article
    #[default]
    Document,

    /// Reference/encyclopedia entry
    Reference,

    /// How-to guide/tutorial
    Guide,

    /// Log/journal entry
    Log,

    /// Meeting notes
    Meeting,

    /// Project documentation
    Project,

    /// Personal note
    Note,

    /// Template for other documents
    Template,

    /// Index/hub page
    Index,

    /// Archived/deprecated content
    Archive,
}

impl Form {
    pub const fn display_name(&self) -> &'static str {
        match self {
            Form::Document => "Document",
            Form::Reference => "Reference",
            Form::Guide => "Guide",
            Form::Log => "Log",
            Form::Meeting => "Meeting",
            Form::Project => "Project",
            Form::Note => "Note",
            Form::Template => "Template",
            Form::Index => "Index",
            Form::Archive => "Archive",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_properties() {
        let props = L1Properties::default();
        assert_eq!(props.refinement.value(), 0.0);
        assert_eq!(props.audience, Audience::Personal);
        assert!(props.stubs.is_empty());
    }

    #[test]
    fn test_with_title() {
        let props = L1Properties::with_title("Test Document");
        assert_eq!(props.title.as_deref(), Some("Test Document"));
    }

    #[test]
    fn test_origin_display() {
        assert_eq!(Origin::Human.display_name(), "Human-authored");
        assert_eq!(Origin::Ai.display_name(), "AI-generated");
    }

    #[test]
    fn test_form_display() {
        assert_eq!(Form::Document.display_name(), "Document");
        assert_eq!(Form::Guide.display_name(), "Guide");
    }
}
