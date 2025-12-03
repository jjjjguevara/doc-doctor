# PRD: Obsidian Plugin Agentic UI

## Overview

Design the AI-assisted editorial features for the Doc Doctor Obsidian Plugin, enabling users to leverage LLM capabilities for document analysis, stub management, and editorial guidance.

---

## Current State

The plugin has a triple toggle UI:
- **Annotations** - Visual overlays for document signals
- **Stubs** - Stub management panel
- **AI Features** - (To be designed)

---

## Design Goals

1. **Guided Workflows** - Preset prompts for common editorial tasks
2. **Context-Aware** - Agent understands current document state and J-Editorial semantics
3. **Tool-Enabled** - Agent can use MCP tools to read/modify documents
4. **Non-Destructive** - Preview changes before applying

---

## Architecture Pattern: Agent with System Prompt + Tool Binding

```
┌─────────────────────────────────────────────────────────────┐
│                    OBSIDIAN PLUGIN UI                        │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  [Annotations] [Stubs] [AI ▼]                       │    │
│  │                         ┌──────────────────────────┐│    │
│  │                         │ ● Analyze & Annotate     ││    │
│  │                         │ ● Suggest Improvements   ││    │
│  │                         │ ● Expand Section         ││    │
│  │                         │ ● Find Related Docs      ││    │
│  │                         │ ● Custom Prompt...       ││    │
│  │                         └──────────────────────────┘│    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     AGENT LAYER                              │
│                                                              │
│  ┌──────────────────┐  ┌──────────────────────────────────┐ │
│  │   SYSTEM PROMPT  │  │         CONTEXT INJECTION        │ │
│  │                  │  │                                  │ │
│  │  Role definition │  │  • Current document content      │ │
│  │  Capabilities    │  │  • Parsed L1 properties          │ │
│  │  Constraints     │  │  • Current stubs                 │ │
│  │  Output format   │  │  • Health/usefulness scores      │ │
│  │                  │  │  • Vault context (optional)      │ │
│  └──────────────────┘  └──────────────────────────────────┘ │
│                              │                               │
│                              ▼                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                    TOOL BINDING                       │   │
│  │                                                       │   │
│  │  Available MCP Tools:                                 │   │
│  │  • add_stub, resolve_stub, update_stub               │   │
│  │  • link_stub_anchor, unlink_stub_anchor              │   │
│  │  • find_stub_anchors, list_stubs                     │   │
│  │  • analyze_document, validate_document               │   │
│  │  • find_related_documents, suggest_links             │   │
│  │                                                       │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      MCP SERVER                              │
│                   (doc-doctor-mcp)                           │
└─────────────────────────────────────────────────────────────┘
```

---

## System Prompt Design

The system prompt establishes the agent's identity, capabilities, and operational constraints.

### Core System Prompt

```markdown
You are an editorial assistant integrated into Obsidian, working with the J-Editorial document framework. Your role is to help authors refine raw drafts and work-in-progress documents through structured annotation and guidance.

## Your Capabilities

You have access to document analysis and modification tools:

### Analysis Tools
- `analyze_document` - Parse content and calculate quality dimensions
- `validate_document` - Check frontmatter against J-Editorial schema
- `list_stubs` - View existing stubs and their status
- `find_stub_anchors` - Find inline anchor references (^anchor-id)
- `calculate_health` - Get document health score
- `calculate_usefulness` - Get usefulness margin for audiences

### Modification Tools
- `add_stub` - Insert a stub into the frontmatter
- `resolve_stub` - Mark a stub as resolved (remove it)
- `update_stub` - Modify stub properties
- `link_stub_anchor` - Connect a stub to an inline anchor
- `unlink_stub_anchor` - Disconnect a stub from an anchor

### Context Tools
- `find_related_documents` - Find semantically similar documents
- `suggest_links` - Recommend wiki-links based on content

## J-Editorial Framework

Documents in this system have:
- **Refinement** (0.0-1.0): How complete/polished the document is
- **Audience**: self → team → internal → external (gates at 0.6, 0.75, 0.85)
- **Form**: signal → draft → reference → standard (lifecycle stages)
- **Stubs**: Embedded signals marking incomplete sections

### Stub Types
- `expand` - Section needs more detail
- `link` - Needs connection to related documents
- `verify` - Claims need fact-checking
- `cite` - Needs source citations
- `revise` - Writing needs improvement
- `question` - Open questions to resolve
- `example` - Needs concrete examples
- `define` - Terms need definition

## Operational Guidelines

1. **Analyze Before Acting**: Always analyze the document first to understand its current state
2. **Use Anchors**: When adding stubs, create inline anchors (^anchor-id) at specific locations in the content to mark exactly where work is needed
3. **Be Specific**: Stub descriptions should be actionable and specific
4. **Respect Audience Gates**: Consider the target audience when suggesting improvements
5. **Preserve Voice**: Don't rewrite content; annotate what needs attention
6. **Explain Reasoning**: When adding stubs, explain why this area needs work

## Output Format

When suggesting changes:
1. Show the current document state (health, refinement, stub count)
2. List proposed stubs with:
   - Type and description
   - Target location (line or section)
   - Reasoning
3. Ask for confirmation before applying changes
```

### Preset-Specific Prompt Extensions

Each preset prompt extends the base system prompt with specific instructions:

#### "Analyze & Annotate"
```markdown
## Task: Comprehensive Document Analysis

Analyze the current document and identify areas that need work. For each issue found:
1. Determine the appropriate stub type
2. Identify the specific location (create an inline anchor)
3. Write a clear, actionable description
4. Explain why this matters for the document's goals

Focus on:
- Missing context or background
- Unsupported claims
- Unclear sections
- Missing connections to related topics
- Gaps in the logical flow

After analysis, present a summary and the proposed stubs. Wait for user approval before making changes.
```

#### "Suggest Improvements"
```markdown
## Task: Editorial Improvement Suggestions

Based on the document's current refinement level and target audience, suggest specific improvements:

1. First, calculate the current health and usefulness margins
2. Identify the biggest gaps between current state and the next audience gate
3. Prioritize suggestions by impact on health score
4. For each suggestion, propose a stub with specific guidance

Present suggestions ranked by priority. Do not add stubs automatically - present options for the user to choose from.
```

#### "Expand Section"
```markdown
## Task: Section Expansion Guidance

The user will indicate a section they want to expand. Your role is to:

1. Analyze the current section content
2. Identify what's missing or underdeveloped
3. Add `expand` stubs with specific guidance on what to add
4. Suggest related documents that might provide useful context

Create inline anchors at specific points where expansion is needed. Each stub should answer: "What specific content should be added here?"
```

#### "Find Related Docs"
```markdown
## Task: Document Connection Analysis

Analyze the current document and find related documents in the vault:

1. Use `find_related_documents` to get semantically similar docs
2. Use `suggest_links` to find potential wiki-link opportunities
3. For each relevant connection, propose a `link` stub
4. Explain how the linked document relates to the current one

Present findings as a list of potential connections with relevance explanations.
```

---

## UI Implementation

### Dropdown Structure

```typescript
interface AIPreset {
  id: string;
  label: string;
  icon: string;
  promptExtension: string;
  requiresSelection?: boolean; // For section-specific actions
  confirmBeforeApply?: boolean;
}

const AI_PRESETS: AIPreset[] = [
  {
    id: 'analyze-annotate',
    label: 'Analyze & Annotate',
    icon: '🔍',
    promptExtension: ANALYZE_ANNOTATE_PROMPT,
    confirmBeforeApply: true
  },
  {
    id: 'suggest-improvements',
    label: 'Suggest Improvements',
    icon: '💡',
    promptExtension: SUGGEST_IMPROVEMENTS_PROMPT,
    confirmBeforeApply: true
  },
  {
    id: 'expand-section',
    label: 'Expand Section',
    icon: '📝',
    promptExtension: EXPAND_SECTION_PROMPT,
    requiresSelection: true,
    confirmBeforeApply: true
  },
  {
    id: 'find-related',
    label: 'Find Related Docs',
    icon: '🔗',
    promptExtension: FIND_RELATED_PROMPT,
    confirmBeforeApply: false
  },
  {
    id: 'custom',
    label: 'Custom Prompt...',
    icon: '✏️',
    promptExtension: '', // User provides
    confirmBeforeApply: true
  }
];
```

### Interaction Flow

```
1. User clicks AI dropdown
2. Selects preset (e.g., "Analyze & Annotate")
3. Plugin constructs message:
   - System prompt (base + preset extension)
   - Context (current document content, parsed state)
   - Tool bindings (MCP tool schemas)
4. Send to LLM API
5. Agent analyzes and proposes changes
6. Show preview panel with proposed stubs
7. User approves/modifies/rejects
8. Apply approved changes via MCP tools
```

### Preview Panel Design

```
┌─────────────────────────────────────────────────────────┐
│ AI Analysis Results                              [×]    │
├─────────────────────────────────────────────────────────┤
│ Current State:                                          │
│   Health: 0.62  |  Refinement: 0.55  |  Stubs: 3        │
│   Audience: team  |  Form: draft                        │
├─────────────────────────────────────────────────────────┤
│ Proposed Changes:                                       │
│                                                         │
│ ☑ [expand] Line 42: "Add implementation details..."    │
│   → Links to ^impl-details anchor                       │
│   Reason: This section mentions the algorithm but...    │
│                                                         │
│ ☑ [cite] Line 67: "Add source for performance claim"   │
│   → Links to ^perf-cite anchor                          │
│   Reason: The 10x improvement claim needs...            │
│                                                         │
│ ☐ [link] Line 12: "Connect to architecture.md"         │
│   Reason: The design patterns mentioned here are...     │
│                                                         │
├─────────────────────────────────────────────────────────┤
│              [Apply Selected]  [Cancel]                 │
└─────────────────────────────────────────────────────────┘
```

---

## Agent File Structure

For the Claude Code / Agent SDK pattern, we could use a `.claude/agents/` structure:

```
.claude/
├── agents/
│   └── editorial-assistant.md    # Agent definition
└── commands/
    └── analyze-doc.md            # Slash command using agent
```

### `editorial-assistant.md`

```markdown
---
name: Editorial Assistant
description: J-Editorial document analysis and annotation
tools:
  - add_stub
  - resolve_stub
  - update_stub
  - link_stub_anchor
  - unlink_stub_anchor
  - find_stub_anchors
  - list_stubs
  - analyze_document
  - validate_document
  - find_related_documents
  - suggest_links
---

# Editorial Assistant

You are an editorial assistant for the J-Editorial document framework...
[Full system prompt here]
```

---

## Configuration Options

Allow users to customize agent behavior:

```yaml
# .obsidian/plugins/doc-doctor/settings.yaml
ai:
  # LLM provider configuration
  provider: anthropic  # or openai, local
  model: claude-sonnet-4-20250514

  # Behavior settings
  auto_analyze_on_open: false
  confirm_before_apply: true
  max_stubs_per_analysis: 10

  # Context settings
  include_vault_context: true
  related_docs_limit: 5

  # Custom presets
  custom_presets:
    - id: team-review
      label: "Prepare for Team Review"
      prompt: |
        Focus on clarity and completeness for team consumption.
        Target audience gate: 0.75
        ...
```

---

## Implementation Phases

### Phase 1: Basic Integration
- [ ] System prompt definition
- [ ] Single preset: "Analyze & Annotate"
- [ ] Preview panel for proposed changes
- [ ] Apply/reject workflow

### Phase 2: Full Preset Suite
- [ ] All 5 preset prompts
- [ ] Context injection (current doc state)
- [ ] Tool execution via MCP

### Phase 3: Advanced Features
- [ ] Custom preset creation UI
- [ ] Vault-wide context (RAG integration)
- [ ] History/undo for AI changes
- [ ] Batch operations across documents

---

## Success Metrics

1. **Adoption**: % of users who use AI features
2. **Accuracy**: % of suggested stubs that users accept
3. **Efficiency**: Time saved vs manual annotation
4. **Quality**: Improvement in document health scores after AI assistance

---

## Open Questions

1. **LLM Provider**: Ship with specific provider or support multiple?
2. **Cost Model**: How to handle API costs (user's key vs embedded)?
3. **Offline Mode**: Support local LLMs for privacy-conscious users?
4. **Learning**: Should the agent learn from user preferences over time?
