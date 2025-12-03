//! Tool Handler Implementations
//!
//! Each handler processes tool arguments and returns a JSON string result.

use std::sync::Arc;

use doc_doctor_application::{AnalyzeDocumentUseCase, ValidateDocumentUseCase};
use doc_doctor_domain::{
    AnalyzeDocument, Audience, DocumentParser, SchemaProvider, StateDimensions, Stub,
    StubContext, ValidateDocument, VectorPhysics,
};

/// Parse document and extract L1 properties
pub fn parse_document(parser: &Arc<dyn DocumentParser>, args: serde_json::Value) -> Result<String, String> {
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'content' argument")?;

    let props = parser.parse(content).map_err(|e| e.to_string())?;

    let result = serde_json::json!({
        "title": props.title,
        "refinement": props.refinement.value(),
        "audience": props.audience.to_string(),
        "origin": props.origin.to_string(),
        "form": props.form.to_string(),
        "stubCount": props.stubs.len(),
        "tags": props.tags,
        "aliases": props.aliases
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

/// Analyze document: parse + calculate dimensions
pub fn analyze_document(parser: &Arc<dyn DocumentParser>, args: serde_json::Value) -> Result<String, String> {
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'content' argument")?;

    let use_case = AnalyzeDocumentUseCase::new(Arc::clone(parser));

    let analysis = use_case.analyze(content).map_err(|e| e.to_string())?;

    let result = serde_json::json!({
        "properties": {
            "title": analysis.properties.title,
            "refinement": analysis.properties.refinement.value(),
            "audience": analysis.properties.audience.to_string(),
            "origin": analysis.properties.origin.to_string(),
            "form": analysis.properties.form.to_string(),
            "stubCount": analysis.properties.stubs.len()
        },
        "dimensions": {
            "health": analysis.dimensions.health,
            "usefulnessMargin": analysis.dimensions.usefulness.margin,
            "isUseful": analysis.dimensions.usefulness.is_useful,
            "trustLevel": analysis.dimensions.trust_level,
            "freshness": analysis.dimensions.freshness
        },
        "warnings": analysis.warnings.iter().map(|w| &w.message).collect::<Vec<_>>()
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

/// Validate document against schema
pub fn validate_document(
    parser: &Arc<dyn DocumentParser>,
    schema_provider: &Arc<dyn SchemaProvider>,
    args: serde_json::Value,
) -> Result<String, String> {
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'content' argument")?;

    let strict = args
        .get("strict")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let use_case = ValidateDocumentUseCase::new(
        Arc::clone(parser),
        Arc::clone(schema_provider),
    );

    let result = use_case.validate(content, strict).map_err(|e| e.to_string())?;

    let output = serde_json::json!({
        "isValid": result.is_valid,
        "errorCount": result.errors.len(),
        "warningCount": result.warnings.len(),
        "errors": result.errors.iter().map(|e| {
            serde_json::json!({
                "message": e.message,
                "path": e.path,
                "position": e.position.map(|p| format!("{}:{}", p.line, p.column))
            })
        }).collect::<Vec<_>>(),
        "warnings": result.warnings.iter().map(|w| {
            serde_json::json!({
                "message": w.message,
                "path": w.path,
                "suggestion": w.suggestion
            })
        }).collect::<Vec<_>>()
    });

    serde_json::to_string_pretty(&output).map_err(|e| e.to_string())
}

/// List stubs from document
pub fn list_stubs(parser: &Arc<dyn DocumentParser>, args: serde_json::Value) -> Result<String, String> {
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'content' argument")?;

    let type_filter = args
        .get("type_filter")
        .and_then(|v| v.as_str());

    let blocking_only = args
        .get("blocking_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let props = parser.parse(content).map_err(|e| e.to_string())?;

    let filtered: Vec<_> = props
        .stubs
        .iter()
        .filter(|s| {
            if let Some(filter) = type_filter {
                if !s.stub_type.as_str().to_lowercase().contains(&filter.to_lowercase()) {
                    return false;
                }
            }
            if blocking_only && !s.is_blocking() {
                return false;
            }
            true
        })
        .map(|s| {
            serde_json::json!({
                "type": s.stub_type.as_str(),
                "description": s.description,
                "stubForm": s.stub_form.to_string(),
                "priority": s.priority.to_string(),
                "isBlocking": s.is_blocking(),
                "vectorFamily": s.vector_family().display_name()
            })
        })
        .collect();

    let result = serde_json::json!({
        "total": filtered.len(),
        "stubs": filtered
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

/// Calculate health score
pub fn calc_health(args: serde_json::Value) -> Result<String, String> {
    let refinement = args
        .get("refinement")
        .and_then(|v| v.as_f64())
        .ok_or("Missing 'refinement' argument")?;

    let stubs: Vec<Stub> = if let Some(stubs_val) = args.get("stubs") {
        serde_json::from_value(stubs_val.clone()).map_err(|e| format!("Invalid stubs: {}", e))?
    } else {
        Vec::new()
    };

    let health = doc_doctor_domain::calculate_health(refinement, &stubs);
    let stub_penalty: f64 = stubs.iter().map(|s| s.refinement_penalty()).sum();

    let result = serde_json::json!({
        "health": health,
        "refinement": refinement,
        "stubCount": stubs.len(),
        "stubPenalty": stub_penalty.min(1.0),
        "formula": "health = 0.7×refinement + 0.3×(1-stubPenalty)"
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

/// Calculate usefulness margin
pub fn calc_usefulness(args: serde_json::Value) -> Result<String, String> {
    let refinement = args
        .get("refinement")
        .and_then(|v| v.as_f64())
        .ok_or("Missing 'refinement' argument")?;

    let audience_str = args
        .get("audience")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'audience' argument")?;

    let audience: Audience = audience_str
        .parse()
        .map_err(|_| format!("Invalid audience: {}", audience_str))?;

    let usefulness = doc_doctor_domain::calculate_usefulness(refinement, audience);

    let result = serde_json::json!({
        "margin": usefulness.margin,
        "isUseful": usefulness.is_useful,
        "refinement": usefulness.refinement,
        "audience": usefulness.audience.to_string(),
        "gate": usefulness.gate,
        "formula": "margin = refinement - gate"
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

/// Calculate all dimensions
pub fn calculate_dimensions(parser: &Arc<dyn DocumentParser>, args: serde_json::Value) -> Result<String, String> {
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'content' argument")?;

    let props = parser.parse(content).map_err(|e| e.to_string())?;
    let dims = StateDimensions::calculate(&props);

    let result = serde_json::json!({
        "health": dims.health,
        "usefulness": {
            "margin": dims.usefulness.margin,
            "isUseful": dims.usefulness.is_useful,
            "audience": dims.usefulness.audience.to_string(),
            "gate": dims.usefulness.gate
        },
        "trustLevel": dims.trust_level,
        "freshness": dims.freshness,
        "complianceFit": dims.compliance_fit,
        "coverageFit": dims.coverage_fit
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

/// Calculate vector physics for a stub
pub fn calculate_vector_physics(args: serde_json::Value) -> Result<String, String> {
    let stub_val = args
        .get("stub")
        .ok_or("Missing 'stub' argument")?;

    let stub: Stub = serde_json::from_value(stub_val.clone())
        .map_err(|e| format!("Invalid stub: {}", e))?;

    let context: StubContext = if let Some(ctx_val) = args.get("context") {
        serde_json::from_value(ctx_val.clone())
            .map_err(|e| format!("Invalid context: {}", e))?
    } else {
        StubContext::default()
    };

    let physics = VectorPhysics::calculate(&stub, &context);

    let result = serde_json::json!({
        "potentialEnergy": physics.potential_energy,
        "frictionCoefficient": physics.friction_coefficient,
        "editorialVelocity": physics.editorial_velocity,
        "magnitude": physics.magnitude,
        "vectorFamily": stub.vector_family().display_name(),
        "formulas": {
            "potentialEnergy": "PE = urgency × impact × complexity",
            "friction": "F = controversy + dependencies + blocker_status",
            "magnitude": "M = √(PE² + F²)"
        }
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

/// Get audience gate thresholds
pub fn get_audience_gates() -> Result<String, String> {
    let result = serde_json::json!({
        "gates": {
            "personal": Audience::Personal.gate(),
            "internal": Audience::Internal.gate(),
            "trusted": Audience::Trusted.gate(),
            "public": Audience::Public.gate()
        },
        "description": "Minimum refinement score required for a document to be useful for each audience level"
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

/// Get JSON schema
pub fn get_schema(schema_provider: &Arc<dyn SchemaProvider>, args: serde_json::Value) -> Result<String, String> {
    let schema_type = args
        .get("schema_type")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'schema_type' argument")?;

    let schema_json = match schema_type.to_lowercase().as_str() {
        "frontmatter" => schema_provider.frontmatter_schema(),
        "stubs" => schema_provider.stubs_schema(),
        other => return Err(format!("Unknown schema type: '{}'. Valid types: frontmatter, stubs", other)),
    };

    // Parse and re-format for consistent output
    let schema: serde_json::Value = serde_json::from_str(schema_json)
        .map_err(|e| format!("Invalid schema JSON: {}", e))?;

    serde_json::to_string_pretty(&schema).map_err(|e| e.to_string())
}

/// Batch analyze multiple documents
pub fn batch_analyze(parser: &Arc<dyn DocumentParser>, args: serde_json::Value) -> Result<String, String> {
    let documents = args
        .get("documents")
        .and_then(|v| v.as_array())
        .ok_or("Missing 'documents' argument")?;

    let use_case = AnalyzeDocumentUseCase::new(Arc::clone(parser));

    let mut results = Vec::new();
    let mut total_health = 0.0;
    let mut health_count = 0;
    let mut succeeded = 0;
    let mut failed = 0;

    for doc in documents {
        let path = doc
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let content = doc
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match use_case.analyze(content) {
            Ok(analysis) => {
                succeeded += 1;
                total_health += analysis.dimensions.health;
                health_count += 1;

                results.push(serde_json::json!({
                    "path": path,
                    "success": true,
                    "health": analysis.dimensions.health,
                    "isUseful": analysis.dimensions.usefulness.is_useful
                }));
            }
            Err(e) => {
                failed += 1;
                results.push(serde_json::json!({
                    "path": path,
                    "success": false,
                    "error": e.to_string()
                }));
            }
        }
    }

    let average_health = if health_count > 0 {
        Some(total_health / health_count as f64)
    } else {
        None
    };

    let result = serde_json::json!({
        "total": documents.len(),
        "succeeded": succeeded,
        "failed": failed,
        "averageHealth": average_health,
        "results": results
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}
