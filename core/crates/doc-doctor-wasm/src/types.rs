//! JSON-serializable types for WASM interface
//!
//! These types provide a stable JSON interface for JavaScript consumers.

use doc_doctor_domain::{
    Audience, DocumentAnalysis, L1Properties, StateDimensions, Stub, Usefulness, VectorPhysics,
};
use serde::{Deserialize, Serialize};

/// Document analysis result (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Whether parsing succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Parsed properties
    pub properties: Option<PropertiesJson>,
    /// Calculated dimensions
    pub dimensions: Option<DimensionsJson>,
    /// Warnings
    pub warnings: Vec<String>,
}

impl AnalysisResult {
    pub fn from_analysis(analysis: DocumentAnalysis) -> Self {
        Self {
            success: true,
            error: None,
            properties: Some(PropertiesJson::from_l1(&analysis.properties)),
            dimensions: Some(DimensionsJson::from_state(&analysis.dimensions)),
            warnings: analysis
                .warnings
                .iter()
                .map(|w| w.message.clone())
                .collect(),
        }
    }

    pub fn from_error(message: String) -> Self {
        Self {
            success: false,
            error: Some(message),
            properties: None,
            dimensions: None,
            warnings: Vec::new(),
        }
    }
}

/// L1 Properties (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertiesJson {
    pub uid: Option<String>,
    pub title: Option<String>,
    pub refinement: f64,
    pub audience: String,
    pub origin: String,
    pub form: String,
    pub stubs: Vec<StubJson>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
}

impl PropertiesJson {
    pub fn from_l1(props: &L1Properties) -> Self {
        Self {
            uid: props.uid.clone(),
            title: props.title.clone(),
            refinement: props.refinement.value(),
            audience: props.audience.to_string(),
            origin: props.origin.to_string(),
            form: props.form.to_string(),
            stubs: props.stubs.iter().map(StubJson::from_stub).collect(),
            tags: props.tags.clone(),
            aliases: props.aliases.clone(),
        }
    }
}

/// Stub (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StubJson {
    #[serde(rename = "type")]
    pub stub_type: String,
    pub description: String,
    pub stub_form: String,
    pub priority: String,
    pub origin: String,
    pub anchor: Option<String>,
    pub urgency: Option<f64>,
    pub impact: Option<f64>,
    pub complexity: Option<f64>,
    pub inline_anchors: Vec<String>,
    pub assignees: Vec<String>,
    pub participants: Vec<String>,
    pub references: Vec<String>,
    pub dependencies: Vec<String>,
}

impl StubJson {
    pub fn from_stub(stub: &Stub) -> Self {
        Self {
            stub_type: stub.stub_type.as_str().to_string(),
            description: stub.description.clone(),
            stub_form: stub.stub_form.to_string(),
            priority: stub.priority.to_string(),
            origin: stub.origin.to_string(),
            anchor: stub.anchor.clone(),
            urgency: stub.urgency,
            impact: stub.impact,
            complexity: stub.complexity,
            inline_anchors: stub.inline_anchors.clone(),
            assignees: stub.assignees.clone(),
            participants: stub.participants.clone(),
            references: stub.references.clone(),
            dependencies: stub.dependencies.clone(),
        }
    }

    pub fn to_stub(&self) -> Result<Stub, String> {
        let mut stub = Stub::compact(&self.stub_type, &self.description);

        stub.stub_form = self
            .stub_form
            .parse()
            .map_err(|_| format!("Invalid stub_form: {}", self.stub_form))?;

        stub.priority = self
            .priority
            .parse()
            .map_err(|_| format!("Invalid priority: {}", self.priority))?;

        stub.origin = self
            .origin
            .parse()
            .map_err(|_| format!("Invalid origin: {}", self.origin))?;

        stub.anchor = self.anchor.clone();
        stub.urgency = self.urgency;
        stub.impact = self.impact;
        stub.complexity = self.complexity;
        stub.inline_anchors = self.inline_anchors.clone();
        stub.assignees = self.assignees.clone();
        stub.participants = self.participants.clone();
        stub.references = self.references.clone();
        stub.dependencies = self.dependencies.clone();

        Ok(stub)
    }
}

/// State dimensions (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionsJson {
    pub health: f64,
    pub usefulness: UsefulnessJson,
    pub trust_level: f64,
    pub freshness: f64,
    pub compliance_fit: f64,
    pub coverage_fit: f64,
}

impl DimensionsJson {
    pub fn from_state(dims: &StateDimensions) -> Self {
        Self {
            health: dims.health,
            usefulness: UsefulnessJson::from_usefulness(&dims.usefulness),
            trust_level: dims.trust_level,
            freshness: dims.freshness,
            compliance_fit: dims.compliance_fit,
            coverage_fit: dims.coverage_fit,
        }
    }
}

/// Usefulness (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsefulnessJson {
    pub margin: f64,
    pub is_useful: bool,
    pub audience: String,
    pub refinement: f64,
    pub gate: f64,
}

impl UsefulnessJson {
    pub fn from_usefulness(u: &Usefulness) -> Self {
        Self {
            margin: u.margin,
            is_useful: u.is_useful,
            audience: u.audience.to_string(),
            refinement: u.refinement,
            gate: u.gate,
        }
    }
}

/// Vector physics (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorPhysicsJson {
    pub potential_energy: f64,
    pub friction_coefficient: f64,
    pub editorial_velocity: f64,
    pub magnitude: f64,
}

impl VectorPhysicsJson {
    pub fn from_physics(vp: &VectorPhysics) -> Self {
        Self {
            potential_energy: vp.potential_energy,
            friction_coefficient: vp.friction_coefficient,
            editorial_velocity: vp.editorial_velocity,
            magnitude: vp.magnitude,
        }
    }
}

/// Audience gates (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudienceGatesJson {
    pub personal: f64,
    pub internal: f64,
    pub trusted: f64,
    pub public: f64,
}

impl Default for AudienceGatesJson {
    fn default() -> Self {
        Self {
            personal: Audience::Personal.gate(),
            internal: Audience::Internal.gate(),
            trusted: Audience::Trusted.gate(),
            public: Audience::Public.gate(),
        }
    }
}

/// Validation result (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultJson {
    pub is_valid: bool,
    pub errors: Vec<ValidationErrorJson>,
    pub warnings: Vec<ValidationWarningJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrorJson {
    pub message: String,
    pub path: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationWarningJson {
    pub message: String,
    pub path: Option<String>,
    pub suggestion: Option<String>,
}
