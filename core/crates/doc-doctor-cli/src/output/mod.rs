//! Output Formatting
//!
//! Supports multiple output formats: human-readable, JSON, YAML.

use serde::Serialize;

/// Output format options
#[derive(Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable output
    #[default]
    Human,
    /// JSON output
    Json,
    /// YAML output
    Yaml,
}

/// Format output based on the selected format
pub fn format_output<T: Serialize + HumanReadable>(
    data: &T,
    format: OutputFormat,
) -> anyhow::Result<String> {
    match format {
        OutputFormat::Human => Ok(data.to_human()),
        OutputFormat::Json => {
            serde_json::to_string_pretty(data).map_err(|e| anyhow::anyhow!("JSON error: {}", e))
        }
        OutputFormat::Yaml => {
            serde_yaml::to_string(data).map_err(|e| anyhow::anyhow!("YAML error: {}", e))
        }
    }
}

/// Trait for types that can be rendered as human-readable output
pub trait HumanReadable {
    fn to_human(&self) -> String;
}

/// Simple string wrapper for basic messages
#[derive(Serialize)]
pub struct Message {
    pub message: String,
}

impl HumanReadable for Message {
    fn to_human(&self) -> String {
        self.message.clone()
    }
}

impl Message {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

/// Parse result for CLI output
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseOutput {
    pub path: String,
    pub title: Option<String>,
    pub refinement: f64,
    pub audience: String,
    pub origin: String,
    pub form: String,
    pub stub_count: usize,
    pub tags: Vec<String>,
}

impl HumanReadable for ParseOutput {
    fn to_human(&self) -> String {
        let mut lines = vec![
            format!("File: {}", self.path),
            format!("Title: {}", self.title.as_deref().unwrap_or("(none)")),
            format!("Refinement: {:.2}", self.refinement),
            format!("Audience: {}", self.audience),
            format!("Origin: {}", self.origin),
            format!("Form: {}", self.form),
            format!("Stubs: {}", self.stub_count),
        ];

        if !self.tags.is_empty() {
            lines.push(format!("Tags: {}", self.tags.join(", ")));
        }

        lines.join("\n")
    }
}

/// Dimensions result for CLI output
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionsOutput {
    pub path: String,
    pub health: f64,
    pub usefulness_margin: f64,
    pub is_useful: bool,
    pub trust_level: f64,
    pub freshness: f64,
}

impl HumanReadable for DimensionsOutput {
    fn to_human(&self) -> String {
        let useful_str = if self.is_useful { "Yes" } else { "No" };
        vec![
            format!("File: {}", self.path),
            format!("Health: {:.4}", self.health),
            format!("Usefulness Margin: {:.4}", self.usefulness_margin),
            format!("Is Useful: {}", useful_str),
            format!("Trust Level: {:.4}", self.trust_level),
            format!("Freshness: {:.4}", self.freshness),
        ]
        .join("\n")
    }
}

/// Health calculation result
#[derive(Serialize)]
pub struct HealthOutput {
    pub health: f64,
    pub refinement: f64,
    pub stub_count: usize,
    pub stub_penalty: f64,
}

impl HumanReadable for HealthOutput {
    fn to_human(&self) -> String {
        vec![
            format!("Health: {:.4}", self.health),
            format!("Refinement: {:.2}", self.refinement),
            format!("Stub Count: {}", self.stub_count),
            format!("Stub Penalty: {:.4}", self.stub_penalty),
        ]
        .join("\n")
    }
}

/// Usefulness calculation result
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsefulnessOutput {
    pub margin: f64,
    pub is_useful: bool,
    pub refinement: f64,
    pub audience: String,
    pub gate: f64,
}

impl HumanReadable for UsefulnessOutput {
    fn to_human(&self) -> String {
        let useful_str = if self.is_useful { "Yes" } else { "No" };
        vec![
            format!("Margin: {:.4}", self.margin),
            format!("Is Useful: {}", useful_str),
            format!("Refinement: {:.2}", self.refinement),
            format!("Audience: {}", self.audience),
            format!("Gate: {:.2}", self.gate),
        ]
        .join("\n")
    }
}

/// Validation result
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationOutput {
    pub path: String,
    pub is_valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl HumanReadable for ValidationOutput {
    fn to_human(&self) -> String {
        let mut lines = vec![
            format!("File: {}", self.path),
            format!("Valid: {}", if self.is_valid { "Yes" } else { "No" }),
        ];

        if !self.errors.is_empty() {
            lines.push(format!("Errors ({}):", self.error_count));
            for err in &self.errors {
                lines.push(format!("  - {}", err));
            }
        }

        if !self.warnings.is_empty() {
            lines.push(format!("Warnings ({}):", self.warning_count));
            for warn in &self.warnings {
                lines.push(format!("  - {}", warn));
            }
        }

        lines.join("\n")
    }
}

/// Batch processing result
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchOutput {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub average_health: Option<f64>,
    pub results: Vec<BatchDocumentOutput>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDocumentOutput {
    pub path: String,
    pub success: bool,
    pub health: Option<f64>,
    pub error: Option<String>,
}

impl HumanReadable for BatchOutput {
    fn to_human(&self) -> String {
        let mut lines = vec![
            format!("Total: {}", self.total),
            format!("Succeeded: {}", self.succeeded),
            format!("Failed: {}", self.failed),
        ];

        if let Some(avg) = self.average_health {
            lines.push(format!("Average Health: {:.4}", avg));
        }

        if self.failed > 0 {
            lines.push("\nFailed documents:".to_string());
            for doc in &self.results {
                if !doc.success {
                    lines.push(format!(
                        "  - {}: {}",
                        doc.path,
                        doc.error.as_deref().unwrap_or("Unknown error")
                    ));
                }
            }
        }

        lines.join("\n")
    }
}

/// Stubs list output
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StubsOutput {
    pub path: String,
    pub total: usize,
    pub stubs: Vec<StubOutput>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StubOutput {
    pub stub_type: String,
    pub description: String,
    pub stub_form: String,
    pub priority: String,
    pub is_blocking: bool,
}

impl HumanReadable for StubsOutput {
    fn to_human(&self) -> String {
        let mut lines = vec![
            format!("File: {}", self.path),
            format!("Total Stubs: {}", self.total),
        ];

        if !self.stubs.is_empty() {
            lines.push("\nStubs:".to_string());
            for (i, stub) in self.stubs.iter().enumerate() {
                let blocking = if stub.is_blocking { " [BLOCKING]" } else { "" };
                lines.push(format!(
                    "  {}. [{}] {} ({}){}",
                    i + 1,
                    stub.stub_type,
                    stub.description,
                    stub.stub_form,
                    blocking
                ));
            }
        }

        lines.join("\n")
    }
}

/// Schema output
#[derive(Serialize)]
pub struct SchemaOutput {
    pub schema_type: String,
    pub schema: serde_json::Value,
}

impl HumanReadable for SchemaOutput {
    fn to_human(&self) -> String {
        format!(
            "Schema: {}\n\n{}",
            self.schema_type,
            serde_json::to_string_pretty(&self.schema).unwrap_or_default()
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//                         STUB OPERATION OUTPUTS
// ═══════════════════════════════════════════════════════════════════════════

/// Result of adding a stub
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StubAddOutput {
    pub action: String,
    pub path: String,
    pub stub_index: usize,
    pub stub_type: String,
    pub description: String,
}

impl HumanReadable for StubAddOutput {
    fn to_human(&self) -> String {
        format!(
            "Added stub #{} to {}\n  Type: {}\n  Description: {}",
            self.stub_index, self.path, self.stub_type, self.description
        )
    }
}

/// Result of resolving a stub
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StubResolveOutput {
    pub action: String,
    pub path: String,
    pub resolved_type: String,
    pub resolved_description: String,
}

impl HumanReadable for StubResolveOutput {
    fn to_human(&self) -> String {
        format!(
            "Resolved stub in {}\n  Type: {}\n  Description: {}",
            self.path, self.resolved_type, self.resolved_description
        )
    }
}

/// Result of updating a stub
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StubUpdateOutput {
    pub action: String,
    pub path: String,
    pub stub_type: String,
    pub description: String,
    pub priority: String,
    pub stub_form: String,
}

impl HumanReadable for StubUpdateOutput {
    fn to_human(&self) -> String {
        format!(
            "Updated stub in {}\n  Type: {}\n  Description: {}\n  Priority: {}\n  Form: {}",
            self.path, self.stub_type, self.description, self.priority, self.stub_form
        )
    }
}

/// Result of linking a stub to an anchor
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StubLinkOutput {
    pub action: String,
    pub path: String,
    pub stub_index: usize,
    pub anchor_id: String,
}

impl HumanReadable for StubLinkOutput {
    fn to_human(&self) -> String {
        format!(
            "Linked stub #{} to anchor ^{} in {}",
            self.stub_index, self.anchor_id, self.path
        )
    }
}

/// Anchors found in a document
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorsOutput {
    pub path: String,
    pub anchors: Vec<AnchorInfo>,
    pub stub_anchors: Vec<StubAnchorInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorInfo {
    pub id: String,
    pub line: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StubAnchorInfo {
    pub stub_index: usize,
    pub anchors: Vec<String>,
}

impl HumanReadable for AnchorsOutput {
    fn to_human(&self) -> String {
        let mut lines = vec![format!("File: {}", self.path)];

        if !self.anchors.is_empty() {
            lines.push(format!("\nAnchors found ({}):", self.anchors.len()));
            for anchor in &self.anchors {
                lines.push(format!("  ^{} (line {})", anchor.id, anchor.line));
            }
        }

        if !self.stub_anchors.is_empty() {
            lines.push("\nStub anchor mappings:".to_string());
            for mapping in &self.stub_anchors {
                if !mapping.anchors.is_empty() {
                    lines.push(format!(
                        "  Stub #{}: {}",
                        mapping.stub_index,
                        mapping.anchors.iter().map(|a| format!("^{}", a)).collect::<Vec<_>>().join(", ")
                    ));
                }
            }
        }

        lines.join("\n")
    }
}
