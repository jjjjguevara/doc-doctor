//! Doc-Doctor Application Layer
//!
//! Use case orchestration that coordinates domain logic with outbound ports.
//! This layer implements the inbound port traits defined in the domain.
//!
//! # Architecture
//!
//! ```text
//!     ┌─────────────────────────────────────┐
//!     │         Inbound Adapters            │
//!     │     (CLI, MCP, WASM, REST API)      │
//!     └──────────────┬──────────────────────┘
//!                    │ calls
//!                    ▼
//!     ┌─────────────────────────────────────┐
//!     │      APPLICATION LAYER              │
//!     │   ┌───────────────────────────┐    │
//!     │   │      Switchboard          │    │
//!     │   │  - parse_document         │    │
//!     │   │  - analyze_document       │    │
//!     │   │  - add_stub               │    │
//!     │   │  - resolve_stub           │    │
//!     │   │  - calculate_health       │    │
//!     │   └───────────────────────────┘    │
//!     └──────────────┬──────────────────────┘
//!                    │ uses
//!                    ▼
//!     ┌─────────────────────────────────────┐
//!     │         Domain Layer               │
//!     │  (Entities, Calculations, Ports)   │
//!     └─────────────────────────────────────┘
//! ```
//!
//! # Switchboard Pattern
//!
//! The application layer implements the Port Switchboard pattern, providing
//! a unified interface for all Doc-Doctor operations. Consumers (CLI, MCP, WASM)
//! call switchboard methods instead of implementing their own adapters.
//!
//! - [`Switchboard`]: Central routing trait for all operations
//! - [`ApplicationSwitchboard`]: Default implementation
//!
//! # Use Cases (Legacy)
//!
//! - [`AnalyzeDocumentUseCase`]: Full document analysis (parse + calculate)
//! - [`ValidateDocumentUseCase`]: Schema validation
//! - [`BatchProcessUseCase`]: Process multiple documents with glob patterns

mod error;
pub mod switchboard;
pub mod use_cases;

pub use error::{ApplicationError, ApplicationResult};
pub use switchboard::{
    AnchorLinkResult, AnchorMatches, ApplicationSwitchboard, NewStub, StubAddResult, StubFilter,
    StubResolveResult, StubUpdateResult, StubUpdates, Switchboard, SwitchboardError,
};
pub use use_cases::{
    AnalyzeDocumentUseCase, BatchProcessUseCase, DocumentService, ValidateDocumentUseCase,
};
