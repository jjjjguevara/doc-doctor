//! Stub Types
//!
//! Stubs are dynamic demand signals representing gaps in documents.
//! Each stub belongs to one of 5 vector families: Retrieval, Computation,
//! Synthesis, Creation, or Structural.
//!
//! Supports multiple frontmatter formats:
//! - Compact: `- verify: "description"`
//! - Compact with object: `- verify: { description: "text", inline_anchor: ^anchor }`
//! - Expanded: `- stub_type: verify, description: "text", stub_origin: qa-detected`
//! - Legacy: `- type: link, description: "text"`

use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;
use crate::errors::{DomainError, DomainResult};

// ============================================================================
// StubForm - Severity/Permanence Classification
// ============================================================================

/// Stub form - severity and expected lifecycle
///
/// Forms determine how stubs impact refinement scores and workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StubForm {
    /// Minor issue, resolve soon
    /// Refinement penalty: -0.02
    #[default]
    Transient,

    /// Long-term gap, document anyway
    /// Refinement penalty: -0.05
    Persistent,

    /// Must resolve before progression
    /// Refinement penalty: -0.10
    Blocking,

    /// Fundamental architecture issue
    /// Refinement penalty: -0.15
    Structural,
}

impl StubForm {
    /// Get the refinement penalty for this form
    pub const fn refinement_penalty(&self) -> f64 {
        match self {
            StubForm::Transient => 0.02,
            StubForm::Persistent => 0.05,
            StubForm::Blocking => 0.10,
            StubForm::Structural => 0.15,
        }
    }

    /// Get the display name
    pub const fn display_name(&self) -> &'static str {
        match self {
            StubForm::Transient => "Transient",
            StubForm::Persistent => "Persistent",
            StubForm::Blocking => "Blocking",
            StubForm::Structural => "Structural",
        }
    }

    /// Get all valid forms
    pub const fn all() -> &'static [StubForm] {
        &[
            StubForm::Transient,
            StubForm::Persistent,
            StubForm::Blocking,
            StubForm::Structural,
        ]
    }

    /// Check if this form blocks document progression
    pub const fn is_blocking(&self) -> bool {
        matches!(self, StubForm::Blocking | StubForm::Structural)
    }

    /// Parse stub form from string
    pub fn parse(s: &str) -> DomainResult<Self> {
        Self::from_str(s)
    }
}

impl FromStr for StubForm {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "transient" => Ok(StubForm::Transient),
            "persistent" => Ok(StubForm::Persistent),
            "blocking" => Ok(StubForm::Blocking),
            "structural" => Ok(StubForm::Structural),
            _ => Err(DomainError::UnknownStubForm {
                value: s.to_string(),
            }),
        }
    }
}

impl std::fmt::Display for StubForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            StubForm::Transient => "transient",
            StubForm::Persistent => "persistent",
            StubForm::Blocking => "blocking",
            StubForm::Structural => "structural",
        })
    }
}

// ============================================================================
// Priority
// ============================================================================

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

    /// Get display name
    pub const fn display_name(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Critical => "Critical",
        }
    }

    /// Parse priority from string
    pub fn parse(s: &str) -> DomainResult<Self> {
        Self::from_str(s)
    }
}

impl FromStr for Priority {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => Err(DomainError::UnknownPriority {
                value: s.to_string(),
            }),
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Critical => "critical",
        })
    }
}

// ============================================================================
// Vector Family
// ============================================================================

/// Vector family classification
///
/// Categorizes stubs by the type of work required to resolve them.
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
    /// Get display name
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

impl Default for VectorFamily {
    fn default() -> Self {
        VectorFamily::Creation
    }
}

// ============================================================================
// StubType
// ============================================================================

/// Stub type - the kind of gap this represents
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StubType(String);

impl StubType {
    /// Create a new stub type
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the string value
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

// ============================================================================
// StubOrigin
// ============================================================================

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

impl StubOrigin {
    /// Get display name
    pub const fn display_name(&self) -> &'static str {
        match self {
            StubOrigin::AuthorIdentified => "Author Identified",
            StubOrigin::PeerSurfaced => "Peer Surfaced",
            StubOrigin::QaDetected => "QA Detected",
            StubOrigin::UserReported => "User Reported",
            StubOrigin::SystemGenerated => "System Generated",
            StubOrigin::ExternalCited => "External Cited",
        }
    }
}

impl FromStr for StubOrigin {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace(['-', ' '], "_").as_str() {
            "author_identified" | "author" => Ok(StubOrigin::AuthorIdentified),
            "peer_surfaced" | "peer" => Ok(StubOrigin::PeerSurfaced),
            "qa_detected" | "qa" => Ok(StubOrigin::QaDetected),
            "user_reported" | "user" => Ok(StubOrigin::UserReported),
            "system_generated" | "system" => Ok(StubOrigin::SystemGenerated),
            "external_cited" | "external" => Ok(StubOrigin::ExternalCited),
            _ => Err(DomainError::InvalidCalculationInput {
                message: format!("Unknown stub origin: {}", s),
            }),
        }
    }
}

impl std::fmt::Display for StubOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            StubOrigin::AuthorIdentified => "author_identified",
            StubOrigin::PeerSurfaced => "peer_surfaced",
            StubOrigin::QaDetected => "qa_detected",
            StubOrigin::UserReported => "user_reported",
            StubOrigin::SystemGenerated => "system_generated",
            StubOrigin::ExternalCited => "external_cited",
        })
    }
}

// ============================================================================
// Stub
// ============================================================================

/// A stub representing a gap or issue in a document
///
/// Supports multiple frontmatter formats via custom deserializer.
#[derive(Debug, Clone, Serialize)]
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

/// All known stub types from J-Editorial framework
const KNOWN_STUB_TYPES: &[&str] = &[
    // Retrieval family
    "source", "check", "link", "cite", "verify", "citation-needed",
    // Computation family
    "data", "model", "fix", "question", "clarify",
    // Synthesis family
    "balance", "controversy", "pov", "merge", "reorganize",
    // Creation family
    "expand", "incomplete", "example-needed", "draft", "idea",
    // Structural family
    "split", "flow", "blocker", "dependency", "move", "todo",
];

impl<'de> Deserialize<'de> for Stub {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct StubVisitor;

        impl<'de> Visitor<'de> for StubVisitor {
            type Value = Stub;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a stub object in compact, expanded, or legacy format")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Stub, M::Error>
            where
                M: MapAccess<'de>,
            {
                // Collect all key-value pairs first
                let mut entries: Vec<(String, serde_yaml::Value)> = Vec::new();
                while let Some((key, value)) = map.next_entry::<String, serde_yaml::Value>()? {
                    entries.push((key, value));
                }

                // Determine format based on keys present
                let has_type = entries.iter().any(|(k, _)| k == "type");
                let has_stub_type = entries.iter().any(|(k, _)| k == "stub_type");

                // Find compact format key (stub type as key name)
                let compact_key = entries.iter().find(|(k, _)| {
                    KNOWN_STUB_TYPES.contains(&k.as_str()) ||
                    k.contains('-') && !["stub_type", "stub_form", "stub_origin", "inline_anchor", "inline_anchors", "example-needed", "citation-needed"].contains(&k.as_str())
                });

                if let Some((stub_type_key, value)) = compact_key {
                    // Compact format: type as key
                    let stub_type = StubType::new(stub_type_key.clone());

                    // Value can be a string (description) or an object with more fields
                    match value {
                        serde_yaml::Value::String(desc) => {
                            // Simple compact: `- verify: "description"`
                            return Ok(Stub::compact(stub_type_key.clone(), desc.clone()));
                        }
                        serde_yaml::Value::Mapping(obj) => {
                            // Compact with object: `- verify: { description: "...", ... }`
                            let description = obj.get(&serde_yaml::Value::String("description".to_string()))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let stub_form = obj.get(&serde_yaml::Value::String("stub_form".to_string()))
                                .and_then(|v| v.as_str())
                                .and_then(|s| StubForm::from_str(s).ok())
                                .unwrap_or_default();

                            let priority = obj.get(&serde_yaml::Value::String("priority".to_string()))
                                .and_then(|v| v.as_str())
                                .and_then(|s| Priority::from_str(s).ok())
                                .unwrap_or_default();

                            let origin = obj.get(&serde_yaml::Value::String("stub_origin".to_string()))
                                .or_else(|| obj.get(&serde_yaml::Value::String("origin".to_string())))
                                .and_then(|v| v.as_str())
                                .and_then(|s| StubOrigin::from_str(s).ok())
                                .unwrap_or_default();

                            let inline_anchors = parse_string_or_vec(&obj, "inline_anchor")
                                .or_else(|| parse_string_or_vec(&obj, "inline_anchors"))
                                .unwrap_or_default();

                            let anchor = obj.get(&serde_yaml::Value::String("anchor".to_string()))
                                .and_then(|v| v.as_str())
                                .map(String::from);

                            return Ok(Stub {
                                stub_type,
                                description,
                                stub_form,
                                priority,
                                origin,
                                anchor,
                                inline_anchors,
                                assignees: Vec::new(),
                                participants: Vec::new(),
                                references: Vec::new(),
                                dependencies: Vec::new(),
                                urgency: None,
                                impact: None,
                                complexity: None,
                            });
                        }
                        _ => {
                            return Err(de::Error::custom(format!(
                                "Invalid compact stub format for type '{}'",
                                stub_type_key
                            )));
                        }
                    }
                }

                // Expanded or legacy format
                let stub_type_str = if has_stub_type {
                    entries.iter()
                        .find(|(k, _)| k == "stub_type")
                        .and_then(|(_, v)| v.as_str())
                        .unwrap_or("todo")
                } else if has_type {
                    entries.iter()
                        .find(|(k, _)| k == "type")
                        .and_then(|(_, v)| v.as_str())
                        .unwrap_or("todo")
                } else {
                    // No type specified, default to "todo"
                    // This handles stubs that only have gap_id, description, etc.
                    "todo"
                };

                let description = entries.iter()
                    .find(|(k, _)| k == "description")
                    .and_then(|(_, v)| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let stub_form = entries.iter()
                    .find(|(k, _)| k == "stub_form")
                    .and_then(|(_, v)| v.as_str())
                    .and_then(|s| StubForm::from_str(s).ok())
                    .unwrap_or_default();

                let priority = entries.iter()
                    .find(|(k, _)| k == "priority")
                    .and_then(|(_, v)| v.as_str())
                    .and_then(|s| Priority::from_str(s).ok())
                    .unwrap_or_default();

                let origin = entries.iter()
                    .find(|(k, _)| k == "stub_origin" || k == "origin")
                    .and_then(|(_, v)| v.as_str())
                    .and_then(|s| StubOrigin::from_str(s).ok())
                    .unwrap_or_default();

                let anchor = entries.iter()
                    .find(|(k, _)| k == "anchor")
                    .and_then(|(_, v)| v.as_str())
                    .map(String::from);

                let inline_anchors = entries.iter()
                    .find(|(k, _)| k == "inline_anchors" || k == "inline_anchor")
                    .map(|(_, v)| {
                        if let Some(s) = v.as_str() {
                            vec![s.to_string()]
                        } else if let Some(seq) = v.as_sequence() {
                            seq.iter().filter_map(|v| v.as_str().map(String::from)).collect()
                        } else {
                            Vec::new()
                        }
                    })
                    .unwrap_or_default();

                Ok(Stub {
                    stub_type: StubType::new(stub_type_str),
                    description,
                    stub_form,
                    priority,
                    origin,
                    anchor,
                    inline_anchors,
                    assignees: Vec::new(),
                    participants: Vec::new(),
                    references: Vec::new(),
                    dependencies: Vec::new(),
                    urgency: None,
                    impact: None,
                    complexity: None,
                })
            }
        }

        deserializer.deserialize_map(StubVisitor)
    }
}

/// Helper to parse a field that can be a string or a list of strings
fn parse_string_or_vec(map: &serde_yaml::Mapping, key: &str) -> Option<Vec<String>> {
    let value = map.get(&serde_yaml::Value::String(key.to_string()))?;
    if let Some(s) = value.as_str() {
        Some(vec![s.to_string()])
    } else if let Some(seq) = value.as_sequence() {
        Some(seq.iter().filter_map(|v| v.as_str().map(String::from)).collect())
    } else {
        None
    }
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
    fn test_stub_form_penalties() {
        assert_eq!(StubForm::Transient.refinement_penalty(), 0.02);
        assert_eq!(StubForm::Persistent.refinement_penalty(), 0.05);
        assert_eq!(StubForm::Blocking.refinement_penalty(), 0.10);
        assert_eq!(StubForm::Structural.refinement_penalty(), 0.15);
    }

    #[test]
    fn test_stub_form_is_blocking() {
        assert!(!StubForm::Transient.is_blocking());
        assert!(!StubForm::Persistent.is_blocking());
        assert!(StubForm::Blocking.is_blocking());
        assert!(StubForm::Structural.is_blocking());
    }

    #[test]
    fn test_stub_form_from_str() {
        assert_eq!(StubForm::from_str("transient").unwrap(), StubForm::Transient);
        assert_eq!(StubForm::from_str("BLOCKING").unwrap(), StubForm::Blocking);
        assert!(StubForm::from_str("unknown").is_err());
    }

    #[test]
    fn test_priority_urgency() {
        assert_eq!(Priority::Low.urgency(), 0.25);
        assert_eq!(Priority::Medium.urgency(), 0.50);
        assert_eq!(Priority::High.urgency(), 0.75);
        assert_eq!(Priority::Critical.urgency(), 1.0);
    }

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
    fn test_stub_is_blocking() {
        let mut stub = Stub::compact("link", "test");
        assert!(!stub.is_blocking());

        stub.stub_form = StubForm::Blocking;
        assert!(stub.is_blocking());
    }

    #[test]
    fn test_effective_values() {
        let mut stub = Stub::compact("link", "test");

        // Defaults
        assert_eq!(stub.effective_urgency(), Priority::Low.urgency());
        assert_eq!(stub.effective_impact(), 0.5);
        assert_eq!(stub.effective_complexity(), 0.5);

        // Explicit values
        stub.urgency = Some(0.8);
        stub.impact = Some(0.9);
        stub.complexity = Some(0.3);

        assert_eq!(stub.effective_urgency(), 0.8);
        assert_eq!(stub.effective_impact(), 0.9);
        assert_eq!(stub.effective_complexity(), 0.3);
    }

    // ========================================================================
    // Deserialization tests for J-Editorial stub formats
    // ========================================================================

    #[test]
    fn test_deserialize_compact_format() {
        // Compact format: type as key, description as value
        let yaml = r#"verify: "References ADR-0023—verify this document exists.""#;
        let stub: Stub = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(stub.stub_type.as_str(), "verify");
        assert_eq!(stub.description, "References ADR-0023—verify this document exists.");
        assert_eq!(stub.stub_form, StubForm::Transient);
    }

    #[test]
    fn test_deserialize_compact_with_object() {
        // Compact with object: type as key, object with fields as value
        let yaml = r#"
citation-needed:
  description: "Need source for claim"
  inline_anchor: "^cite-123"
  stub_form: blocking
"#;
        let stub: Stub = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(stub.stub_type.as_str(), "citation-needed");
        assert_eq!(stub.description, "Need source for claim");
        assert_eq!(stub.stub_form, StubForm::Blocking);
        assert_eq!(stub.inline_anchors, vec!["^cite-123"]);
    }

    #[test]
    fn test_deserialize_expanded_format() {
        // Expanded format: stub_type field
        let yaml = r#"
stub_type: verify
description: "Check source validity"
stub_origin: qa-detected
stub_form: persistent
"#;
        let stub: Stub = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(stub.stub_type.as_str(), "verify");
        assert_eq!(stub.description, "Check source validity");
        assert_eq!(stub.origin, StubOrigin::QaDetected);
        assert_eq!(stub.stub_form, StubForm::Persistent);
    }

    #[test]
    fn test_deserialize_legacy_format() {
        // Legacy format: type field
        let yaml = r#"
type: link
description: "Citation needed"
priority: high
"#;
        let stub: Stub = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(stub.stub_type.as_str(), "link");
        assert_eq!(stub.description, "Citation needed");
        assert_eq!(stub.priority, Priority::High);
    }

    #[test]
    fn test_deserialize_all_stub_types() {
        // Test that all known stub types work in compact format
        for stub_type in KNOWN_STUB_TYPES {
            let yaml = format!("{}: \"test description\"", stub_type);
            let stub: Stub = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(stub.stub_type.as_str(), *stub_type);
            assert_eq!(stub.description, "test description");
        }
    }

    #[test]
    fn test_deserialize_stub_list() {
        // Test parsing a list of stubs in mixed formats
        let yaml = r#"
- verify: "Check reference"
- citation-needed:
    description: "Need source"
    stub_form: blocking
- type: expand
  description: "Add more detail"
  priority: medium
"#;
        let stubs: Vec<Stub> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(stubs.len(), 3);
        assert_eq!(stubs[0].stub_type.as_str(), "verify");
        assert_eq!(stubs[1].stub_type.as_str(), "citation-needed");
        assert_eq!(stubs[1].stub_form, StubForm::Blocking);
        assert_eq!(stubs[2].stub_type.as_str(), "expand");
        assert_eq!(stubs[2].priority, Priority::Medium);
    }
}
