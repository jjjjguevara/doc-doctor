//! Outbound Ports (Driven)
//!
//! These ports define services that the domain needs from the outside world.
//! Adapters implement these traits to provide actual functionality.

mod config_provider;
mod parser;
mod repository;
mod rules;
mod schema;

pub use config_provider::{ConfigError, ConfigProvider, DefaultConfigProvider, LayeredConfigProvider};
pub use parser::{DocumentParser, DocumentWriter, MetadataSpan, ParseError, SerializeError, SourcePosition};
pub use repository::{DocumentMetadata, DocumentRepository, RepositoryError, RepositoryErrorKind};
pub use rules::{Action, ActionType, NoOpRuleEngine, RuleContext, RuleEngine, RuleError, RuleResult};
pub use schema::{EmbeddedSchemaProvider, SchemaProvider};
