//! Stub Type and Priority
//!
//! Core stub data structure supporting both compact and structured syntax.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::error::DocDoctorError;
use super::StubForm;

/// Priority level for stubs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    /// Get numeric urgency value (0.0-1.0)
    pub const fn urgency(&self) -> f64 {
        match self {
            Priority::Low => 0.25,
            Priority::Medium => 0.50,
            Priority::High => 0.75,
            Priority::Critical => 1.0,
        }
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Critical => "Critical",
        }
    }
}

impl FromStr for Priority {
    type Err = DocDoctorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => Err(DocDoctorError::InvalidPriority {
                priority: s.to_string(),
                position: None,
            }),
        }
    }
}

/// Stub type - the kind of gap this represents
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StubType(String);

impl StubType {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the vector family for this stub type
    pub fn vector_family(&self) -> VectorFamily {
        match self.0.to_lowercase().as_str() {
            // Retrieval family
            "source" | "check" | "link" | "cite" | "verify" => VectorFamily::Retrieval,
            // Computation family
            "data" | "model" | "fix" | "question" | "clarify" => VectorFamily::Computation,
            // Synthesis family
            "balance" | "controversy" | "pov" | "merge" | "reorganize" => VectorFamily::Synthesis,
            // Creation family
            "expand" | "incomplete" | "example-needed" | "draft" | "idea" => VectorFamily::Creation,
            // Structural family
            "split" | "flow" | "blocker" | "dependency" | "move" | "todo" => VectorFamily::Structural,
            // Default to Creation for unknown types
            _ => VectorFamily::Creation,
        }
    }
}

impl Default for StubType {
    fn default() -> Self {
        Self("todo".to_string())
    }
}

impl std::fmt::Display for StubType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Vector family classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorFamily {
    /// Finding existing information
    Retrieval,
    /// Analyzing and deriving
    Computation,
    /// Combining perspectives
    Synthesis,
    /// Generating new content
    Creation,
    /// Architectural changes
    Structural,
}

impl VectorFamily {
    pub const fn display_name(&self) -> &'static str {
        match self {
            VectorFamily::Retrieval => "Retrieval",
            VectorFamily::Computation => "Computation",
            VectorFamily::Synthesis => "Synthesis",
            VectorFamily::Creation => "Creation",
            VectorFamily::Structural => "Structural",
        }
    }
}

/// Stub origin - who identified this gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StubOrigin {
    /// Author surfaced during writing
    #[default]
    AuthorIdentified,
    /// Discovered during peer review
    PeerSurfaced,
    /// Found during QA testing
    QaDetected,
    /// Reported by end users
    UserReported,
    /// Detected automatically
    SystemGenerated,
    /// From external sources
    ExternalCited,
}

/// A stub representing a gap or issue in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stub {
    /// Stub type (e.g., "link", "expand", "fix")
    #[serde(rename = "type")]
    pub stub_type: StubType,

    /// Description of what's needed
    pub description: String,

    /// Severity/lifecycle classification
    #[serde(default)]
    pub stub_form: StubForm,

    /// Priority level
    #[serde(default)]
    pub priority: Priority,

    /// Who identified this stub
    #[serde(default)]
    pub origin: StubOrigin,

    /// Block anchor ID (e.g., "^stub-abc123")
    #[serde(default)]
    pub anchor: Option<String>,

    /// Inline anchor references
    #[serde(default)]
    pub inline_anchors: Vec<String>,

    /// Assigned individuals/teams
    #[serde(default)]
    pub assignees: Vec<String>,

    /// Participants in discussion
    #[serde(default)]
    pub participants: Vec<String>,

    /// Related references (wikilinks, URLs)
    #[serde(default)]
    pub references: Vec<String>,

    /// Dependencies (other stubs or documents)
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Urgency factor (0.0-1.0)
    #[serde(default)]
    pub urgency: Option<f64>,

    /// Impact factor (0.0-1.0)
    #[serde(default)]
    pub impact: Option<f64>,

    /// Complexity factor (0.0-1.0)
    #[serde(default)]
    pub complexity: Option<f64>,
}

impl Stub {
    /// Create a compact stub with just type and description
    pub fn compact(stub_type: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            stub_type: StubType::new(stub_type),
            description: description.into(),
            stub_form: StubForm::default(),
            priority: Priority::default(),
            origin: StubOrigin::default(),
            anchor: None,
            inline_anchors: Vec::new(),
            assignees: Vec::new(),
            participants: Vec::new(),
            references: Vec::new(),
            dependencies: Vec::new(),
            urgency: None,
            impact: None,
            complexity: None,
        }
    }

    /// Get the vector family for this stub
    pub fn vector_family(&self) -> VectorFamily {
        self.stub_type.vector_family()
    }

    /// Check if this stub blocks document progression
    pub fn is_blocking(&self) -> bool {
        self.stub_form.is_blocking()
    }

    /// Get the refinement penalty for this stub
    pub fn refinement_penalty(&self) -> f64 {
        self.stub_form.refinement_penalty()
    }

    /// Calculate urgency (from explicit value or priority)
    pub fn effective_urgency(&self) -> f64 {
        self.urgency.unwrap_or_else(|| self.priority.urgency())
    }

    /// Calculate impact (default 0.5 if not specified)
    pub fn effective_impact(&self) -> f64 {
        self.impact.unwrap_or(0.5)
    }

    /// Calculate complexity (default 0.5 if not specified)
    pub fn effective_complexity(&self) -> f64 {
        self.complexity.unwrap_or(0.5)
    }
}

impl Default for Stub {
    fn default() -> Self {
        Self::compact("todo", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_stub() {
        let stub = Stub::compact("link", "Citation needed");
        assert_eq!(stub.stub_type.as_str(), "link");
        assert_eq!(stub.description, "Citation needed");
        assert_eq!(stub.stub_form, StubForm::Transient);
    }

    #[test]
    fn test_vector_family() {
        assert_eq!(StubType::new("link").vector_family(), VectorFamily::Retrieval);
        assert_eq!(StubType::new("fix").vector_family(), VectorFamily::Computation);
        assert_eq!(StubType::new("expand").vector_family(), VectorFamily::Creation);
        assert_eq!(StubType::new("controversy").vector_family(), VectorFamily::Synthesis);
        assert_eq!(StubType::new("split").vector_family(), VectorFamily::Structural);
    }

    #[test]
    fn test_priority_urgency() {
        assert_eq!(Priority::Low.urgency(), 0.25);
        assert_eq!(Priority::Critical.urgency(), 1.0);
    }

    #[test]
    fn test_is_blocking() {
        let mut stub = Stub::compact("link", "test");
        assert!(!stub.is_blocking());

        stub.stub_form = StubForm::Blocking;
        assert!(stub.is_blocking());
    }
}
