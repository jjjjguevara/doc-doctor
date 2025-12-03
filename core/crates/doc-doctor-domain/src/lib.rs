//! Doc-Doctor Domain Layer
//!
//! Pure domain logic following Hexagonal Architecture principles.
//! This crate contains:
//!
//! - **Entities**: L1 intrinsic properties (refinement, audience, form, origin, stubs)
//! - **Calculations**: L2 extrinsic dimensions (state and trajectory)
//! - **Ports**: Interfaces to the outside world (inbound use cases, outbound services)
//! - **Errors**: Domain-specific errors
//!
//! # Architecture
//!
//! ```text
//!                    ┌─────────────────────────────────────┐
//!                    │          Inbound Ports              │
//!                    │  (AnalyzeDocument, ValidateDocument,│
//!                    │   CalculateDimensions, BatchProcess)│
//!                    └──────────────┬──────────────────────┘
//!                                   │
//!                                   ▼
//!     ┌─────────────────────────────────────────────────────────┐
//!     │                    DOMAIN LAYER                         │
//!     │  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
//!     │  │  Entities   │  │ Calculations │  │    Errors     │  │
//!     │  │ (L1 Props)  │  │ (L2 Dims)    │  │  (Validation) │  │
//!     │  └─────────────┘  └──────────────┘  └───────────────┘  │
//!     └─────────────────────────────────────────────────────────┘
//!                                   │
//!                                   ▼
//!                    ┌─────────────────────────────────────┐
//!                    │         Outbound Ports              │
//!                    │  (DocumentParser, DocumentRepository│
//!                    │   SchemaProvider, RuleEngine)       │
//!                    └─────────────────────────────────────┘
//! ```
//!
//! # Design Principles
//!
//! - **Zero external dependencies** (except serde for serialization)
//! - **Pure functions** for all calculations
//! - **Format-agnostic** ports (YAML, LKO, TOML support via adapters)
//! - **Axiological foundations**: Principled Disconnection, Deterministic Execution

pub mod calculations;
pub mod config;
pub mod entities;
pub mod errors;
pub mod ports;

// Re-export commonly used types for convenience
pub use calculations::{
    calculate_friction, calculate_health, calculate_health_with_config, calculate_magnitude,
    calculate_potential_energy, calculate_stub_penalty, calculate_stub_penalty_with_config,
    calculate_usefulness, calculate_usefulness_with_config, forecast_completion, StateDimensions,
    StubContext, TrajectoryDimensions, Usefulness, VectorPhysics,
};

pub use entities::{
    Audience, Form, L1Properties, Origin, Priority, Refinement, Stub, StubForm, StubOrigin,
    StubType, VectorFamily,
};

pub use config::{
    AudienceGatesConfig, CalculationConfig, ConfigValidationError, FormCadencesConfig,
    HealthConfig, StubPenaltiesConfig, TrustFactorsConfig, VectorPhysicsConfig,
};

pub use errors::{DomainError, DomainResult, ValidationWarning};

pub use ports::inbound::{
    AnalysisError, AnalyzeDocument, BatchDocumentResult, BatchError, BatchProcess, BatchResult,
    CalculateDimensions, DefaultCalculator, DocumentAnalysis, SchemaError, SchemaWarning,
    ValidateDocument, ValidationError, ValidationResult,
};

pub use ports::outbound::{
    Action, ActionType, ConfigError, ConfigProvider, DefaultConfigProvider, DocumentMetadata,
    DocumentParser, DocumentRepository, DocumentWriter, EmbeddedSchemaProvider, LayeredConfigProvider,
    MetadataSpan, NoOpRuleEngine, ParseError, RepositoryError, RepositoryErrorKind, RuleContext,
    RuleEngine, RuleError, RuleResult, SchemaProvider, SerializeError, SourcePosition,
};
