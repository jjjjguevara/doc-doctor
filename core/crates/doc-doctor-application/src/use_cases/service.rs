//! Document Service
//!
//! Convenience facade that combines all use cases with shared dependencies.

use doc_doctor_domain::{
    AnalysisError, AnalyzeDocument, BatchError, BatchProcess, BatchResult, DocumentAnalysis,
    DocumentParser, DocumentRepository, SchemaProvider, ValidateDocument, ValidationError,
    ValidationResult,
};
use std::sync::Arc;

use super::{AnalyzeDocumentUseCase, BatchProcessUseCase, ValidateDocumentUseCase};

/// Document service combining all use cases
///
/// Provides a single entry point for all document operations with
/// shared parser and repository dependencies.
pub struct DocumentService {
    analyze: AnalyzeDocumentUseCase,
    validate: ValidateDocumentUseCase,
    batch: BatchProcessUseCase,
}

impl DocumentService {
    /// Create a new document service
    pub fn new(
        parser: Arc<dyn DocumentParser>,
        repository: Arc<dyn DocumentRepository>,
        schema_provider: Arc<dyn SchemaProvider>,
    ) -> Self {
        Self {
            analyze: AnalyzeDocumentUseCase::new(Arc::clone(&parser)),
            validate: ValidateDocumentUseCase::new(Arc::clone(&parser), schema_provider),
            batch: BatchProcessUseCase::new(parser, repository),
        }
    }

    /// Analyze a document (parse + calculate dimensions)
    pub fn analyze(&self, content: &str) -> Result<DocumentAnalysis, AnalysisError> {
        self.analyze.analyze(content)
    }

    /// Validate document against schema
    pub fn validate(&self, content: &str, strict: bool) -> Result<ValidationResult, ValidationError> {
        self.validate.validate(content, strict)
    }

    /// Batch process documents matching a pattern
    pub fn batch_process(&self, pattern: &str) -> Result<BatchResult, BatchError> {
        self.batch.process(pattern)
    }
}

/// Builder for DocumentService
pub struct DocumentServiceBuilder {
    parser: Option<Arc<dyn DocumentParser>>,
    repository: Option<Arc<dyn DocumentRepository>>,
    schema_provider: Option<Arc<dyn SchemaProvider>>,
}

impl DocumentServiceBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            parser: None,
            repository: None,
            schema_provider: None,
        }
    }

    /// Set the parser
    pub fn parser(mut self, parser: impl DocumentParser + 'static) -> Self {
        self.parser = Some(Arc::new(parser));
        self
    }

    /// Set the parser from Arc
    pub fn parser_arc(mut self, parser: Arc<dyn DocumentParser>) -> Self {
        self.parser = Some(parser);
        self
    }

    /// Set the repository
    pub fn repository(mut self, repository: impl DocumentRepository + 'static) -> Self {
        self.repository = Some(Arc::new(repository));
        self
    }

    /// Set the repository from Arc
    pub fn repository_arc(mut self, repository: Arc<dyn DocumentRepository>) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Set the schema provider
    pub fn schema_provider(mut self, provider: impl SchemaProvider + 'static) -> Self {
        self.schema_provider = Some(Arc::new(provider));
        self
    }

    /// Set the schema provider from Arc
    pub fn schema_provider_arc(mut self, provider: Arc<dyn SchemaProvider>) -> Self {
        self.schema_provider = Some(provider);
        self
    }

    /// Build the service
    ///
    /// # Panics
    /// Panics if any required dependency is missing
    pub fn build(self) -> DocumentService {
        DocumentService::new(
            self.parser.expect("Parser is required"),
            self.repository.expect("Repository is required"),
            self.schema_provider.expect("Schema provider is required"),
        )
    }

    /// Try to build the service
    pub fn try_build(self) -> Option<DocumentService> {
        Some(DocumentService::new(
            self.parser?,
            self.repository?,
            self.schema_provider?,
        ))
    }
}

impl Default for DocumentServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_doctor_domain::{
        DocumentMetadata, EmbeddedSchemaProvider, L1Properties, MetadataSpan, ParseError,
        RepositoryError,
    };
    use std::path::{Path, PathBuf};

    struct MockParser;

    impl DocumentParser for MockParser {
        fn parse(&self, _content: &str) -> Result<L1Properties, ParseError> {
            Ok(L1Properties::new().refinement(0.75))
        }

        fn extract_metadata(&self, _content: &str) -> Option<MetadataSpan> {
            None
        }

        fn format_id(&self) -> &'static str {
            "mock"
        }
    }

    struct MockRepository;

    impl DocumentRepository for MockRepository {
        fn read(&self, _path: &Path) -> Result<String, RepositoryError> {
            Ok("content".to_string())
        }

        fn write(&self, _path: &Path, _content: &str) -> Result<(), RepositoryError> {
            Ok(())
        }

        fn list(&self, _pattern: &str) -> Result<Vec<PathBuf>, RepositoryError> {
            Ok(vec![PathBuf::from("test.md")])
        }

        fn exists(&self, _path: &Path) -> bool {
            true
        }

        fn metadata(&self, _path: &Path) -> Result<DocumentMetadata, RepositoryError> {
            Ok(DocumentMetadata {
                size: 100,
                modified: None,
                created: None,
                is_directory: false,
            })
        }
    }

    #[test]
    fn test_service_builder() {
        let service = DocumentServiceBuilder::new()
            .parser(MockParser)
            .repository(MockRepository)
            .schema_provider(EmbeddedSchemaProvider)
            .build();

        // Service should work
        let result = service.analyze("content");
        assert!(result.is_ok());
    }

    #[test]
    fn test_service_analyze() {
        let service = DocumentServiceBuilder::new()
            .parser(MockParser)
            .repository(MockRepository)
            .schema_provider(EmbeddedSchemaProvider)
            .build();

        let analysis = service.analyze("content").unwrap();
        assert_eq!(analysis.properties.refinement.value(), 0.75);
    }

    #[test]
    fn test_service_validate() {
        let service = DocumentServiceBuilder::new()
            .parser(MockParser)
            .repository(MockRepository)
            .schema_provider(EmbeddedSchemaProvider)
            .build();

        let result = service.validate("content", false).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_service_batch() {
        let service = DocumentServiceBuilder::new()
            .parser(MockParser)
            .repository(MockRepository)
            .schema_provider(EmbeddedSchemaProvider)
            .build();

        let result = service.batch_process("**/*.md").unwrap();
        assert_eq!(result.total, 1);
    }
}
