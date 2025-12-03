# Hexagonal Architecture for Doc Doctor

**Version**: 0.2.0
**Date**: 2024-12-02
**Status**: Approved

---

## 1. Overview

A **Hexagonal Architecture** (Ports & Adapters) refactor of doc-doctor's Rust core, aligned with:

- **J-Editorial Framework** - 4-layer architecture (L1 Intrinsic, L2 Extrinsic, L3 Operational, L4 Commoning)
- **Axiological Foundations** - Principled Disconnection (Symploké), Deterministic Execution, Measurable Quality
- **LKO Preparation** - Format-agnostic ports for future Living Knowledge Object support

**Core Principle**: Domain depends on NOTHING. All dependencies point inward.

**Decision**: Clean break from `doc-doctor-core` - new crate structure, no backward compatibility facade.

---

## 2. Axiological Alignment

| Axiom | Hexagonal Mapping | Implementation |
|-------|-------------------|----------------|
| **Principled Disconnection (Symploké)** | Ports isolate domain from infrastructure | Domain crate has zero external deps beyond serde |
| **Deterministic Execution** | Domain = pure functions (Executor) | L2 calculations are deterministic; adapters handle I/O |
| **Measurable Quality** | Domain calculations | Health, usefulness, vector physics with exact J-Editorial formulas |
| **Internal Products** | Port traits as APIs | Each port has clear contract, documentation, stable interface |
| **Lifecycle Budgeting** | Separate crates | Domain stable for years; adapters evolve independently |

---

## 3. Hexagonal Architecture Principles

```
                    ┌─────────────────────────────────────┐
                    │         INBOUND ADAPTERS            │
                    │   (CLI, MCP, WASM, REST API)        │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │         INBOUND PORTS               │
                    │   (Use Case Interfaces/Traits)      │
                    └──────────────┬──────────────────────┘
                                   │
    ┌──────────────────────────────▼──────────────────────────────┐
    │                        DOMAIN                                │
    │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
    │   │   Types     │  │ Calculations│  │   Rules     │        │
    │   │  (Entities) │  │   (Logic)   │  │  (Future)   │        │
    │   └─────────────┘  └─────────────┘  └─────────────┘        │
    └──────────────────────────────┬──────────────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │        OUTBOUND PORTS               │
                    │   (Repository/Service Traits)       │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │        OUTBOUND ADAPTERS            │
                    │   (YAML Parser, FileSystem, DB)     │
                    └─────────────────────────────────────┘
```

---

## 4. Crate Structure

```
doc-doctor/core/
├── Cargo.toml                           # Workspace root
├── rust-toolchain.toml
│
└── crates/
    │
    │  ══════════════════════════════════════════════════════════
    │                         DOMAIN LAYER
    │  ══════════════════════════════════════════════════════════
    │
    ├── doc-doctor-domain/               # Pure domain (NO external deps)
    │   ├── Cargo.toml                   # Only: serde (derive macros)
    │   └── src/
    │       ├── lib.rs
    │       │
    │       ├── entities/                # J-Editorial L1 value objects
    │       │   ├── mod.rs
    │       │   ├── document.rs          # L1Properties, DocumentId
    │       │   ├── refinement.rs        # Refinement (simple + composite)
    │       │   ├── audience.rs          # Audience enum + gates
    │       │   ├── origin.rs            # Origin enum
    │       │   ├── form.rs              # Form enum + staleness cadences
    │       │   └── stub.rs              # Stub, StubForm, StubType, VectorFamily
    │       │
    │       ├── calculations/            # J-Editorial L2 pure functions
    │       │   ├── mod.rs
    │       │   ├── state.rs             # health, usefulness, compliance, trust, freshness, coverage
    │       │   ├── trajectory.rs        # drift, health_trend, adoption, vector physics
    │       │   ├── network.rs           # network_position, propagation_risk
    │       │   └── priority.rs          # attention_priority, retention_value, effort_to_improve
    │       │
    │       ├── errors/                  # Domain-only errors
    │       │   ├── mod.rs
    │       │   ├── validation.rs        # RefinementOutOfRange, UnknownAudience, etc.
    │       │   └── calculation.rs       # MissingDependency, DivisionByZero
    │       │
    │       └── ports/                   # Trait definitions (interfaces)
    │           ├── mod.rs
    │           │
    │           ├── inbound/             # Use case contracts (driving ports)
    │           │   ├── mod.rs
    │           │   ├── analyze.rs       # trait AnalyzeDocument
    │           │   ├── validate.rs      # trait ValidateDocument
    │           │   ├── calculate.rs     # trait CalculateDimensions
    │           │   └── batch.rs         # trait BatchProcess
    │           │
    │           └── outbound/            # Service contracts (driven ports)
    │               ├── mod.rs
    │               ├── parser.rs        # trait DocumentParser (format-agnostic!)
    │               ├── repository.rs    # trait DocumentRepository
    │               ├── schema.rs        # trait SchemaProvider
    │               └── rules.rs         # trait RuleEngine (L3 - port only, deferred impl)
    │
    │  ══════════════════════════════════════════════════════════
    │                       APPLICATION LAYER
    │  ══════════════════════════════════════════════════════════
    │
    ├── doc-doctor-application/          # Use case orchestration
    │   ├── Cargo.toml                   # Depends on: domain
    │   └── src/
    │       ├── lib.rs
    │       └── use_cases/
    │           ├── mod.rs
    │           ├── analyze_document.rs  # impl AnalyzeDocument
    │           ├── validate_document.rs # impl ValidateDocument
    │           ├── calculate_health.rs
    │           ├── calculate_usefulness.rs
    │           └── batch_process.rs     # impl BatchProcess
    │
    │  ══════════════════════════════════════════════════════════
    │                        ADAPTER LAYER
    │  ══════════════════════════════════════════════════════════
    │
    ├── doc-doctor-parser-yaml/          # Outbound adapter: YAML frontmatter
    │   ├── Cargo.toml                   # Depends on: domain, serde_yaml
    │   └── src/
    │       ├── lib.rs
    │       ├── yaml_parser.rs           # impl DocumentParser for YamlParser
    │       ├── frontmatter.rs           # Frontmatter extraction
    │       └── position.rs              # Line/column tracking
    │
    ├── doc-doctor-fs/                   # Outbound adapter: file system
    │   ├── Cargo.toml                   # Depends on: domain, std::fs, glob
    │   └── src/
    │       ├── lib.rs
    │       └── file_repository.rs       # impl DocumentRepository for FileRepository
    │
    ├── doc-doctor-wasm/                 # Inbound adapter: WASM bindings
    │   ├── Cargo.toml                   # Depends on: domain, application, parser-yaml
    │   └── src/
    │       ├── lib.rs
    │       └── bindings.rs              # wasm_bindgen exports
    │
    ├── doc-doctor-mcp/                  # Inbound adapter: MCP server
    │   ├── Cargo.toml                   # Depends on: domain, application, parser-yaml, fs
    │   └── src/
    │       ├── main.rs
    │       ├── server.rs                # MCP protocol handler
    │       └── tools/                   # Tool implementations
    │
    └── doc-doctor-cli/                  # Inbound adapter + Composition Root
        ├── Cargo.toml                   # Depends on: all crates
        └── src/
            ├── main.rs                  # Wires adapters to domain
            ├── commands/                # CLI command handlers
            └── output/                  # Formatters (human, json, yaml)
```

---

## 5. Port Definitions (Contracts)

### 5.1 Outbound Ports (Driven - Domain Requests Services)

```rust
// doc-doctor-domain/src/ports/outbound/parser.rs
// FORMAT-AGNOSTIC: Same trait for YAML, LKO, TOML, etc.

pub trait DocumentParser: Send + Sync {
    /// Parse document content into L1 properties
    fn parse(&self, content: &str) -> Result<L1Properties, ParseError>;

    /// Extract raw metadata without full parsing (for validation)
    fn extract_metadata(&self, content: &str) -> Option<MetadataSpan>;

    /// Get supported format identifier
    fn format_id(&self) -> &'static str;  // "yaml", "lko", "toml"
}

pub trait DocumentRepository: Send + Sync {
    fn read(&self, path: &Path) -> Result<String, RepositoryError>;
    fn write(&self, path: &Path, content: &str) -> Result<(), RepositoryError>;
    fn list(&self, pattern: &str) -> Result<Vec<PathBuf>, RepositoryError>;
    fn exists(&self, path: &Path) -> bool;
}

pub trait SchemaProvider: Send + Sync {
    fn frontmatter_schema(&self) -> &str;  // JSON Schema
    fn stubs_schema(&self) -> &str;
}

// L3 Rule Engine - PORT ONLY (implementation deferred)
pub trait RuleEngine: Send + Sync {
    fn evaluate(&self, context: &RuleContext) -> Vec<RuleResult>;
    fn apply(&self, action: &Action, target: &mut L1Properties) -> Result<(), RuleError>;
}
```

### 5.2 Inbound Ports (Driving - External Actors Call Domain)

```rust
// doc-doctor-domain/src/ports/inbound/analyze.rs

pub trait AnalyzeDocument {
    fn analyze(&self, content: &str) -> Result<DocumentAnalysis, AnalysisError>;
}

pub struct DocumentAnalysis {
    pub properties: L1Properties,
    pub dimensions: StateDimensions,
    pub warnings: Vec<ValidationWarning>,
}

pub trait CalculateDimensions {
    fn health(&self, refinement: f64, stubs: &[Stub]) -> f64;
    fn usefulness(&self, refinement: f64, audience: Audience) -> Usefulness;
    fn vector_physics(&self, stub: &Stub, context: &StubContext) -> VectorPhysics;
    fn all_state_dimensions(&self, props: &L1Properties) -> StateDimensions;
}

pub trait BatchProcess {
    fn process(&self, pattern: &str) -> Result<BatchResult, BatchError>;
}
```

---

## 6. J-Editorial L1 Entities (Domain Layer)

```rust
// doc-doctor-domain/src/entities/

pub struct L1Properties {
    // Identity (Optional for lightweight)
    pub id: Option<String>,
    pub title: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub version: Option<String>,

    // Core 5 (Required)
    pub refinement: Refinement,
    pub origin: Origin,
    pub form: Form,
    pub audience: Audience,
    pub stubs: Vec<Stub>,

    // Extended (Optional)
    pub derived_from: Option<Vec<Attribution>>,
    pub authors: Option<Vec<Author>>,
    pub content_hash: Option<String>,
    pub evidence_level: Option<EvidenceLevel>,
    pub dependencies: Option<Vec<String>>,
}

pub enum Refinement {
    Simple(f32),
    Composite {
        score: f32,
        derived_from: RefinementComponents,
        weights: RefinementWeights,
    }
}

pub enum Audience { Personal, Internal, Trusted, Public }
pub enum Origin { Question, Requirement, Insight, Dialogue, Curiosity, Derivative }
pub enum Form { Transient, Developing, Stable, Evergreen, Canonical }

pub struct Stub {
    pub gap_id: String,
    pub description: String,
    pub stub_type: StubType,
    pub vector_family: VectorFamily,  // Derived from stub_type
    pub stub_form: StubForm,
    pub stub_origin: StubOrigin,
    pub urgency: f32,
    pub impact: f32,
    pub complexity: f32,
    pub inline_anchors: Vec<String>,
    pub assignees: Vec<String>,
    pub dependencies: Vec<String>,
}

pub enum VectorFamily { Retrieval, Computation, Synthesis, Creation, Structural }
pub enum StubForm { Transient, Persistent, Blocking, Structural }
pub enum StubOrigin { AuthorIdentified, PeerSurfaced, QADetected, SystemGenerated }
```

---

## 7. J-Editorial L2 Calculations (Domain Layer)

All calculations are **pure functions** in `doc-doctor-domain/src/calculations/`:

### 7.1 State Dimensions

```rust
// health = 0.7 × refinement + 0.3 × (1 - stub_penalty)
pub fn calculate_health(refinement: f32, stubs: &[Stub]) -> f32 {
    let penalty = calculate_stub_penalty(stubs);
    (0.7 * refinement + 0.3 * (1.0 - penalty)).clamp(0.0, 1.0)
}

fn calculate_stub_penalty(stubs: &[Stub]) -> f32 {
    stubs.iter().map(|s| match s.stub_form {
        StubForm::Transient => 0.02,
        StubForm::Persistent => 0.05,
        StubForm::Blocking => 0.10,
        StubForm::Structural => 0.15,
    }).sum::<f32>().min(1.0)
}

// margin = refinement - audience_gate
pub fn calculate_usefulness(refinement: f32, audience: Audience) -> Usefulness {
    let gate = audience.gate();
    Usefulness {
        margin: refinement - gate,
        is_useful: refinement >= gate,
        audience,
        refinement,
        gate,
    }
}

impl Audience {
    pub const fn gate(&self) -> f32 {
        match self {
            Audience::Personal => 0.50,
            Audience::Internal => 0.70,
            Audience::Trusted => 0.80,
            Audience::Public => 0.90,
        }
    }
}

// freshness = e^(-ln(2) × Δt / τ_form)
pub fn calculate_freshness(modified: DateTime<Utc>, form: Form, now: DateTime<Utc>) -> f32 {
    let days_since = (now - modified).num_days() as f32;
    let half_life = form.staleness_cadence_days();
    (-0.693 * days_since / half_life).exp()
}
```

### 7.2 Trajectory Dimensions (Vector Physics)

```rust
// potential_energy = urgency × impact × complexity
pub fn calculate_potential_energy(stub: &Stub) -> f32 {
    stub.urgency * stub.impact * stub.complexity
}

// friction = controversy + dependency_count/10 + blocker_weight
pub fn calculate_friction(stub: &Stub, context: &StubContext) -> f32 {
    let controversy = if context.has_controversy { 0.3 } else { 0.0 };
    let deps = (stub.dependencies.len() as f32) / 10.0;
    let blocker = if stub.stub_form == StubForm::Blocking { 0.2 } else { 0.0 };
    (controversy + deps + blocker).clamp(0.0, 1.0)
}

// magnitude = √(PE² + μ²)
pub fn calculate_magnitude(potential_energy: f32, friction: f32) -> f32 {
    (potential_energy.powi(2) + friction.powi(2)).sqrt()
}

// forecast = PE / (velocity × (1 - friction))
pub fn forecast_completion(pe: f32, velocity: f32, friction: f32) -> Option<f32> {
    let denominator = velocity * (1.0 - friction);
    if denominator > 0.0 { Some(pe / denominator) } else { None }
}
```

---

## 8. CLI Commands

```bash
dd <COMMAND> [OPTIONS]

Commands:
  parse       Parse and validate documents
  validate    Validate frontmatter
  stubs       List stubs from documents
  dimensions  Calculate L2 dimensions
  health      Calculate health score
  usefulness  Calculate usefulness margin
  batch       Batch process documents
  sync        Check sync status
  schema      Export JSON schema

Options:
  -f, --format <FORMAT>  human, json, yaml [default: human]
  -v, --verbose
  --vault <PATH>         Vault path (defaults to testing vault)
```

### Examples

```bash
dd parse document.md --format json
dd validate "docs/**/*.md"
dd stubs doc.md --type-filter blocker --form-filter blocking
dd health --refinement 0.75 --stubs '[{"stub_form":"blocking"}]'
dd batch "vault/**/*.md" --dimensions --jobs 8
```

---

## 9. MCP Server Tools (13 tools)

| Tool | Description |
|------|-------------|
| `doc_doctor_parse_document` | Parse markdown, extract L1 properties |
| `doc_doctor_parse_stubs` | Parse stubs from YAML |
| `doc_doctor_validate_frontmatter` | Validate against schema |
| `doc_doctor_calculate_health` | Calculate health score |
| `doc_doctor_calculate_usefulness` | Calculate usefulness margin |
| `doc_doctor_calculate_state_dimensions` | All state dimensions |
| `doc_doctor_calculate_trajectory_dimensions` | Trajectory dimensions |
| `doc_doctor_calculate_vector_physics` | Stub vector physics |
| `doc_doctor_list_stubs` | List stubs with filtering |
| `doc_doctor_check_sync_status` | Stub-anchor sync check |
| `doc_doctor_batch_analyze` | Batch document analysis |
| `doc_doctor_analyze_stub` | Deep stub analysis |
| `doc_doctor_get_audience_gates` | Get audience gate values |

---

## 10. Obsidian Plugin Integration (WASM)

```typescript
// Initialize WASM module
import init, { DocDoctor } from '@doc-doctor/wasm';

await init();
const dd = new DocDoctor();

// Use Rust core for parsing
const result = JSON.parse(dd.parseDocument(content));
const health = dd.calculateHealth(0.8, JSON.stringify(stubs));
```

### Migration Path

1. **Parallel**: Run TS + Rust, compare outputs
2. **Validation**: Test suite comparing implementations
3. **Replacement**: Swap TS parser for Rust/WASM

---

## 11. LKO Preparation (Future Adapter)

The Hexagonal Architecture enables future LKO support with minimal changes:

```rust
// Future: doc-doctor-parser-lko/src/lib.rs
pub struct LkoParser;

impl DocumentParser for LkoParser {
    fn parse(&self, content: &str) -> Result<L1Properties, ParseError> {
        // Parse LKO format, return same L1Properties
    }

    fn format_id(&self) -> &'static str {
        "lko"
    }
}
```

**What stays the same:**
- Domain entities (L1Properties, Stub, etc.)
- All L2 calculations
- Application use cases
- CLI commands (auto-detect format)
- MCP tools (format-agnostic)

**What changes:**
- New parser adapter crate
- Format detection logic in CLI/MCP

---

## 12. Migration Strategy (Current → Hexagonal)

### Current Structure

```
crates/
├── doc-doctor-core/     # Mixed concerns
├── doc-doctor-ffi/
├── doc-doctor-mcp/
└── doc-doctor-cli/
```

### Migration Steps

1. **Create `doc-doctor-domain` crate** (new)
   - Extract pure types from `doc-doctor-core/src/types/` → `domain/src/entities/`
   - Extract calculations from `doc-doctor-core/src/dimensions/` → `domain/src/calculations/`
   - Define port traits in `domain/src/ports/`
   - No external dependencies except `serde` derive macros

2. **Create `doc-doctor-application` crate** (new)
   - Implement inbound port traits
   - Wire use cases that coordinate domain + outbound ports
   - Depends only on `domain`

3. **Create `doc-doctor-parser-yaml` crate** (new)
   - Move YAML parsing from `doc-doctor-core/src/parser/` → `parser-yaml/src/`
   - Implement `DocumentParser` trait for YAML format
   - Depends on `domain`, `serde_yaml`

4. **Create `doc-doctor-fs` crate** (new)
   - Implement `DocumentRepository` trait for file system
   - Depends on `domain`, `std::fs`, `glob`

5. **Refactor `doc-doctor-wasm`** (rename from ffi)
   - Wire: `domain` + `application` + `parser-yaml`
   - Export WASM bindings via `wasm-bindgen`

6. **Refactor `doc-doctor-mcp`**
   - Wire: `domain` + `application` + `parser-yaml` + `fs`
   - Use application layer use cases for tool handlers

7. **Refactor `doc-doctor-cli`** (Composition Root)
   - Wire all adapters to domain
   - CLI commands call application layer

8. **Delete `doc-doctor-core`** once migration complete

---

## 13. Implementation Phases

### Phase 1: Domain Layer Foundation
- Create `doc-doctor-domain` crate with zero external deps (only serde derive)
- Define L1 entities: L1Properties, Refinement, Audience, Origin, Form, Stub
- Define port traits: DocumentParser, DocumentRepository, SchemaProvider, RuleEngine
- Implement L2 state calculations: health, usefulness, freshness, compliance, trust
- Implement L2 trajectory calculations: vector physics (PE, friction, magnitude)
- Unit tests for all calculations with J-Editorial formula verification

### Phase 2: Parser Adapter + Application Layer
- Create `doc-doctor-parser-yaml` crate implementing DocumentParser trait
- Frontmatter extraction with position tracking (line/column)
- Stub parsing for both compact (`- link: "desc"`) and structured syntax
- Create `doc-doctor-application` crate with use case implementations
- Rich error types with exact positions for diagnostics

### Phase 3: WASM Bindings
- Create `doc-doctor-wasm` crate with wasm-bindgen exports
- Wire domain + application + parser-yaml
- TypeScript type generation from Rust types
- npm package: `@doc-doctor/wasm`
- Performance benchmarks (target: <1ms per document)

### Phase 4: CLI Tool
- Refactor `doc-doctor-cli` as composition root
- All commands: parse, validate, stubs, dimensions, health, usefulness, batch, sync, schema
- Multiple output formats (human, JSON, YAML)
- Batch processing with rayon parallelism
- Shell completions (bash, zsh, fish)

### Phase 5: MCP Server
- Refactor `doc-doctor-mcp` using application layer
- MCP protocol (JSON-RPC over stdio)
- 13 tool handlers with JSON Schema definitions
- Claude Desktop configuration examples

### Phase 6: Obsidian Plugin Integration
- WASM integration in Obsidian plugin
- Replace TypeScript calculations with Rust/WASM calls
- Hybrid mode with TS fallback during validation
- Parity tests ensuring identical outputs
- Cache invalidation strategy (file modification time)

### Phase 7: LKO Adapter (Future)
- Create `doc-doctor-parser-lko` crate implementing DocumentParser trait
- Same port interface as YAML parser
- Zero changes to domain or application layers
- Demonstrates Hexagonal Architecture benefit

---

## 14. Testing Strategy

### Domain Layer Tests (Pure Unit Tests)
- **Location**: `doc-doctor-domain/tests/`
- **Scope**: All L2 calculations, entity validation
- **No mocks needed**: Pure functions with deterministic outputs
- **Formula verification**: Test against J-Editorial spec values

### Parser Adapter Tests
- **Location**: `doc-doctor-parser-yaml/tests/`
- **Fixtures**: Real documents from test vault
- **Edge cases**: Empty stubs, malformed YAML, missing fields
- **Position tracking**: Verify line/column accuracy

### Integration Tests
- **Location**: `doc-doctor-cli/tests/`
- **Scope**: End-to-end CLI command testing
- **Comparison**: Rust output vs TypeScript output parity

### WASM Tests
- **Location**: `doc-doctor-wasm/tests/`
- **Browser tests**: wasm-pack test --headless --chrome
- **Round-trip**: Parse → serialize → parse produces identical results

---

## 15. Success Criteria

### Architecture
1. **Domain isolation**: `doc-doctor-domain` has zero runtime dependencies
2. **Port abstraction**: Adding LKO parser requires only new adapter crate
3. **Clean layering**: No circular dependencies between crates

### Correctness
4. **J-Editorial parity**: All L2 calculations match spec formulas exactly
5. **Parser parity**: Rust produces identical results to TypeScript for all test fixtures
6. **Error positions**: All parse errors include accurate line:column positions

### Performance
7. **Per-document**: <1ms for full L1 parse + L2 calculation
8. **Batch processing**: 1000 documents in <10 seconds
9. **WASM cold start**: <100ms initialization

### Developer Experience
10. **Unified API**: Same functions available via CLI, MCP, and WASM
11. **TypeScript types**: Generated from Rust schemas
12. **Documentation**: All public APIs documented with examples
13. **Error messages**: Actionable with suggestions for resolution

---

## 16. Key Dependencies

**Rust:**
- `serde`, `serde_yaml`, `serde_json` - Serialization
- `thiserror` - Error handling
- `chrono` - DateTime handling
- `wasm-bindgen` - WASM bindings
- `napi-rs` - Node.js bindings (optional)
- `clap` - CLI parsing
- `glob` - File patterns
- `rayon` - Parallel processing

**MCP Server (doc-doctor-mcp):**
- `rmcp` or custom implementation
- `tokio` - Async runtime
- `tower` - Service framework

---

## 17. References

- [Hexagonal Architecture - Alistair Cockburn](https://alistair.cockburn.us/hexagonal-architecture/)
- [Clean Architecture - Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [J-Editorial Framework](https://jjjjguevara.vercel.app/j-editorial)
- [LKO Specification](./LKO-spec.md)
