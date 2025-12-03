//! WASM Bindings Implementation
//!
//! Exposes Doc-Doctor functionality to JavaScript via wasm-bindgen.

use wasm_bindgen::prelude::*;
use std::sync::Arc;

use doc_doctor_application::{AnalyzeDocumentUseCase, ValidateDocumentUseCase};
use doc_doctor_domain::{
    calculate_health, calculate_usefulness, Audience, AnalyzeDocument, DocumentParser,
    EmbeddedSchemaProvider, SchemaProvider, Stub, StubContext, ValidateDocument, VectorPhysics,
};
use doc_doctor_parser_yaml::YamlParser;

use crate::types::{
    AnalysisResult, AudienceGatesJson, DimensionsJson, PropertiesJson, StubJson,
    UsefulnessJson, ValidationErrorJson, ValidationResultJson, ValidationWarningJson,
    VectorPhysicsJson,
};

/// Main Doc-Doctor WASM interface
///
/// Provides access to all document analysis functionality.
#[wasm_bindgen]
pub struct DocDoctor {
    parser: Arc<YamlParser>,
    schema_provider: Arc<EmbeddedSchemaProvider>,
}

#[wasm_bindgen]
impl DocDoctor {
    /// Create a new DocDoctor instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            parser: Arc::new(YamlParser::new()),
            schema_provider: Arc::new(EmbeddedSchemaProvider),
        }
    }

    /// Parse a markdown document and return L1 properties as JSON
    ///
    /// # Arguments
    /// * `content` - Markdown document content with YAML frontmatter
    ///
    /// # Returns
    /// JSON string with parsed properties or error
    #[wasm_bindgen(js_name = parseDocument)]
    pub fn parse_document(&self, content: &str) -> Result<String, JsValue> {
        let props = self
            .parser
            .parse(content)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let json = PropertiesJson::from_l1(&props);
        serde_json::to_string(&json).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Analyze a document (parse + calculate dimensions)
    ///
    /// # Arguments
    /// * `content` - Markdown document content with YAML frontmatter
    ///
    /// # Returns
    /// JSON string with full analysis result
    #[wasm_bindgen(js_name = analyzeDocument)]
    pub fn analyze_document(&self, content: &str) -> String {
        let use_case = AnalyzeDocumentUseCase::new(Arc::clone(&self.parser) as Arc<dyn DocumentParser>);

        let result = match use_case.analyze(content) {
            Ok(analysis) => AnalysisResult::from_analysis(analysis),
            Err(e) => AnalysisResult::from_error(e.to_string()),
        };

        serde_json::to_string(&result).unwrap_or_else(|e| {
            format!(r#"{{"success":false,"error":"Serialization error: {}"}}"#, e)
        })
    }

    /// Validate a document against the J-Editorial schema
    ///
    /// # Arguments
    /// * `content` - Markdown document content
    /// * `strict` - Whether to reject unknown fields
    ///
    /// # Returns
    /// JSON string with validation result
    #[wasm_bindgen(js_name = validateDocument)]
    pub fn validate_document(&self, content: &str, strict: bool) -> String {
        let use_case = ValidateDocumentUseCase::new(
            Arc::clone(&self.parser) as Arc<dyn DocumentParser>,
            Arc::clone(&self.schema_provider) as Arc<dyn SchemaProvider>,
        );

        match use_case.validate(content, strict) {
            Ok(result) => {
                let json = ValidationResultJson {
                    is_valid: result.is_valid,
                    errors: result
                        .errors
                        .iter()
                        .map(|e| ValidationErrorJson {
                            message: e.message.clone(),
                            path: e.path.clone(),
                            line: e.position.map(|p| p.line),
                            column: e.position.map(|p| p.column),
                        })
                        .collect(),
                    warnings: result
                        .warnings
                        .iter()
                        .map(|w| ValidationWarningJson {
                            message: w.message.clone(),
                            path: w.path.clone(),
                            suggestion: w.suggestion.clone(),
                        })
                        .collect(),
                };
                serde_json::to_string(&json).unwrap()
            }
            Err(e) => {
                let json = ValidationResultJson {
                    is_valid: false,
                    errors: vec![ValidationErrorJson {
                        message: e.to_string(),
                        path: None,
                        line: None,
                        column: None,
                    }],
                    warnings: vec![],
                };
                serde_json::to_string(&json).unwrap()
            }
        }
    }

    /// Calculate health score
    ///
    /// # Arguments
    /// * `refinement` - Refinement score (0.0-1.0)
    /// * `stubs_json` - JSON array of stubs
    ///
    /// # Returns
    /// Health score (0.0-1.0)
    #[wasm_bindgen(js_name = calculateHealth)]
    pub fn calculate_health(&self, refinement: f64, stubs_json: &str) -> Result<f64, JsValue> {
        let stubs = self.parse_stubs(stubs_json)?;
        Ok(calculate_health(refinement, &stubs))
    }

    /// Calculate usefulness for an audience
    ///
    /// # Arguments
    /// * `refinement` - Refinement score (0.0-1.0)
    /// * `audience` - Audience string (personal, internal, trusted, public)
    ///
    /// # Returns
    /// JSON string with usefulness result
    #[wasm_bindgen(js_name = calculateUsefulness)]
    pub fn calculate_usefulness(&self, refinement: f64, audience: &str) -> Result<String, JsValue> {
        let audience: Audience = audience
            .parse()
            .map_err(|_| JsValue::from_str(&format!("Invalid audience: {}", audience)))?;

        let usefulness = calculate_usefulness(refinement, audience);
        let json = UsefulnessJson::from_usefulness(&usefulness);

        serde_json::to_string(&json).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Calculate vector physics for a stub
    ///
    /// # Arguments
    /// * `stub_json` - JSON object representing a stub
    /// * `context_json` - JSON object with context (optional velocity, dependencies, etc.)
    ///
    /// # Returns
    /// JSON string with vector physics
    #[wasm_bindgen(js_name = calculateVectorPhysics)]
    pub fn calculate_vector_physics(
        &self,
        stub_json: &str,
        context_json: &str,
    ) -> Result<String, JsValue> {
        let stub_data: StubJson =
            serde_json::from_str(stub_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

        let stub = stub_data
            .to_stub()
            .map_err(|e| JsValue::from_str(&e))?;

        let context: StubContext = if context_json.is_empty() || context_json == "{}" {
            StubContext::default()
        } else {
            serde_json::from_str(context_json).map_err(|e| JsValue::from_str(&e.to_string()))?
        };

        let physics = VectorPhysics::calculate(&stub, &context);
        let json = VectorPhysicsJson::from_physics(&physics);

        serde_json::to_string(&json).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Calculate state dimensions for properties
    ///
    /// # Arguments
    /// * `properties_json` - JSON object with L1 properties
    ///
    /// # Returns
    /// JSON string with dimensions
    #[wasm_bindgen(js_name = calculateDimensions)]
    pub fn calculate_dimensions(&self, properties_json: &str) -> Result<String, JsValue> {
        let props: doc_doctor_domain::L1Properties = serde_json::from_str(properties_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let dims = doc_doctor_domain::StateDimensions::calculate(&props);
        let json = DimensionsJson::from_state(&dims);

        serde_json::to_string(&json).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get audience gate values
    ///
    /// # Returns
    /// JSON object with gate thresholds for each audience
    #[wasm_bindgen(js_name = getAudienceGates)]
    pub fn get_audience_gates(&self) -> String {
        let gates = AudienceGatesJson::default();
        serde_json::to_string(&gates).unwrap()
    }

    /// Parse stubs from JSON array
    ///
    /// # Arguments
    /// * `stubs_json` - JSON array of stubs
    ///
    /// # Returns
    /// JSON array of parsed stubs with derived fields
    #[wasm_bindgen(js_name = parseStubs)]
    pub fn parse_stubs_json(&self, stubs_json: &str) -> Result<String, JsValue> {
        let stubs = self.parse_stubs(stubs_json)?;
        let json: Vec<StubJson> = stubs.iter().map(StubJson::from_stub).collect();
        serde_json::to_string(&json).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get the version string
    #[wasm_bindgen(js_name = version)]
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Get the parser format identifier
    #[wasm_bindgen(js_name = formatId)]
    pub fn format_id(&self) -> String {
        self.parser.format_id().to_string()
    }

    // Helper: Parse stubs from JSON
    fn parse_stubs(&self, stubs_json: &str) -> Result<Vec<Stub>, JsValue> {
        if stubs_json.is_empty() || stubs_json == "[]" {
            return Ok(Vec::new());
        }

        let stub_data: Vec<StubJson> =
            serde_json::from_str(stubs_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

        stub_data
            .into_iter()
            .map(|s| s.to_stub().map_err(|e| JsValue::from_str(&e)))
            .collect()
    }
}

impl Default for DocDoctor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_document() {
        let dd = DocDoctor::new();
        let content = "---\ntitle: Test\nrefinement: 0.75\n---\n# Content";

        let result = dd.parse_document(content).unwrap();
        assert!(result.contains("\"refinement\":0.75"));
        assert!(result.contains("\"title\":\"Test\""));
    }

    #[test]
    fn test_analyze_document() {
        let dd = DocDoctor::new();
        let content = "---\ntitle: Test\nrefinement: 0.8\naudience: internal\n---\n";

        let result = dd.analyze_document(content);
        assert!(result.contains("\"success\":true"));
        assert!(result.contains("\"health\""));
    }

    #[test]
    fn test_validate_document() {
        let dd = DocDoctor::new();
        let content = "---\ntitle: Test\n---\n";

        let result = dd.validate_document(content, false);
        assert!(result.contains("\"isValid\":true"));
    }

    #[test]
    fn test_calculate_health() {
        let dd = DocDoctor::new();
        let health = dd.calculate_health(0.8, "[]").unwrap();
        assert!(health > 0.0 && health <= 1.0);
    }

    #[test]
    fn test_calculate_usefulness() {
        let dd = DocDoctor::new();
        let result = dd.calculate_usefulness(0.9, "public").unwrap();
        assert!(result.contains("\"isUseful\":true"));
    }

    #[test]
    fn test_get_audience_gates() {
        let dd = DocDoctor::new();
        let result = dd.get_audience_gates();
        assert!(result.contains("\"personal\":0.5"));
        assert!(result.contains("\"public\":0.9"));
    }

    #[test]
    fn test_version() {
        let dd = DocDoctor::new();
        let version = dd.version();
        assert!(!version.is_empty());
    }
}
