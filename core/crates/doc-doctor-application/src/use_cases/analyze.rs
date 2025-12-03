//! Analyze Document Use Case
//!
//! Full document analysis: parse content and calculate all dimensions.

use doc_doctor_domain::{
    AnalysisError, AnalyzeDocument, DocumentAnalysis, DocumentParser, StateDimensions,
    ValidationWarning,
};
use std::sync::Arc;

/// Analyze document use case implementation
///
/// Parses a document and calculates all L2 dimensions.
pub struct AnalyzeDocumentUseCase {
    parser: Arc<dyn DocumentParser>,
}

impl AnalyzeDocumentUseCase {
    /// Create a new analyze document use case
    pub fn new(parser: Arc<dyn DocumentParser>) -> Self {
        Self { parser }
    }

    /// Create with a boxed parser
    pub fn with_parser(parser: Box<dyn DocumentParser>) -> Self {
        Self {
            parser: Arc::from(parser),
        }
    }
}

impl AnalyzeDocument for AnalyzeDocumentUseCase {
    fn analyze(&self, content: &str) -> Result<DocumentAnalysis, AnalysisError> {
        // Parse the document
        let properties = self.parser.parse(content).map_err(|e| {
            AnalysisError::new(e.message).with_cause("parse")
        })?;

        // Calculate state dimensions
        let dimensions = StateDimensions::calculate(&properties);

        // Collect any warnings (currently empty, would come from validation)
        let warnings: Vec<ValidationWarning> = Vec::new();

        Ok(DocumentAnalysis {
            properties,
            dimensions,
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_doctor_domain::{Audience, L1Properties, MetadataSpan, ParseError};

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
    fn test_analyze_success() {
        let props = L1Properties::new()
            .refinement(0.75)
            .audience(Audience::Internal);

        let use_case = AnalyzeDocumentUseCase::with_parser(Box::new(MockParser::success(props)));

        let result = use_case.analyze("test content");
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.properties.refinement.value(), 0.75);
        assert!(analysis.dimensions.health > 0.0);
    }

    #[test]
    fn test_analyze_parse_failure() {
        let use_case =
            AnalyzeDocumentUseCase::with_parser(Box::new(MockParser::failure("Parse failed")));

        let result = use_case.analyze("test content");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.cause.as_deref(), Some("parse"));
    }

    #[test]
    fn test_dimensions_calculated() {
        let mut props = L1Properties::new()
            .refinement(0.8)
            .audience(Audience::Public);

        // Add a stub to affect health
        props.stubs.push(doc_doctor_domain::Stub::compact("link", "citation"));

        let use_case = AnalyzeDocumentUseCase::with_parser(Box::new(MockParser::success(props)));

        let analysis = use_case.analyze("content").unwrap();

        // Health should be calculated (affected by stubs)
        assert!(analysis.dimensions.health < 1.0);

        // Usefulness should be calculated for Public audience
        assert!(!analysis.dimensions.usefulness.is_useful); // 0.8 < 0.9 gate
    }
}
