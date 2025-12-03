//! L1 Intrinsic Properties
//!
//! Core document metadata stored in frontmatter.
//! These properties are context-independent (portable across systems).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Audience, Form, Origin, Refinement, Stub};

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

    /// Creation driver
    #[serde(default)]
    pub origin: Origin,

    /// Permanence intent
    #[serde(default)]
    pub form: Form,

    /// Intended visibility
    #[serde(default)]
    pub audience: Audience,

    /// Editorial demand signals
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
    /// Create a new empty L1Properties
    pub fn new() -> Self {
        Self::default()
    }

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

    /// Get blocking stubs
    pub fn blocking_stubs(&self) -> Vec<&Stub> {
        self.stubs.iter().filter(|s| s.is_blocking()).collect()
    }

    /// Builder: set refinement
    pub fn refinement(mut self, value: f64) -> Self {
        self.refinement = Refinement::new_clamped(value);
        self
    }

    /// Builder: set audience
    pub fn audience(mut self, audience: Audience) -> Self {
        self.audience = audience;
        self
    }

    /// Builder: set origin
    pub fn origin(mut self, origin: Origin) -> Self {
        self.origin = origin;
        self
    }

    /// Builder: set form
    pub fn form(mut self, form: Form) -> Self {
        self.form = form;
        self
    }

    /// Builder: add a stub
    pub fn with_stub(mut self, stub: Stub) -> Self {
        self.stubs.push(stub);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::StubForm;

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
    fn test_builder_pattern() {
        let props = L1Properties::new()
            .refinement(0.8)
            .audience(Audience::Internal)
            .origin(Origin::Human)
            .form(Form::Stable);

        assert_eq!(props.refinement.value(), 0.8);
        assert_eq!(props.audience, Audience::Internal);
        assert_eq!(props.origin, Origin::Human);
        assert_eq!(props.form, Form::Stable);
    }

    #[test]
    fn test_has_blocking_stubs() {
        let mut props = L1Properties::default();
        assert!(!props.has_blocking_stubs());

        props.stubs.push(Stub::compact("link", "test"));
        assert!(!props.has_blocking_stubs());

        let mut blocking_stub = Stub::compact("fix", "critical");
        blocking_stub.stub_form = StubForm::Blocking;
        props.stubs.push(blocking_stub);
        assert!(props.has_blocking_stubs());
    }

    #[test]
    fn test_blocking_stubs() {
        let mut props = L1Properties::default();
        props.stubs.push(Stub::compact("link", "test1"));

        let mut blocking = Stub::compact("fix", "critical");
        blocking.stub_form = StubForm::Blocking;
        props.stubs.push(blocking);

        props.stubs.push(Stub::compact("expand", "test2"));

        let blocking_stubs = props.blocking_stubs();
        assert_eq!(blocking_stubs.len(), 1);
        assert_eq!(blocking_stubs[0].description, "critical");
    }
}
