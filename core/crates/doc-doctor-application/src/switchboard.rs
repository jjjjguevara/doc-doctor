//! Application Switchboard
//!
//! Central routing mechanism for all Doc-Doctor operations.
//! Consumers (CLI, MCP, WASM) call switchboard methods instead of
//! implementing their own adapters.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                       CONSUMERS                              │
//! │   ┌──────┐    ┌──────┐    ┌──────┐    ┌────────────────┐   │
//! │   │ CLI  │    │ MCP  │    │ WASM │    │ Obsidian Plugin│   │
//! │   └──┬───┘    └──┬───┘    └──┬───┘    └───────┬────────┘   │
//! └──────┼───────────┼───────────┼────────────────┼─────────────┘
//!        │           │           │                │
//!        └───────────┴─────┬─────┴────────────────┘
//!                          │
//!            ┌─────────────▼─────────────────┐
//!            │    APPLICATION SWITCHBOARD    │
//!            │                               │
//!            │  • parse_document             │
//!            │  • analyze_document           │
//!            │  • add_stub                   │
//!            │  • resolve_stub               │
//!            │  • calculate_health           │
//!            │  ...                          │
//!            └─────────────┬─────────────────┘
//!                          │
//!               ┌──────────▼──────────┐
//!               │       DOMAIN        │
//!               │  (pure functions)   │
//!               └─────────────────────┘
//! ```

use std::sync::Arc;

use doc_doctor_domain::{
    calculate_health, calculate_usefulness, Audience, DocumentAnalysis, DocumentParser,
    DocumentWriter, L1Properties, ParseError, SchemaProvider, SerializeError, StateDimensions,
    Stub, StubContext, StubForm, StubType, Usefulness, ValidationResult, VectorPhysics,
};

use crate::error::ApplicationError;
use crate::use_cases::{AnalyzeDocumentUseCase, ValidateDocumentUseCase};

// ═══════════════════════════════════════════════════════════════════════════
//                          SWITCHBOARD TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Error type for switchboard operations
#[derive(Debug, Clone)]
pub enum SwitchboardError {
    /// Parsing error
    Parse(String),
    /// Serialization error
    Serialize(String),
    /// Validation error
    Validation(String),
    /// Stub operation error
    StubOperation(String),
    /// Generic operation error
    Operation(String),
}

impl std::fmt::Display for SwitchboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(msg) => write!(f, "Parse error: {}", msg),
            Self::Serialize(msg) => write!(f, "Serialize error: {}", msg),
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::StubOperation(msg) => write!(f, "Stub operation error: {}", msg),
            Self::Operation(msg) => write!(f, "Operation error: {}", msg),
        }
    }
}

impl std::error::Error for SwitchboardError {}

impl From<ParseError> for SwitchboardError {
    fn from(e: ParseError) -> Self {
        Self::Parse(e.to_string())
    }
}

impl From<SerializeError> for SwitchboardError {
    fn from(e: SerializeError) -> Self {
        Self::Serialize(e.to_string())
    }
}

impl From<ApplicationError> for SwitchboardError {
    fn from(e: ApplicationError) -> Self {
        Self::Operation(e.to_string())
    }
}

/// Input for creating a new stub
#[derive(Debug, Clone)]
pub struct NewStub {
    /// Stub type (e.g., "expand", "link", "verify")
    pub stub_type: String,
    /// Description of the stub
    pub description: String,
    /// Optional priority
    pub priority: Option<String>,
    /// Optional stub form override
    pub stub_form: Option<String>,
    /// Optional inline anchor to link
    pub anchor: Option<String>,
}

impl Default for NewStub {
    fn default() -> Self {
        Self {
            stub_type: "expand".to_string(),
            description: String::new(),
            priority: None,
            stub_form: None,
            anchor: None,
        }
    }
}

/// Result of adding a stub
#[derive(Debug, Clone)]
pub struct StubAddResult {
    /// Updated document content
    pub updated_content: String,
    /// Index of the new stub in the stubs array
    pub stub_index: usize,
    /// The created stub
    pub stub: Stub,
}

/// Result of resolving a stub
#[derive(Debug, Clone)]
pub struct StubResolveResult {
    /// Updated document content
    pub updated_content: String,
    /// The resolved stub
    pub resolved_stub: Stub,
}

/// Updates to apply to a stub
#[derive(Debug, Clone, Default)]
pub struct StubUpdates {
    /// New description
    pub description: Option<String>,
    /// New priority
    pub priority: Option<String>,
    /// New stub form
    pub stub_form: Option<String>,
}

/// Result of updating a stub
#[derive(Debug, Clone)]
pub struct StubUpdateResult {
    /// Updated document content
    pub updated_content: String,
    /// The updated stub
    pub stub: Stub,
}

/// Result of linking a stub to an anchor
#[derive(Debug, Clone)]
pub struct AnchorLinkResult {
    /// Updated document content
    pub updated_content: String,
    /// The stub with updated anchors
    pub stub: Stub,
}

/// Anchor matches found in document content
#[derive(Debug, Clone)]
pub struct AnchorMatches {
    /// List of anchors found (anchor_id, line_number)
    pub anchors: Vec<(String, usize)>,
    /// Stubs with their matched anchors
    pub stub_anchors: Vec<(usize, Vec<String>)>,
}

/// Filter options for listing stubs
#[derive(Debug, Clone, Default)]
pub struct StubFilter {
    /// Filter by stub type
    pub stub_type: Option<String>,
    /// Only show blocking stubs
    pub blocking_only: bool,
    /// Filter by priority
    pub priority: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//                          SWITCHBOARD TRAIT
// ═══════════════════════════════════════════════════════════════════════════

/// Central routing mechanism for all Doc-Doctor operations.
///
/// Consumers (CLI, MCP, WASM) call switchboard methods instead of
/// implementing their own adapters. This ensures:
///
/// - Single source of truth for all operations
/// - Consistent behavior across all interfaces
/// - Centralized auditing and logging
/// - Easy testability via mocking
pub trait Switchboard: Send + Sync {
    // ═══════════════════════════════════════════════════════════════
    //                     ANALYSIS OPERATIONS
    // ═══════════════════════════════════════════════════════════════

    /// Parse document content, extract L1 properties
    fn parse_document(&self, content: &str) -> Result<L1Properties, SwitchboardError>;

    /// Full analysis: parse + L2 dimensions
    fn analyze_document(&self, content: &str) -> Result<DocumentAnalysis, SwitchboardError>;

    /// Validate frontmatter against schema
    fn validate_document(
        &self,
        content: &str,
        strict: bool,
    ) -> Result<ValidationResult, SwitchboardError>;

    /// List stubs with optional filters
    fn list_stubs(
        &self,
        content: &str,
        filter: Option<StubFilter>,
    ) -> Result<Vec<Stub>, SwitchboardError>;

    /// Find stub anchors in content
    fn find_stub_anchors(&self, content: &str) -> Result<AnchorMatches, SwitchboardError>;

    // ═══════════════════════════════════════════════════════════════
    //                     STUB MANAGEMENT
    // ═══════════════════════════════════════════════════════════════

    /// Add a stub to document frontmatter
    fn add_stub(&self, content: &str, stub: NewStub) -> Result<StubAddResult, SwitchboardError>;

    /// Remove a resolved stub
    fn resolve_stub(
        &self,
        content: &str,
        stub_index: usize,
    ) -> Result<StubResolveResult, SwitchboardError>;

    /// Update stub properties
    fn update_stub(
        &self,
        content: &str,
        stub_index: usize,
        updates: StubUpdates,
    ) -> Result<StubUpdateResult, SwitchboardError>;

    /// Link stub to inline anchor
    fn link_stub_anchor(
        &self,
        content: &str,
        stub_index: usize,
        anchor_id: &str,
    ) -> Result<AnchorLinkResult, SwitchboardError>;

    /// Unlink stub from inline anchor
    fn unlink_stub_anchor(
        &self,
        content: &str,
        stub_index: usize,
        anchor_id: &str,
    ) -> Result<AnchorLinkResult, SwitchboardError>;

    // ═══════════════════════════════════════════════════════════════
    //                     CALCULATIONS
    // ═══════════════════════════════════════════════════════════════

    /// Calculate health score
    fn calc_health(&self, refinement: f64, stubs: &[Stub]) -> f64;

    /// Calculate usefulness margin
    fn calc_usefulness(&self, refinement: f64, audience: Audience) -> Usefulness;

    /// Calculate all L2 dimensions from parsed properties
    fn calc_dimensions(&self, props: &L1Properties) -> StateDimensions;

    /// Calculate vector physics for stub prioritization
    fn calc_vector_physics(&self, stub: &Stub, context: &StubContext) -> VectorPhysics;

    // ═══════════════════════════════════════════════════════════════
    //                     INFO/SCHEMA
    // ═══════════════════════════════════════════════════════════════

    /// Get JSON schema for frontmatter
    fn get_frontmatter_schema(&self) -> &str;

    /// Get JSON schema for stubs
    fn get_stubs_schema(&self) -> &str;
}

// ═══════════════════════════════════════════════════════════════════════════
//                       SWITCHBOARD IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════

/// Application switchboard implementation
///
/// Coordinates all Doc-Doctor operations through domain and outbound ports.
pub struct ApplicationSwitchboard<P, W, S>
where
    P: DocumentParser,
    W: DocumentWriter,
    S: SchemaProvider,
{
    parser: Arc<P>,
    writer: Arc<W>,
    schema_provider: Arc<S>,
    analyze_use_case: AnalyzeDocumentUseCase,
    validate_use_case: ValidateDocumentUseCase,
}

impl<P, W, S> ApplicationSwitchboard<P, W, S>
where
    P: DocumentParser + 'static,
    W: DocumentWriter + 'static,
    S: SchemaProvider + 'static,
{
    /// Create a new application switchboard
    pub fn new(parser: Arc<P>, writer: Arc<W>, schema_provider: Arc<S>) -> Self {
        let parser_dyn: Arc<dyn DocumentParser> = Arc::clone(&parser) as Arc<dyn DocumentParser>;
        let schema_dyn: Arc<dyn SchemaProvider> = Arc::clone(&schema_provider) as Arc<dyn SchemaProvider>;

        Self {
            parser: Arc::clone(&parser),
            writer,
            schema_provider: Arc::clone(&schema_provider),
            analyze_use_case: AnalyzeDocumentUseCase::new(parser_dyn),
            validate_use_case: ValidateDocumentUseCase::new(
                Arc::clone(&parser) as Arc<dyn DocumentParser>,
                schema_dyn,
            ),
        }
    }
}

impl<P, W, S> Switchboard for ApplicationSwitchboard<P, W, S>
where
    P: DocumentParser + 'static,
    W: DocumentWriter + 'static,
    S: SchemaProvider + 'static,
{
    fn parse_document(&self, content: &str) -> Result<L1Properties, SwitchboardError> {
        self.parser.parse(content).map_err(SwitchboardError::from)
    }

    fn analyze_document(&self, content: &str) -> Result<DocumentAnalysis, SwitchboardError> {
        use doc_doctor_domain::AnalyzeDocument;
        self.analyze_use_case
            .analyze(content)
            .map_err(|e| SwitchboardError::Operation(e.to_string()))
    }

    fn validate_document(
        &self,
        content: &str,
        strict: bool,
    ) -> Result<ValidationResult, SwitchboardError> {
        use doc_doctor_domain::ValidateDocument;
        self.validate_use_case
            .validate(content, strict)
            .map_err(|e| SwitchboardError::Validation(e.to_string()))
    }

    fn list_stubs(
        &self,
        content: &str,
        filter: Option<StubFilter>,
    ) -> Result<Vec<Stub>, SwitchboardError> {
        let props = self.parser.parse(content)?;
        let mut stubs = props.stubs;

        if let Some(f) = filter {
            if let Some(stub_type) = f.stub_type {
                stubs.retain(|s| s.stub_type.as_str() == stub_type);
            }
            if f.blocking_only {
                stubs.retain(|s| matches!(s.stub_form, StubForm::Blocking));
            }
            if let Some(priority) = f.priority {
                stubs.retain(|s| s.priority.to_string().to_lowercase() == priority.to_lowercase());
            }
        }

        Ok(stubs)
    }

    fn find_stub_anchors(&self, content: &str) -> Result<AnchorMatches, SwitchboardError> {
        let props = self.parser.parse(content)?;

        // Find all ^anchor patterns in content
        let mut anchors = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let mut pos = 0;
            while let Some(start) = line[pos..].find("^") {
                let anchor_start = pos + start + 1;
                // Find end of anchor (alphanumeric + hyphens)
                let anchor_end = line[anchor_start..]
                    .find(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                    .map(|i| anchor_start + i)
                    .unwrap_or(line.len());

                if anchor_end > anchor_start {
                    let anchor_id = &line[anchor_start..anchor_end];
                    anchors.push((anchor_id.to_string(), line_num + 1));
                }
                pos = anchor_end;
            }
        }

        // Match stubs to anchors
        let stub_anchors: Vec<(usize, Vec<String>)> = props
            .stubs
            .iter()
            .enumerate()
            .map(|(i, stub)| {
                let matched: Vec<String> = stub
                    .inline_anchors
                    .iter()
                    .filter(|a| anchors.iter().any(|(id, _)| id == *a))
                    .cloned()
                    .collect();
                (i, matched)
            })
            .collect();

        Ok(AnchorMatches {
            anchors,
            stub_anchors,
        })
    }

    fn add_stub(&self, content: &str, new_stub: NewStub) -> Result<StubAddResult, SwitchboardError> {
        let mut props = self.parser.parse(content)?;

        // Create the stub
        let stub = Stub {
            stub_type: StubType::new(&new_stub.stub_type),
            description: new_stub.description.clone(),
            priority: new_stub
                .priority
                .as_ref()
                .map(|p| p.parse().unwrap_or_default())
                .unwrap_or_default(),
            stub_form: new_stub
                .stub_form
                .as_ref()
                .map(|f| f.parse().unwrap_or_default())
                .unwrap_or_default(),
            inline_anchors: new_stub.anchor.into_iter().collect(),
            ..Default::default()
        };

        let stub_index = props.stubs.len();
        props.stubs.push(stub.clone());

        // Serialize back
        let updated_content = self.writer.serialize_document(content, &props)?;

        Ok(StubAddResult {
            updated_content,
            stub_index,
            stub,
        })
    }

    fn resolve_stub(
        &self,
        content: &str,
        stub_index: usize,
    ) -> Result<StubResolveResult, SwitchboardError> {
        let mut props = self.parser.parse(content)?;

        if stub_index >= props.stubs.len() {
            return Err(SwitchboardError::StubOperation(format!(
                "Stub index {} out of range (max: {})",
                stub_index,
                props.stubs.len()
            )));
        }

        let resolved_stub = props.stubs.remove(stub_index);
        let updated_content = self.writer.serialize_document(content, &props)?;

        Ok(StubResolveResult {
            updated_content,
            resolved_stub,
        })
    }

    fn update_stub(
        &self,
        content: &str,
        stub_index: usize,
        updates: StubUpdates,
    ) -> Result<StubUpdateResult, SwitchboardError> {
        let mut props = self.parser.parse(content)?;

        if stub_index >= props.stubs.len() {
            return Err(SwitchboardError::StubOperation(format!(
                "Stub index {} out of range (max: {})",
                stub_index,
                props.stubs.len()
            )));
        }

        let stub = &mut props.stubs[stub_index];

        if let Some(desc) = updates.description {
            stub.description = desc;
        }
        if let Some(priority) = updates.priority {
            stub.priority = priority.parse().unwrap_or_default();
        }
        if let Some(form) = updates.stub_form {
            stub.stub_form = form.parse().unwrap_or_default();
        }

        let updated_stub = stub.clone();
        let updated_content = self.writer.serialize_document(content, &props)?;

        Ok(StubUpdateResult {
            updated_content,
            stub: updated_stub,
        })
    }

    fn link_stub_anchor(
        &self,
        content: &str,
        stub_index: usize,
        anchor_id: &str,
    ) -> Result<AnchorLinkResult, SwitchboardError> {
        let mut props = self.parser.parse(content)?;

        if stub_index >= props.stubs.len() {
            return Err(SwitchboardError::StubOperation(format!(
                "Stub index {} out of range (max: {})",
                stub_index,
                props.stubs.len()
            )));
        }

        let stub = &mut props.stubs[stub_index];

        // Add anchor if not already present
        if !stub.inline_anchors.contains(&anchor_id.to_string()) {
            stub.inline_anchors.push(anchor_id.to_string());
        }

        let updated_stub = stub.clone();
        let updated_content = self.writer.serialize_document(content, &props)?;

        Ok(AnchorLinkResult {
            updated_content,
            stub: updated_stub,
        })
    }

    fn unlink_stub_anchor(
        &self,
        content: &str,
        stub_index: usize,
        anchor_id: &str,
    ) -> Result<AnchorLinkResult, SwitchboardError> {
        let mut props = self.parser.parse(content)?;

        if stub_index >= props.stubs.len() {
            return Err(SwitchboardError::StubOperation(format!(
                "Stub index {} out of range (max: {})",
                stub_index,
                props.stubs.len()
            )));
        }

        let stub = &mut props.stubs[stub_index];

        // Remove anchor if present
        stub.inline_anchors.retain(|a| a != anchor_id);

        let updated_stub = stub.clone();
        let updated_content = self.writer.serialize_document(content, &props)?;

        Ok(AnchorLinkResult {
            updated_content,
            stub: updated_stub,
        })
    }

    fn calc_health(&self, refinement: f64, stubs: &[Stub]) -> f64 {
        calculate_health(refinement, stubs)
    }

    fn calc_usefulness(&self, refinement: f64, audience: Audience) -> Usefulness {
        calculate_usefulness(refinement, audience)
    }

    fn calc_dimensions(&self, props: &L1Properties) -> StateDimensions {
        StateDimensions::calculate(props)
    }

    fn calc_vector_physics(&self, stub: &Stub, context: &StubContext) -> VectorPhysics {
        VectorPhysics::calculate(stub, context)
    }

    fn get_frontmatter_schema(&self) -> &str {
        self.schema_provider.frontmatter_schema()
    }

    fn get_stubs_schema(&self) -> &str {
        self.schema_provider.stubs_schema()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_doctor_domain::EmbeddedSchemaProvider;
    use doc_doctor_parser_yaml::YamlParser;

    fn create_test_switchboard(
    ) -> ApplicationSwitchboard<YamlParser, YamlParser, EmbeddedSchemaProvider> {
        let parser = Arc::new(YamlParser::new());
        let writer = Arc::clone(&parser);
        let schema = Arc::new(EmbeddedSchemaProvider);
        ApplicationSwitchboard::new(parser, writer, schema)
    }

    #[test]
    fn test_parse_document() {
        let switchboard = create_test_switchboard();
        let content = "---\ntitle: Test\nrefinement: 0.75\n---\n# Content";

        let props = switchboard.parse_document(content).unwrap();
        assert_eq!(props.title.as_deref(), Some("Test"));
        assert_eq!(props.refinement.value(), 0.75);
    }

    #[test]
    fn test_add_stub() {
        let switchboard = create_test_switchboard();
        let content = "---\ntitle: Test\nrefinement: 0.5\n---\n# Content";

        let result = switchboard
            .add_stub(
                content,
                NewStub {
                    stub_type: "expand".to_string(),
                    description: "Add more details".to_string(),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(result.stub_index, 0);
        assert!(result.updated_content.contains("stubs:"));
        assert!(result.updated_content.contains("expand"));
    }

    #[test]
    fn test_resolve_stub() {
        let switchboard = create_test_switchboard();
        let content = r#"---
title: Test
stubs:
  - type: expand
    description: "Test stub"
---
# Content"#;

        let result = switchboard.resolve_stub(content, 0).unwrap();
        assert!(!result.updated_content.contains("expand"));
    }

    #[test]
    fn test_calc_health() {
        let switchboard = create_test_switchboard();
        let health = switchboard.calc_health(0.8, &[]);
        assert!(health > 0.5);
    }
}
