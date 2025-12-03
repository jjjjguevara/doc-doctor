//! CLI Command Implementations
//!
//! Each command is implemented as a separate module.
//! All commands use the Application Switchboard for consistent behavior.

pub mod batch;
pub mod config;
pub mod dashboard;
pub mod dimensions;
pub mod health;
pub mod parse;
pub mod schema;
pub mod stubs;
pub mod test;
pub mod usefulness;
pub mod validate;

use std::path::Path;
use std::sync::Arc;

use doc_doctor_application::{
    AnalyzeDocumentUseCase, ApplicationSwitchboard, ValidateDocumentUseCase,
};
use doc_doctor_domain::{DocumentParser, EmbeddedSchemaProvider, SchemaProvider};
use doc_doctor_parser_yaml::YamlParser;

/// Type alias for the concrete switchboard used by CLI
pub type CliSwitchboard = ApplicationSwitchboard<YamlParser, YamlParser, EmbeddedSchemaProvider>;

/// Create the application switchboard
///
/// This is the central routing mechanism for all Doc-Doctor operations.
/// CLI commands should use this switchboard instead of directly accessing
/// parsers or use cases.
pub fn create_switchboard() -> Arc<CliSwitchboard> {
    let parser = Arc::new(YamlParser::new());
    let writer = Arc::clone(&parser);
    let schema_provider = Arc::new(EmbeddedSchemaProvider);
    Arc::new(ApplicationSwitchboard::new(parser, writer, schema_provider))
}

/// Create the default parser
pub fn create_parser() -> Arc<dyn DocumentParser> {
    Arc::new(YamlParser::new())
}

/// Create the default schema provider
pub fn create_schema_provider() -> Arc<dyn SchemaProvider> {
    Arc::new(EmbeddedSchemaProvider)
}

/// Create the analyze use case
pub fn create_analyze_use_case() -> AnalyzeDocumentUseCase {
    AnalyzeDocumentUseCase::new(create_parser())
}

/// Create the validate use case
pub fn create_validate_use_case() -> ValidateDocumentUseCase {
    ValidateDocumentUseCase::new(create_parser(), create_schema_provider())
}

/// Read file content with error handling
pub fn read_file(path: &Path) -> anyhow::Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read '{}': {}", path.display(), e))
}

/// Write file content with error handling
pub fn write_file(path: &Path, content: &str) -> anyhow::Result<()> {
    std::fs::write(path, content)
        .map_err(|e| anyhow::anyhow!("Failed to write '{}': {}", path.display(), e))
}
