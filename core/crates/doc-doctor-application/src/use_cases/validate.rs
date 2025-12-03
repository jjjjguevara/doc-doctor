//! Validate Document Use Case
//!
//! Schema validation for document frontmatter.

use doc_doctor_domain::{
    DocumentParser, SchemaError, SchemaProvider, SchemaWarning, ValidateDocument,
    ValidationError, ValidationResult,
};
use std::sync::Arc;

/// Validate document use case implementation
///
/// Validates document content against the J-Editorial schema.
pub struct ValidateDocumentUseCase {
    parser: Arc<dyn DocumentParser>,
    schema_provider: Arc<dyn SchemaProvider>,
}

impl ValidateDocumentUseCase {
    /// Create a new validate document use case
    pub fn new(
        parser: Arc<dyn DocumentParser>,
        schema_provider: Arc<dyn SchemaProvider>,
    ) -> Self {
        Self {
            parser,
            schema_provider,
        }
    }

    /// Create with boxed dependencies
    pub fn with_deps(
        parser: Box<dyn DocumentParser>,
        schema_provider: Box<dyn SchemaProvider>,
    ) -> Self {
        Self {
            parser: Arc::from(parser),
            schema_provider: Arc::from(schema_provider),
        }
    }

    /// Validate value ranges
    fn validate_ranges(&self, content: &str) -> Vec<SchemaError> {
        let mut errors = Vec::new();

        // Try to parse - if it fails, that's a different error
        if let Ok(props) = self.parser.parse(content) {
            // Check refinement range
            let refinement = props.refinement.value();
            if !(0.0..=1.0).contains(&refinement) {
                errors.push(
                    SchemaError::new(format!(
                        "Refinement value {} is out of range (must be 0.0-1.0)",
                        refinement
                    ))
                    .with_path("/refinement"),
                );
            }

            // Check stub urgency/impact/complexity ranges
            for (i, stub) in props.stubs.iter().enumerate() {
                if let Some(urgency) = stub.urgency {
                    if !(0.0..=1.0).contains(&urgency) {
                        errors.push(
                            SchemaError::new(format!(
                                "Stub urgency {} is out of range (must be 0.0-1.0)",
                                urgency
                            ))
                            .with_path(format!("/stubs/{}/urgency", i)),
                        );
                    }
                }

                if let Some(impact) = stub.impact {
                    if !(0.0..=1.0).contains(&impact) {
                        errors.push(
                            SchemaError::new(format!(
                                "Stub impact {} is out of range (must be 0.0-1.0)",
                                impact
                            ))
                            .with_path(format!("/stubs/{}/impact", i)),
                        );
                    }
                }

                if let Some(complexity) = stub.complexity {
                    if !(0.0..=1.0).contains(&complexity) {
                        errors.push(
                            SchemaError::new(format!(
                                "Stub complexity {} is out of range (must be 0.0-1.0)",
                                complexity
                            ))
                            .with_path(format!("/stubs/{}/complexity", i)),
                        );
                    }
                }
            }
        }

        errors
    }

    /// Check for warnings (non-fatal issues)
    fn check_warnings(&self, content: &str) -> Vec<SchemaWarning> {
        let mut warnings = Vec::new();

        if let Ok(props) = self.parser.parse(content) {
            // Warn if refinement is 0.0 (default)
            if props.refinement.value() == 0.0 {
                warnings.push(
                    SchemaWarning::new("Refinement is 0.0 (default). Consider setting an explicit value.")
                        .with_path("/refinement"),
                );
            }

            // Warn about stubs without descriptions
            for (i, stub) in props.stubs.iter().enumerate() {
                if stub.description.is_empty() {
                    warnings.push(
                        SchemaWarning::new("Stub has empty description")
                            .with_path(format!("/stubs/{}/description", i))
                            .with_suggestion("Add a meaningful description for the stub"),
                    );
                }
            }

            // Warn about missing title
            if props.title.is_none() {
                warnings.push(
                    SchemaWarning::new("Document has no title")
                        .with_path("/title")
                        .with_suggestion("Add a title for better organization"),
                );
            }
        }

        warnings
    }
}

impl ValidateDocument for ValidateDocumentUseCase {
    fn validate(&self, content: &str, strict: bool) -> Result<ValidationResult, ValidationError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // First, try to parse - collect syntax errors
        match self.parser.parse(content) {
            Ok(_) => {
                // Parse succeeded, check semantic validity
                errors.extend(self.validate_ranges(content));
                warnings.extend(self.check_warnings(content));
            }
            Err(parse_err) => {
                // Parse failed
                errors.push(SchemaError::new(parse_err.message).with_position_opt(parse_err.position));
            }
        }

        // If strict mode, we might add additional checks
        if strict {
            // Could add unknown field detection here
            // (already handled in parser for YAML)
        }

        if errors.is_empty() {
            Ok(ValidationResult::valid().with_warnings(warnings))
        } else {
            Ok(ValidationResult::invalid(errors).with_warnings(warnings))
        }
    }
}

/// Extension trait for SchemaError to add optional position
trait SchemaErrorExt {
    fn with_position_opt(self, position: Option<doc_doctor_domain::SourcePosition>) -> Self;
}

impl SchemaErrorExt for SchemaError {
    fn with_position_opt(self, position: Option<doc_doctor_domain::SourcePosition>) -> Self {
        if let Some(pos) = position {
            self.with_position(pos)
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_doctor_domain::{EmbeddedSchemaProvider, L1Properties, MetadataSpan, ParseError};

    /// Mock parser for testing
    struct MockParser {
        result: Result<L1Properties, ParseError>,
    }

    impl MockParser {
        fn success(props: L1Properties) -> Self {
            Self { result: Ok(props) }
        }

        fn failure(message: &str) -> Self {
            Self {
                result: Err(ParseError::new(message)),
            }
        }
    }

    impl DocumentParser for MockParser {
        fn parse(&self, _content: &str) -> Result<L1Properties, ParseError> {
            self.result.clone()
        }

        fn extract_metadata(&self, _content: &str) -> Option<MetadataSpan> {
            None
        }

        fn format_id(&self) -> &'static str {
            "mock"
        }
    }

    #[test]
    fn test_validate_valid_document() {
        let props = L1Properties::new().refinement(0.75);

        let use_case = ValidateDocumentUseCase::with_deps(
            Box::new(MockParser::success(props)),
            Box::new(EmbeddedSchemaProvider),
        );

        let result = use_case.validate("content", false).unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_parse_error() {
        let use_case = ValidateDocumentUseCase::with_deps(
            Box::new(MockParser::failure("Invalid YAML")),
            Box::new(EmbeddedSchemaProvider),
        );

        let result = use_case.validate("content", false).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_warnings() {
        // Document with default refinement and no title
        let props = L1Properties::new();

        let use_case = ValidateDocumentUseCase::with_deps(
            Box::new(MockParser::success(props)),
            Box::new(EmbeddedSchemaProvider),
        );

        let result = use_case.validate("content", false).unwrap();
        assert!(result.is_valid); // Warnings don't make it invalid
        assert!(!result.warnings.is_empty());
    }
}
