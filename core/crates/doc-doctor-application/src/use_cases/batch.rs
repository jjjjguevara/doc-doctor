//! Batch Process Use Case
//!
//! Process multiple documents matching a glob pattern.

use doc_doctor_domain::{
    BatchDocumentResult, BatchError, BatchProcess, BatchResult, DocumentParser,
    DocumentRepository, StateDimensions,
};
use rayon::prelude::*;
use std::sync::Arc;

/// Batch process use case implementation
///
/// Processes multiple documents in parallel using glob patterns.
pub struct BatchProcessUseCase {
    parser: Arc<dyn DocumentParser>,
    repository: Arc<dyn DocumentRepository>,
}

impl BatchProcessUseCase {
    /// Create a new batch process use case
    pub fn new(
        parser: Arc<dyn DocumentParser>,
        repository: Arc<dyn DocumentRepository>,
    ) -> Self {
        Self { parser, repository }
    }

    /// Create with boxed dependencies
    pub fn with_deps(
        parser: Box<dyn DocumentParser>,
        repository: Box<dyn DocumentRepository>,
    ) -> Self {
        Self {
            parser: Arc::from(parser),
            repository: Arc::from(repository),
        }
    }

    /// Process a single document
    fn process_document(&self, path: std::path::PathBuf) -> BatchDocumentResult {
        // Read content
        let content = match self.repository.read(&path) {
            Ok(c) => c,
            Err(e) => {
                return BatchDocumentResult::failure(path, format!("Read error: {}", e.message));
            }
        };

        // Parse document
        let properties = match self.parser.parse(&content) {
            Ok(p) => p,
            Err(e) => {
                return BatchDocumentResult::failure(path, format!("Parse error: {}", e.message));
            }
        };

        // Calculate dimensions
        let dimensions = StateDimensions::calculate(&properties);

        BatchDocumentResult::success(path, properties, dimensions)
    }
}

impl BatchProcess for BatchProcessUseCase {
    fn process(&self, pattern: &str) -> Result<BatchResult, BatchError> {
        // List files matching the pattern
        let paths = self.repository.list(pattern).map_err(|e| {
            BatchError::new(format!("Failed to list files: {}", e.message))
        })?;

        if paths.is_empty() {
            return Ok(BatchResult::new(Vec::new()));
        }

        // Process documents in parallel
        let results: Vec<BatchDocumentResult> = paths
            .into_par_iter()
            .map(|path| self.process_document(path))
            .collect();

        Ok(BatchResult::new(results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_doctor_domain::{
        DocumentMetadata, L1Properties, MetadataSpan, ParseError, RepositoryError,
        RepositoryErrorKind,
    };
    use std::path::{Path, PathBuf};

    /// Mock parser for testing
    struct MockParser;

    impl DocumentParser for MockParser {
        fn parse(&self, content: &str) -> Result<L1Properties, ParseError> {
            if content.contains("error") {
                Err(ParseError::new("Parse failed"))
            } else {
                Ok(L1Properties::new().refinement(0.75))
            }
        }

        fn extract_metadata(&self, _content: &str) -> Option<MetadataSpan> {
            None
        }

        fn format_id(&self) -> &'static str {
            "mock"
        }
    }

    /// Mock repository for testing
    struct MockRepository {
        files: Vec<(PathBuf, String)>,
    }

    impl MockRepository {
        fn new(files: Vec<(&str, &str)>) -> Self {
            Self {
                files: files
                    .into_iter()
                    .map(|(p, c)| (PathBuf::from(p), c.to_string()))
                    .collect(),
            }
        }
    }

    impl DocumentRepository for MockRepository {
        fn read(&self, path: &Path) -> Result<String, RepositoryError> {
            self.files
                .iter()
                .find(|(p, _)| p == path)
                .map(|(_, c)| c.clone())
                .ok_or_else(|| RepositoryError {
                    kind: RepositoryErrorKind::NotFound,
                    message: "File not found".to_string(),
                    path: Some(path.to_path_buf()),
                })
        }

        fn write(&self, _path: &Path, _content: &str) -> Result<(), RepositoryError> {
            Ok(())
        }

        fn list(&self, _pattern: &str) -> Result<Vec<PathBuf>, RepositoryError> {
            Ok(self.files.iter().map(|(p, _)| p.clone()).collect())
        }

        fn exists(&self, path: &Path) -> bool {
            self.files.iter().any(|(p, _)| p == path)
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
    fn test_batch_process_success() {
        let repo = MockRepository::new(vec![
            ("doc1.md", "title: Doc 1"),
            ("doc2.md", "title: Doc 2"),
        ]);

        let use_case =
            BatchProcessUseCase::with_deps(Box::new(MockParser), Box::new(repo));

        let result = use_case.process("**/*.md").unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.succeeded, 2);
        assert_eq!(result.failed, 0);
    }

    #[test]
    fn test_batch_process_partial_failure() {
        let repo = MockRepository::new(vec![
            ("doc1.md", "title: Doc 1"),
            ("doc2.md", "error: bad content"),
        ]);

        let use_case =
            BatchProcessUseCase::with_deps(Box::new(MockParser), Box::new(repo));

        let result = use_case.process("**/*.md").unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.succeeded, 1);
        assert_eq!(result.failed, 1);
    }

    #[test]
    fn test_batch_process_empty() {
        let repo = MockRepository::new(vec![]);

        let use_case =
            BatchProcessUseCase::with_deps(Box::new(MockParser), Box::new(repo));

        let result = use_case.process("**/*.md").unwrap();
        assert_eq!(result.total, 0);
    }

    #[test]
    fn test_average_health() {
        let repo = MockRepository::new(vec![
            ("doc1.md", "title: Doc 1"),
            ("doc2.md", "title: Doc 2"),
        ]);

        let use_case =
            BatchProcessUseCase::with_deps(Box::new(MockParser), Box::new(repo));

        let result = use_case.process("**/*.md").unwrap();
        let avg_health = result.average_health();

        assert!(avg_health.is_some());
        assert!(avg_health.unwrap() > 0.0);
    }
}
