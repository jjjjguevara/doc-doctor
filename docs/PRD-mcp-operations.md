---
title: "PRD: MCP Server Operations"
refinement: 0.7
audience: internal
form: developing
origin: requirement
stubs:
  - type: expand
    description: "Add detailed JSON schemas for each tool in Appendix A"
  - type: review-needed
    description: "Review tool naming conventions with team"
---

# PRD: MCP Server Operations

## 1. Overview

### 1.1 Purpose

This document defines the MCP (Model Context Protocol) tools and agent workflows for Doc-Doctor, enabling AI-assisted document operations within the J-Editorial framework. The MCP server exposes deterministic tools that can be invoked by Claude or other LLM clients to analyze, modify, and manage documents in an Obsidian vault.

### 1.2 Scope

- **34 MCP tools** across 7 categories
- **Prompt system** for customizable AI operations
- **Plugin integrations** with Obsidian Git and Smart Connections
- **UI integration** via command palette, slash commands, and AI sidebar

### 1.3 Architectural Principles

Following the J-Editorial axiological foundations:

#### Planner-Executor Separation
A probabilistic planner (LLM) MAY propose actions, but a deterministic executor (code) MUST perform them.

```
┌─────────────────┐     ┌─────────────────┐
│  LLM (Planner)  │────▶│  MCP Tool       │
│  "Draft content │     │  (Executor)     │
│   for stub X"   │     │  Deterministic  │
└─────────────────┘     └─────────────────┘
```

#### Instrumentation First
All operations emit traces, metrics, and logs. A feature is incomplete without observability.

#### File-Anchored Contracts
All I/O defined by schema. Files are the single source of truth.

#### Quality Floors
Measurable targets, not subjective vibes. Every metric has a threshold.

#### Human-in-the-Loop
Confidence thresholds trigger review gates. No automation without audit trail.

---

## 2. Current State

### 2.1 Existing Tools (11)

The MCP server currently implements these tools in `crates/doc-doctor-mcp/src/tools/`:

| Tool | Category | Description |
|------|----------|-------------|
| `parse_document` | Analysis | Extract L1 properties from markdown with YAML frontmatter |
| `analyze_document` | Analysis | Full parse + L2 dimension calculation (health, usefulness) |
| `validate_document` | Analysis | Validate frontmatter against JSON schema |
| `list_stubs` | Analysis | List stubs with filtering by type and blocking status |
| `calculate_health` | Calculation | Formula: `health = 0.7×refinement + 0.3×(1-stub_penalty)` |
| `calculate_usefulness` | Calculation | Formula: `margin = refinement - audience_gate` |
| `calculate_dimensions` | Calculation | All L2 state dimensions in one call |
| `calculate_vector_physics` | Calculation | Stub prioritization using PE, friction, magnitude |
| `get_audience_gates` | Info | Return refinement thresholds per audience level |
| `get_schema` | Info | Retrieve JSON schemas for frontmatter or stubs |
| `batch_analyze` | Batch | Process multiple documents with aggregate statistics |

### 2.2 Gap Analysis

| Capability | Current State | Gap |
|------------|---------------|-----|
| File system access | Content passed as strings | Need `read_document` tool |
| Stub modification | Read-only | Need CRUD operations |
| AI content generation | Not available | Need prompt system |
| Vault-wide operations | Limited to batch_analyze | Need scan/search tools |
| Plugin integrations | None | Need Git and RAG bridges |

---

## 3. Tool Specifications

### 3.1 Category 1: Document Analysis

#### `read_document`
Load a file from path and return content with analysis.

```yaml
name: read_document
description: "Read a markdown file and return its content with L1/L2 analysis"

inputs:
  path:
    type: string
    description: "Absolute or vault-relative path to the document"
    required: true
  include_content:
    type: boolean
    description: "Include raw content in response"
    default: true

outputs:
  content: string
  properties: L1Properties
  dimensions: L2Dimensions
  health: number
```

#### `find_stub_anchors`
Match frontmatter stubs to inline `^anchor` references in content.

```yaml
name: find_stub_anchors
description: "Find inline anchors (^anchor-id) that correspond to stubs"

inputs:
  content:
    type: string
    description: "Document content with YAML frontmatter"
    required: true

outputs:
  matches:
    type: array
    items:
      stub_index: number
      stub_type: string
      anchor_id: string
      line_number: number
      context: string  # Surrounding text
  unmatched_stubs: array
  orphan_anchors: array
```

### 3.2 Category 2: Stub Management

#### `add_stub`
Insert a stub into document frontmatter.

```yaml
name: add_stub
description: "Add a stub to document frontmatter"

inputs:
  content:
    type: string
    description: "Document content with YAML frontmatter"
    required: true
  stub_type:
    type: string
    enum: [expand, verify, citation-needed, clarify, incomplete,
           example-needed, reorganize, merge, split, pov,
           blocker, dependency, question, todo, review-needed, update]
    required: true
  description:
    type: string
    required: true
  priority:
    type: string
    enum: [critical, high, medium, low]
    default: medium
  anchor:
    type: string
    description: "Inline anchor reference (^anchor-id)"
  stub_form:
    type: string
    enum: [transient, blocking, persistent]
    description: "Override default form for stub type"

outputs:
  updated_content: string
  stub_id: string
  stub_index: number
```

#### `resolve_stub`
Mark a stub as resolved and remove from frontmatter.

```yaml
name: resolve_stub
description: "Remove a resolved stub from frontmatter"

inputs:
  content:
    type: string
    required: true
  stub_index:
    type: number
    description: "Index of stub in stubs array"
    required: true
  resolution_note:
    type: string
    description: "Optional note about how stub was resolved"

outputs:
  updated_content: string
  removed_stub: Stub
```

#### `update_stub`
Modify an existing stub's properties.

```yaml
name: update_stub
description: "Update stub properties (status, priority, description)"

inputs:
  content:
    type: string
    required: true
  stub_index:
    type: number
    required: true
  updates:
    type: object
    properties:
      description: string
      priority: string
      stub_form: string
      anchor: string

outputs:
  updated_content: string
  previous_stub: Stub
  updated_stub: Stub
```

#### `link_stub_anchor`
Associate a stub with an inline anchor.

```yaml
name: link_stub_anchor
description: "Link a stub to an inline ^anchor in the content"

inputs:
  content:
    type: string
    required: true
  stub_index:
    type: number
    required: true
  anchor_id:
    type: string
    description: "Anchor ID without ^ prefix"
    required: true
  create_if_missing:
    type: boolean
    description: "Create anchor in content if not found"
    default: false
  insert_position:
    type: object
    description: "Where to insert anchor if creating"
    properties:
      line: number
      column: number

outputs:
  updated_content: string
  anchor_created: boolean
  anchor_location: object
```

### 3.3 Category 3: Content Operations

#### `run_prompt`
Execute a prompt template with document context.

```yaml
name: run_prompt
description: "Run a prompt template against the current document context"

inputs:
  prompt_id:
    type: string
    description: "ID of prompt from prompts registry"
    required: true
  document_context:
    type: object
    description: "Document properties for template variables"
    required: true
  additional_context:
    type: object
    description: "Extra variables to inject"
  user_instructions:
    type: string
    description: "User-provided additional instructions"

outputs:
  rendered_prompt: string
  prompt_metadata:
    name: string
    category: string
    description: string
```

#### `draft_for_stub`
Generate content to resolve a specific stub.

```yaml
name: draft_for_stub
description: "Generate draft content for a stub using appropriate prompt"

inputs:
  content:
    type: string
    required: true
  stub_index:
    type: number
    required: true
  context_documents:
    type: array
    description: "Related documents for RAG context (if available)"
    items:
      type: string
  user_instructions:
    type: string

outputs:
  draft_content: string
  suggested_insertion_point: object
  confidence: number
  sources_used: array
  integration_status:
    has_rag: boolean
    rag_documents_count: number
```

#### `find_citations`
Search for sources to support claims.

```yaml
name: find_citations
description: "Find citations for claims, searching vault and web"

inputs:
  claim:
    type: string
    description: "The claim or statement needing citation"
    required: true
  search_scope:
    type: string
    enum: [vault, web, both]
    default: both
  max_results:
    type: number
    default: 5

outputs:
  vault_sources:
    type: array
    items:
      path: string
      title: string
      relevance: number
      excerpt: string
  web_sources:
    type: array
    items:
      url: string
      title: string
      relevance: number
      excerpt: string
  integration_status:
    has_rag: boolean
```

#### `verify_claim`
Fact-check a claim against available sources.

```yaml
name: verify_claim
description: "Verify a factual claim against vault and web sources"

inputs:
  claim:
    type: string
    required: true
  context:
    type: string
    description: "Surrounding context for the claim"

outputs:
  verdict: string
  enum: [supported, contradicted, insufficient_evidence, partially_supported]
  confidence: number
  supporting_evidence: array
  contradicting_evidence: array
  recommendation: string
```

### 3.4 Category 4: Batch Operations

#### `scan_vault`
Analyze all markdown files in a directory.

```yaml
name: scan_vault
description: "Scan a directory for markdown files and analyze health"

inputs:
  path:
    type: string
    description: "Directory path to scan"
    required: true
  recursive:
    type: boolean
    default: true
  include_patterns:
    type: array
    items:
      type: string
    default: ["**/*.md"]
  exclude_patterns:
    type: array
    items:
      type: string
    default: ["**/node_modules/**", "**/.git/**"]

outputs:
  total_files: number
  files_with_frontmatter: number
  files_without_frontmatter: number
  average_health: number
  total_stubs: number
  blocking_stubs: number
  documents:
    type: array
    items:
      path: string
      health: number
      stub_count: number
```

#### `find_blocking_stubs`
List all blocking stubs across the vault.

```yaml
name: find_blocking_stubs
description: "Find all blocking stubs across multiple documents"

inputs:
  paths:
    type: array
    description: "Document paths to search, or directory"
    items:
      type: string
    required: true

outputs:
  blocking_stubs:
    type: array
    items:
      document_path: string
      document_title: string
      stub_index: number
      stub_type: string
      description: string
      priority: string
  total_count: number
  by_type: object
  by_priority: object
```

#### `detect_stale_documents`
Find documents past their form cadence threshold.

```yaml
name: detect_stale_documents
description: "Find documents that haven't been updated within their form cadence"

inputs:
  paths:
    type: array
    items:
      type: string
    required: true

outputs:
  stale_documents:
    type: array
    items:
      path: string
      title: string
      form: string
      expected_cadence_days: number
      days_since_modified: number
      staleness_ratio: number
  total_count: number
  by_form: object
```

### 3.5 Category 5: Prompt Management

#### `list_prompts`
List all available prompts from all sources.

```yaml
name: list_prompts
description: "List all prompts from bundled and user-defined sources"

inputs:
  category:
    type: string
    description: "Filter by category"
  include_templates:
    type: boolean
    description: "Include template content in response"
    default: false

outputs:
  prompts:
    type: array
    items:
      id: string
      name: string
      description: string
      category: string
      source: string  # "bundled" | "user" | filename
      aliases: array
      icon: string
```

#### `get_prompt`
Get a prompt template by ID.

```yaml
name: get_prompt
description: "Retrieve a prompt template by ID or alias"

inputs:
  id:
    type: string
    required: true

outputs:
  prompt:
    id: string
    name: string
    description: string
    template: string
    category: string
    variables: array
```

#### `preview_prompt`
Render a prompt with context (dry run).

```yaml
name: preview_prompt
description: "Preview prompt rendering without executing"

inputs:
  prompt_id:
    type: string
    required: true
  context:
    type: object
    required: true

outputs:
  rendered: string
  missing_variables: array
  warnings: array
```

### 3.6 Category 6: Git Integration

These tools require the Obsidian Git plugin to be installed.

#### `snapshot_before_edit`
Create a safety commit before modifying a document.

```yaml
name: snapshot_before_edit
description: "Create a commit snapshot before making changes"

inputs:
  path:
    type: string
    required: true
  message:
    type: string
    default: "Auto-snapshot before Doc-Doctor edit"

outputs:
  commit_hash: string
  success: boolean
  integration_available: boolean
```

#### `commit_stub_resolution`
Commit changes with a descriptive message about resolved stub.

```yaml
name: commit_stub_resolution
description: "Commit with message describing the resolved stub"

inputs:
  path:
    type: string
    required: true
  stub_type:
    type: string
    required: true
  stub_description:
    type: string
    required: true
  additional_message:
    type: string

outputs:
  commit_hash: string
  message: string
  success: boolean
```

#### `get_document_history`
List commits affecting a document.

```yaml
name: get_document_history
description: "Get git history for a specific document"

inputs:
  path:
    type: string
    required: true
  limit:
    type: number
    default: 10

outputs:
  commits:
    type: array
    items:
      hash: string
      message: string
      author: string
      date: string
  integration_available: boolean
```

#### `diff_document_versions`
Compare document across commits.

```yaml
name: diff_document_versions
description: "Show diff between document versions"

inputs:
  path:
    type: string
    required: true
  from_commit:
    type: string
    description: "Commit hash or 'HEAD~n'"
  to_commit:
    type: string
    default: "HEAD"

outputs:
  diff: string
  additions: number
  deletions: number
  changed_sections: array
```

### 3.7 Category 7: RAG Integration

These tools require Smart Connections plugin.

#### `find_related_documents`
Semantic search for similar content.

```yaml
name: find_related_documents
description: "Find semantically related documents using embeddings"

inputs:
  query:
    type: string
    description: "Search query or document content"
    required: true
  limit:
    type: number
    default: 5
  min_similarity:
    type: number
    default: 0.7

outputs:
  results:
    type: array
    items:
      path: string
      title: string
      similarity: number
      excerpt: string
  integration_available: boolean
  reminder: string  # Shown when SC not available
```

#### `draft_with_context`
Generate content using RAG from vault.

```yaml
name: draft_with_context
description: "Generate content with RAG context from related documents"

inputs:
  stub_description:
    type: string
    required: true
  document_context:
    type: object
    required: true
  max_context_documents:
    type: number
    default: 3

outputs:
  draft: string
  context_documents:
    type: array
    items:
      path: string
      relevance: number
      excerpt_used: string
  integration_status:
    has_rag: boolean
    fallback_used: boolean
    reminder: string
```

#### `suggest_links`
Find documents to link for `link` stubs.

```yaml
name: suggest_links
description: "Suggest documents to link based on content similarity"

inputs:
  content:
    type: string
    required: true
  existing_links:
    type: array
    items:
      type: string
  limit:
    type: number
    default: 5

outputs:
  suggestions:
    type: array
    items:
      path: string
      title: string
      relevance: number
      reason: string
  integration_available: boolean
```

#### `detect_duplicates`
Find semantically similar sections.

```yaml
name: detect_duplicates
description: "Find potentially duplicate content across vault"

inputs:
  content:
    type: string
    required: true
  threshold:
    type: number
    description: "Similarity threshold (0-1)"
    default: 0.85

outputs:
  duplicates:
    type: array
    items:
      path: string
      section: string
      similarity: number
      recommendation: string  # "merge" | "dedupe" | "reference"
  integration_available: boolean
```

---

## 4. Prompt System

### 4.1 File Discovery

The plugin discovers prompts from:
```
.doc-doctor/
  prompts/
    prompts.yaml       # Bundled defaults (structured, multi-prompt)
    my-custom.yaml     # User-created (structured or single)
    quick-draft.md     # Unstructured single prompt
```

### 4.2 File Formats

#### Structured Multi-Prompt File
```yaml
version: 1
prompts:
  - id: draft-expand
    name: "Draft Section"
    description: "Generate content for expand/incomplete stubs"
    aliases: [draft, expand, write]
    category: content
    icon: wand
    template: |
      You are drafting content for a J-Editorial document.

      Document Context:
      - Title: {{title}}
      - Audience: {{audience}}
      - Form: {{form}}

      Stub: {{stub.description}}

      {{user_instructions}}

  - id: find-citations
    name: "Find Citations"
    description: "Search for sources to support claims"
    aliases: [cite, source, reference]
    category: research
    icon: search
    template: |
      Find authoritative sources for:
      {{selected_text}}
```

#### Single-Prompt File
```yaml
name: "Quick Draft"
description: "Fast content generation"
template: |
  Write a draft section about: {{topic}}
  Style: {{audience}} audience, {{form}} document
```

#### Unstructured File (Markdown)
```markdown
You are an expert technical writer.
Generate content for: {{stub.description}}
```

### 4.3 Template Variables

| Variable | Type | Description |
|----------|------|-------------|
| `title` | string | Document title |
| `audience` | enum | personal, internal, trusted, public |
| `form` | enum | transient, developing, stable, evergreen, canonical |
| `refinement` | number | 0.0-1.0 |
| `origin` | string | Document origin |
| `tags` | array | Document tags |
| `stub.type` | string | Current stub type |
| `stub.description` | string | Stub description |
| `stub.priority` | string | Stub priority |
| `stub.anchor` | string | Linked anchor |
| `stub.context` | string | Additional context |
| `selected_text` | string | User-selected text |
| `cursor_position` | object | Line and column |
| `user_instructions` | string | User-provided instructions |

### 4.4 UI Display

| Format | UI Display |
|--------|------------|
| Structured | Name, description, icon, category grouping |
| Single-prompt | Filename as name, template preview |
| Unstructured | Filename, "Custom prompt" label |

---

## 5. Plugin Integration

### 5.1 Command Palette Actions

| Command | Shortcut | Tool | UI |
|---------|----------|------|-----|
| Doc Doctor: Analyze Document | Cmd+Shift+H | `analyze_document` | - |
| Doc Doctor: Add Stub | Cmd+Shift+S | `add_stub` | Stub picker |
| Doc Doctor: Resolve Stub | - | `resolve_stub` | Stub list |
| Doc Doctor: Scan Vault Health | - | `scan_vault` | Dashboard |
| Doc Doctor: Draft for Stub | - | `draft_for_stub` | Prompt selector |
| Doc Doctor: Find Citations | - | `find_citations` | Results panel |

### 5.2 Slash Commands

| Trigger | Tool | Description |
|---------|------|-------------|
| `/health` | `analyze_document` | Check current document health |
| `/stubs` | `list_stubs` | List all stubs in document |
| `/add-stub {type} {description}` | `add_stub` | Add a stub to frontmatter |
| `/resolve {index}` | `resolve_stub` | Mark stub as resolved |
| `/draft {index}` | `draft_for_stub` | Draft content for a stub |
| `/cite {claim}` | `find_citations` | Find citations for a claim |
| `/verify {claim}` | `verify_claim` | Fact-check a claim |
| `/related` | `find_related_documents` | Find related documents |
| `/publish-check` | (workflow) | Run publication readiness check |

### 5.3 AI Sidebar Buttons

**Quick Actions Group:**
- Health Check (heart-pulse icon)
- View Stubs (list-checks icon)
- Add Stub (plus-circle icon)

**Content Assist Group:**
- Draft Section (wand icon) - requires selected stub
- Find Citations (search icon) - requires selected text
- Verify Claim (check-circle icon) - requires selected text

**Integration Group (conditional):**
- Snapshot (git-commit icon) - if Git available
- Find Related (brain icon) - if Smart Connections available

---

## 6. Example Workflows

### 6.1 Context-Aware Stub Resolution

```
User: /resolve-stub expand:api-docs

1. [Git] snapshot_before_edit → Create safety commit
2. [RAG] find_related_documents → Find similar API docs in vault
3. [Draft] draft_with_context → Generate content using RAG context
4. [User] Review and approve draft
5. [Edit] Apply changes to document
6. [Stub] resolve_stub → Mark stub as resolved
7. [Git] commit_stub_resolution → Commit "Resolved expand:api-docs"
```

### 6.2 Citation Discovery with Provenance

```
User: /cite "performance improved by 40%"

1. [RAG] find_related_documents → Search vault for performance data
2. [Web] find_citations → Search web for external sources
3. [Present] Show combined results with source types
4. [User] Select citations to add
5. [Git] snapshot_before_edit → Safety commit
6. [Edit] Add citations to document
7. [Stub] Add citation-needed stub if no good sources found
```

### 6.3 Publish Gate with Full Audit

```
User: /publish-check

1. [Analysis] analyze_document → Get health score
2. [Stubs] list_stubs → Check for blocking stubs
3. [RAG] detect_duplicates → Check for redundant content
4. [Git] get_document_history → Show recent changes
5. [Report] Generate publish readiness report:
   - Health: 85% ✓
   - Blocking stubs: 0 ✓
   - Duplicates: None ✓
   - Last commit: 2h ago
   - Recommendation: Ready for publish
```

### 6.4 Smart Reorganization

```
User: /reorganize

1. [Git] snapshot_before_edit → Safety commit
2. [RAG] find_related_documents → Find structurally similar docs
3. [Analysis] Analyze current document structure
4. [Generate] suggest_reorganization → Propose new structure
5. [User] Review and approve structure
6. [Edit] Apply reorganization
7. [Git] commit_stub_resolution → Commit "Reorganized"
```

---

## 7. Implementation Roadmap

### Phase 1: Stub Management Tools
- `add_stub`, `resolve_stub`, `update_stub`, `link_stub_anchor`
- Basic command palette integration
- Stub picker UI component

### Phase 2: Prompt System
- Prompt file discovery and parsing
- `list_prompts`, `get_prompt`, `preview_prompt`
- Template rendering with Handlebars

### Phase 3: Content Operations
- `run_prompt`, `draft_for_stub`
- `find_citations`, `verify_claim`
- Slash command handlers

### Phase 4: Batch & Vault Operations
- `scan_vault`, `find_blocking_stubs`, `detect_stale_documents`
- Dashboard integration
- Progress indicators

### Phase 5: Plugin Integrations
- Git bridge (Obsidian Git)
- RAG bridge (Smart Connections)
- Graceful degradation patterns

---

## Appendix A: Tool Summary

| Category | Tool | Status |
|----------|------|--------|
| Analysis | `parse_document` | Exists |
| Analysis | `analyze_document` | Exists |
| Analysis | `validate_document` | Exists |
| Analysis | `list_stubs` | Exists |
| Analysis | `read_document` | **NEW** |
| Analysis | `find_stub_anchors` | **NEW** |
| Calculation | `calculate_health` | Exists |
| Calculation | `calculate_usefulness` | Exists |
| Calculation | `calculate_dimensions` | Exists |
| Calculation | `calculate_vector_physics` | Exists |
| Info | `get_audience_gates` | Exists |
| Info | `get_schema` | Exists |
| Batch | `batch_analyze` | Exists |
| Batch | `scan_vault` | **NEW** |
| Batch | `find_blocking_stubs` | **NEW** |
| Batch | `detect_stale_documents` | **NEW** |
| Stub | `add_stub` | **NEW** |
| Stub | `resolve_stub` | **NEW** |
| Stub | `update_stub` | **NEW** |
| Stub | `link_stub_anchor` | **NEW** |
| Content | `run_prompt` | **NEW** |
| Content | `draft_for_stub` | **NEW** |
| Content | `find_citations` | **NEW** |
| Content | `verify_claim` | **NEW** |
| Prompt | `list_prompts` | **NEW** |
| Prompt | `get_prompt` | **NEW** |
| Prompt | `preview_prompt` | **NEW** |
| Git | `snapshot_before_edit` | **NEW** |
| Git | `commit_stub_resolution` | **NEW** |
| Git | `get_document_history` | **NEW** |
| Git | `diff_document_versions` | **NEW** |
| RAG | `find_related_documents` | **NEW** |
| RAG | `draft_with_context` | **NEW** |
| RAG | `suggest_links` | **NEW** |
| RAG | `detect_duplicates` | **NEW** |

**Total: 34 tools** (11 existing + 23 new)

---

## Appendix B: Default Prompt Templates

See `.doc-doctor/prompts/prompts.yaml` for bundled defaults.

---

## Appendix C: J-Editorial Vector Type Mapping

| Stub Type | Agent Family | Default Form | Default Origin |
|-----------|--------------|--------------|----------------|
| `expand` | Generative | transient | author-identified |
| `incomplete` | Generative | transient | author-identified |
| `verify` | Retrieval | transient | qa-detected |
| `citation-needed` | Retrieval | transient | qa-detected |
| `clarify` | Synoptic | transient | peer-surfaced |
| `reorganize` | Operational | transient | author-identified |
| `blocker` | Operational | blocking | author-identified |
| `dependency` | Operational | blocking | author-identified |
| `question` | Synoptic | transient | author-identified |
| `review-needed` | Synoptic | blocking | system-generated |
| `update` | Operational | persistent | system-generated |
