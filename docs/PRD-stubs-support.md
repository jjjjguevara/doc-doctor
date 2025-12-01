# PRD: Configurable Stubs Support for Enhanced Annotations

**Version**: 0.2.0
**Date**: 2025-12-01
**Author**: Claude (with Josué Guevara)
**Status**: Draft - Revised with User Feedback

---

## 1. Overview

### 1.1 Problem Statement

Writers and knowledge workers need a way to **acknowledge and track gaps** in their documents—missing citations, sections to expand, questions to resolve, blocking issues. While J-Editorial provides a comprehensive "stubs" framework for this, there is no Obsidian plugin that:

1. Provides first-class UI for managing stubs in the sidebar
2. Supports configurable stub vocabularies (not hard-coded types)
3. Enables click-to-navigate from stub list to content location
4. Integrates seamlessly with existing annotation workflows
5. Supports bidirectional sync between inline anchors and frontmatter

### 1.2 Proposed Solution

Extend the **obsidian-enhanced-annotations** plugin to support a configurable stubs system that:

- Adds a "Stubs" section to the existing sidebar panel (same UX patterns as annotations)
- Allows users to define their own stub type enums (2 levels of nesting max)
- Supports both compact and structured YAML syntax
- Provides inline decorations with color highlighting (matching annotation behavior)
- Implements bidirectional sync between `^stub-*` anchors and frontmatter
- Auto-generates anchor IDs (like Obsidian's "copy link to block") with manual override

### 1.3 Target User

**Primary**: Solo knowledge workers using Obsidian for:
- Technical documentation
- Academic writing
- Personal knowledge management
- Any workflow requiring systematic gap tracking

**Non-goal**: Multi-user collaboration features, hard-coded J-Editorial logic

---

## 2. Goals & Non-Goals

### 2.1 Goals

| ID | Goal | Success Metric |
|----|------|----------------|
| G1 | Add stubs as a first-class feature alongside annotations | Stubs appear in sidebar with same UX patterns as annotations |
| G2 | Support configurable stub type vocabulary (2 levels) | Users can define arbitrary stub types and sub-properties |
| G3 | Enable compact and structured stub syntax | Both `- type: "desc"` and `- type: { ... }` parse correctly |
| G4 | Provide click-to-navigate from sidebar to content | Clicking stub scrolls editor to `^stub-*` anchor location |
| G5 | Highlight `^stub-*` anchors with type-specific colors | Same inline decoration behavior as annotations |
| G6 | Implement bidirectional sync | Inline anchors ↔ frontmatter stay synchronized |
| G7 | Auto-generate anchor IDs matching Obsidian semantics | Like "copy link to block" with manual override support |
| G8 | Maintain backward compatibility | Existing annotation features continue working unchanged |

### 2.2 Non-Goals

| ID | Non-Goal | Rationale |
|----|----------|-----------|
| NG1 | Hard-code J-Editorial types/logic | Keep plugin generic; users configure their vocabulary |
| NG2 | Multi-user collaboration features | Target is solo workflow; collaboration adds complexity |
| NG3 | Frontmatter color storage | Colors are UI concerns, stored in plugin settings |
| NG4 | More than 2 levels of nesting | Match Obsidian's default YAML handling |
| NG5 | L2/L3 calculations (stub_health, etc.) | J-Editorial-specific; users can implement via Dataview |

---

## 3. User Stories

### 3.1 Core Workflows

**US-1: Configure stub vocabulary**
> As a user, I want to define my own stub types (e.g., "link", "expand", "question") in the sidebar settings, so that the vocabulary matches my workflow.

**US-2: Create stub via command**
> As a user, I want to invoke an "Insert Stub" command that shows a dropdown of my configured stub types, lets me enter a description, auto-generates an anchor ID, and inserts both the frontmatter entry and inline anchor.

**US-3: Manual anchor ID**
> As a user, I want to manually type `^stub-my-custom-id` in the document and have it recognized as a stub anchor, similar to how Obsidian handles manual block IDs.

**US-4: Browse active stubs**
> As a user, I want to see all stubs in the current document listed in the sidebar, grouped by type, with color-coded indicators matching my configuration.

**US-5: Navigate to stub location**
> As a user, I want to click a stub in the sidebar and have the editor scroll to the corresponding `^stub-*` anchor in the document.

**US-6: See inline highlighting**
> As a user, I want `^stub-*` anchors in my document to be highlighted with the color of their stub type, just like annotation highlights work.

**US-7: Bidirectional sync**
> As a user, I want changes to inline anchors to reflect in frontmatter and vice versa, keeping the two in sync.

**US-8: Create structured stub**
> As a user, I want to create stubs with additional properties (form, priority) using structured syntax in frontmatter.

### 3.2 Edge Cases

**US-9: Mixed format support**
> As a user, I want to mix compact and structured stubs in the same document.

**US-10: Invalid stub handling**
> As a user, I want clear error messages when stubs have invalid types or malformed syntax.

**US-11: Orphaned anchors**
> As a user, I want to be notified when a `^stub-*` anchor exists without a corresponding frontmatter entry (and vice versa).

**US-12: Anchor collision avoidance**
> As a user, I don't want stub anchors to collide with footnotes (`[^1]`) or content block links (`![[page#^block]]`).

---

## 4. Functional Requirements

### 4.1 Anchor Syntax Design

To avoid collisions with Obsidian's footnotes and block references, stub anchors use a distinctive prefix:

```
^stub-<id>
```

**Valid examples**:
- `^stub-abc123` (auto-generated)
- `^stub-my-custom-id` (manual)
- `^stub-link-001` (type-prefixed auto)

**Collision avoidance**:
| Obsidian Feature | Syntax | Our Prefix |
|------------------|--------|------------|
| Footnotes | `[^1]`, `[^note]` | `^stub-` (no brackets) |
| Block references | `^block-id` | `^stub-` prefix distinguishes |
| Content embeds | `![[page#^block]]` | Different context |

**Recognition rule**: Any `^stub-` followed by alphanumeric/hyphen/underscore is a stub anchor.

### 4.2 Configuration Schema

Users configure stubs in plugin settings (2 levels max):

```typescript
interface StubsConfiguration {
  // Whether stubs feature is enabled
  enabled: boolean;

  // Top-level frontmatter key (default: "stubs")
  frontmatterKey: string;

  // Level 1: Defined stub types with display properties
  stubTypes: StubTypeDefinition[];

  // Level 2: Structured format property definitions
  structuredProperties: StructuredPropertyDefinition[];

  // Editor decoration settings (matching annotation behavior)
  decorations: DecorationSettings;

  // Anchor generation settings
  anchors: AnchorSettings;

  // Sidebar display settings
  sidebar: SidebarSettings;
}

interface StubTypeDefinition {
  // The YAML key (e.g., "link", "expand", "question")
  key: string;

  // Display name in UI (e.g., "Citation Needed", "Expand Section")
  displayName: string;

  // Color for sidebar and inline highlighting (hex or CSS color)
  color: string;

  // Optional icon (Lucide icon name)
  icon?: string;

  // Default values for Level 2 properties
  defaults?: Record<string, unknown>;
}

interface StructuredPropertyDefinition {
  // Property key in YAML (e.g., "stub_form", "priority")
  key: string;

  // Display name in UI
  displayName: string;

  // Property type
  type: 'string' | 'enum' | 'array' | 'boolean';

  // For enum type: allowed values (configurable)
  enumValues?: string[];

  // Whether property is required
  required?: boolean;
}

interface AnchorSettings {
  // Prefix for stub anchors (default: "stub")
  prefix: string;

  // ID generation style
  idStyle: 'random' | 'sequential' | 'type-prefixed';

  // Length of random IDs
  randomIdLength: number;
}

interface DecorationSettings {
  // Show inline highlighting for ^stub-* anchors
  showInlineHighlights: boolean;

  // Highlight style: 'background' | 'underline' | 'gutter'
  highlightStyle: string;

  // Opacity for highlights (0-1)
  highlightOpacity: number;
}
```

### 4.3 Default Configuration

Out-of-the-box configuration matching J-Editorial Tier 1 types:

```yaml
# Plugin Settings (stored in data.json)
stubs:
  enabled: true
  frontmatterKey: "stubs"

  # Level 1: Stub types (user-configurable enum)
  stubTypes:
    - key: "link"
      displayName: "Citation Needed"
      color: "#e67e22"
      icon: "link"
      defaults:
        stub_form: "persistent"
    - key: "clarify"
      displayName: "Clarify"
      color: "#3498db"
      icon: "help-circle"
    - key: "expand"
      displayName: "Expand"
      color: "#2ecc71"
      icon: "plus-circle"
    - key: "question"
      displayName: "Question"
      color: "#9b59b6"
      icon: "message-circle"
    - key: "controversy"
      displayName: "Controversy"
      color: "#e74c3c"
      icon: "alert-triangle"
      defaults:
        stub_form: "blocking"
    - key: "blocker"
      displayName: "Blocker"
      color: "#c0392b"
      icon: "octagon"
      defaults:
        stub_form: "blocking"

  # Level 2: Structured properties (user-configurable enums)
  structuredProperties:
    - key: "stub_form"
      displayName: "Form"
      type: "enum"
      enumValues: ["transient", "persistent", "blocking", "structural"]
    - key: "priority"
      displayName: "Priority"
      type: "enum"
      enumValues: ["low", "medium", "high", "critical"]
    - key: "assignees"
      displayName: "Assignees"
      type: "array"
    - key: "inline_anchors"
      displayName: "Anchors"
      type: "array"

  anchors:
    prefix: "stub"
    idStyle: "random"
    randomIdLength: 6

  decorations:
    showInlineHighlights: true
    highlightStyle: "background"
    highlightOpacity: 0.3
```

### 4.4 YAML Syntax Support

**Compact syntax** (type-as-key with string value):

```yaml
stubs:
  - link: "Add citation for performance claims"
    anchor: "^stub-abc123"
  - expand: "Add deployment examples"
    anchor: "^stub-def456"
```

**Structured syntax** (type-as-key with object value):

```yaml
stubs:
  - controversy:
      description: "Pricing model disagreement"
      stub_form: blocking
      priority: high
      anchor: "^stub-pricing"
```

**Inline anchors in document**:

```markdown
Our API handles 10,000 requests per second. ^stub-abc123

## Deployment

This section needs more detail. ^stub-def456

The pricing model is under discussion. ^stub-pricing
```

### 4.5 Bidirectional Sync Algorithm

#### 4.5.1 Sync Triggers

Sync runs on:
1. Document save
2. Frontmatter modification
3. Anchor insertion/deletion in content
4. Manual sync command

#### 4.5.2 Sync Logic

```typescript
interface SyncResult {
  added: StubSyncAction[];      // Anchors found, stubs created
  removed: StubSyncAction[];    // Anchors deleted, stubs removed
  orphaned: OrphanedItem[];     // Mismatches requiring user attention
  updated: StubSyncAction[];    // Anchor positions updated
}

function syncStubsAndAnchors(
  frontmatterStubs: ParsedStub[],
  inlineAnchors: InlineAnchor[],
  config: StubsConfiguration
): SyncResult {
  const result: SyncResult = { added: [], removed: [], orphaned: [], updated: [] };

  const stubAnchors = new Set(frontmatterStubs.map(s => s.anchor));
  const documentAnchors = new Set(inlineAnchors.map(a => a.id));

  // Find orphaned frontmatter stubs (anchor deleted from content)
  for (const stub of frontmatterStubs) {
    if (stub.anchor && !documentAnchors.has(stub.anchor)) {
      result.orphaned.push({
        type: 'frontmatter_orphan',
        stub,
        message: `Stub "${stub.description}" references anchor ${stub.anchor} which no longer exists`,
      });
    }
  }

  // Find orphaned inline anchors (no corresponding frontmatter)
  for (const anchor of inlineAnchors) {
    if (anchor.id.startsWith(`^${config.anchors.prefix}-`)) {
      if (!stubAnchors.has(anchor.id)) {
        result.orphaned.push({
          type: 'anchor_orphan',
          anchor,
          message: `Anchor ${anchor.id} has no corresponding frontmatter stub`,
        });
      }
    }
  }

  return result;
}
```

#### 4.5.3 Orphan Resolution UI

When orphans are detected, show notification with options:
- **Frontmatter orphan**: "Delete stub" or "Re-insert anchor at cursor"
- **Anchor orphan**: "Create stub for anchor" or "Delete anchor"

### 4.6 Sidebar UI Requirements

#### 4.6.1 Stubs Panel

- New "Stubs" icon in sidebar icon menu (alongside existing annotations icon)
- Clicking expands stubs panel with same UX as annotations:
  - Header: "Stubs (N)" where N is count
  - Filter/search input
  - Grouped list by stub type
  - Color indicators matching configuration

#### 4.6.2 Stub Type Groups

For each configured stub type with active stubs:

```
▼ Citation Needed (3)          [●] orange
  ├─ "Add citation for..." [^stub-abc123]   [click → navigate]
  ├─ "Reference OAuth spec" [^stub-def456]
  └─ "Cite performance data" [^stub-perf01]

▼ Expand (2)                   [●] green
  ├─ "Add deployment steps" [^stub-deploy]
  └─ "Include error handling" [^stub-errors]

▶ Question (0)                 [●] purple [collapsed/dimmed]
```

#### 4.6.3 Stub Item Display

Each stub item shows:
- Stub description (truncated with tooltip for full text)
- Anchor ID (`^stub-*`)
- Color indicator matching stub type
- Click behavior: scroll editor to anchor location
- Context menu: Edit, Delete, Copy Anchor

#### 4.6.4 Create Stub Action

- "+" button in panel header OR invoke command
- Dropdown shows configured stub types (same pattern as annotations)
- Flow:
  1. User selects stub type from dropdown
  2. Modal/inline input for description
  3. Option: "Auto-generate ID" (default) or "Custom ID" input
  4. On submit:
     - Inserts `^stub-<id>` at cursor position
     - Adds stub entry to frontmatter with anchor reference
     - Syncs immediately

### 4.7 Inline Decoration Behavior

**Matching annotation plugin behavior**:

1. **Color highlighting**: `^stub-*` text gets background color matching stub type
2. **Same opacity/style settings** as annotations
3. **Hover tooltip**: Shows stub type and description
4. **Click behavior**: Could highlight corresponding sidebar entry

**CSS example**:
```css
/* Generated per stub type */
.stub-anchor-link { background-color: rgba(230, 126, 34, 0.3); }
.stub-anchor-clarify { background-color: rgba(52, 152, 219, 0.3); }
.stub-anchor-expand { background-color: rgba(46, 204, 113, 0.3); }
```

### 4.8 Commands

| Command | Description |
|---------|-------------|
| `stubs:insert` | Open stub type dropdown, create stub at cursor |
| `stubs:insert-<type>` | Create stub of specific type directly (one per configured type) |
| `stubs:list` | Focus sidebar stubs panel |
| `stubs:next` | Navigate to next stub anchor in document |
| `stubs:previous` | Navigate to previous stub anchor in document |
| `stubs:sync` | Manually trigger bidirectional sync |
| `stubs:resolve-orphans` | Show orphan resolution UI |

---

## 5. Technical Architecture

### 5.1 Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Enhanced Annotations Plugin               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Annotations │  │    Stubs     │  │     Settings     │  │
│  │    Module    │  │    Module    │  │      Module      │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │            │
│         └────────┬────────┴────────────────────┘            │
│                  │                                          │
│         ┌────────▼────────┐                                 │
│         │  Shared Services │                                │
│         │  - Parser        │                                │
│         │  - Decorations   │ ← Same decoration engine       │
│         │  - Sidebar       │                                │
│         │  - Commands      │                                │
│         │  - Sync Engine   │ ← NEW: Bidirectional sync      │
│         └─────────────────┘                                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 New Files Structure

```
src/
├── stubs/
│   ├── stubs-module.ts           # Module orchestration
│   ├── stubs-parser.ts           # YAML frontmatter parsing
│   ├── stubs-store.ts            # Reactive state management
│   ├── stubs-commands.ts         # Editor commands
│   ├── stubs-decorations.ts      # CodeMirror decorations (reuse annotation patterns)
│   ├── stubs-sync.ts             # Bidirectional sync engine
│   │
│   ├── components/
│   │   ├── stubs-panel.svelte    # Sidebar panel (mirrors annotations)
│   │   ├── stub-list.svelte      # Grouped stub list
│   │   ├── stub-item.svelte      # Individual stub row
│   │   ├── stub-create-modal.svelte
│   │   ├── stub-type-dropdown.svelte  # Type selector (like annotation labels)
│   │   └── orphan-resolver.svelte     # Orphan resolution UI
│   │
│   └── helpers/
│       ├── anchor-generator.ts   # ID generation (like Obsidian block IDs)
│       ├── anchor-parser.ts      # Find ^stub-* in content
│       ├── stub-defaults.ts      # Default config
│       └── stub-validation.ts    # Schema validation
│
├── settings/
│   └── stubs-settings.ts         # Settings tab additions
│
└── shared/
    ├── frontmatter-utils.ts      # Shared YAML utilities
    └── decoration-utils.ts       # Shared decoration patterns
```

### 5.3 Key Interfaces

```typescript
// Parsed stub representation
interface ParsedStub {
  // Identification
  id: string;                      // Generated hash
  type: string;                    // The stub type key (Level 1)
  description: string;

  // Anchor linking
  anchor: string | null;           // ^stub-<id> reference
  anchorResolved: boolean;         // Whether anchor was found in content

  // Level 2 properties (if structured)
  properties: Record<string, unknown>;

  // Parsing metadata
  syntax: 'compact' | 'structured';
  lineInFrontmatter: number;
}

interface InlineAnchor {
  // The anchor ID (e.g., "^stub-abc123")
  id: string;

  // Position in document
  line: number;
  ch: number;

  // Whether it has a corresponding frontmatter stub
  hasStub: boolean;

  // The stub type (if linked)
  stubType?: string;
}

// Sync state
interface SyncState {
  // All stubs from frontmatter
  stubs: ParsedStub[];

  // All ^stub-* anchors from content
  anchors: InlineAnchor[];

  // Linked pairs
  linked: Map<string, { stub: ParsedStub; anchor: InlineAnchor }>;

  // Orphans
  orphanedStubs: ParsedStub[];      // Frontmatter stub, no anchor
  orphanedAnchors: InlineAnchor[];  // Anchor, no frontmatter stub
}

// Store state
interface StubsState {
  // Sync state
  sync: SyncState;

  // Grouped by type for sidebar
  byType: Map<string, ParsedStub[]>;

  // Loading/error state
  loading: boolean;
  error: string | null;

  // UI state
  expandedTypes: Set<string>;
  selectedStubId: string | null;
  filterText: string;
}
```

### 5.4 Anchor ID Generation

Matching Obsidian's "copy link to block" behavior:

```typescript
function generateAnchorId(
  config: AnchorSettings,
  stubType?: string
): string {
  const prefix = config.prefix; // "stub"

  switch (config.idStyle) {
    case 'random':
      // Like Obsidian: random alphanumeric
      return `^${prefix}-${randomAlphanumeric(config.randomIdLength)}`;

    case 'sequential':
      // Auto-increment: stub-001, stub-002
      return `^${prefix}-${getNextSequentialId()}`;

    case 'type-prefixed':
      // Include type: stub-link-abc123
      return `^${prefix}-${stubType}-${randomAlphanumeric(4)}`;

    default:
      return `^${prefix}-${randomAlphanumeric(6)}`;
  }
}

function randomAlphanumeric(length: number): string {
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}
```

### 5.5 Integration Points

#### With Existing Annotation System

| Component | Integration |
|-----------|-------------|
| Sidebar | Add stubs icon to icon bar; stubs panel as peer to annotations panel |
| Settings Tab | Add "Stubs" section; mirror annotation settings patterns |
| Decorations | **Reuse decoration infrastructure**; add stub-specific color rules |
| Commands | Register alongside existing commands |
| Color Picker | Same color picker component as annotations |

#### With Obsidian APIs

| API | Usage |
|-----|-------|
| `MarkdownView` | Get/set frontmatter, cursor position |
| `Editor` | Insert anchors, scroll to position |
| `MetadataCache` | Watch for frontmatter changes |
| `Vault.on('modify')` | Trigger sync on file change |
| `Workspace` | Panel registration, leaf management |
| `Plugin.addCommand` | Register commands |

---

## 6. Settings UI Design

### 6.1 Settings Tab Layout

```
Enhanced Annotations Settings

▼ Annotations
  [existing annotation settings]

▼ Stubs                                    [NEW SECTION]

  ─────────────────────────────────────────────────────

  [x] Enable Stubs Feature

  Frontmatter Key
  ┌─────────────────────────────────────┐
  │ stubs                               │
  └─────────────────────────────────────┘

  ─────────────────────────────────────────────────────

  ▼ Stub Types (Level 1)

    Define the stub type vocabulary for your workflow.

    ┌─────────────────────────────────────────────────────┐
    │ Key         │ Display Name     │ Color   │ Actions │
    ├─────────────────────────────────────────────────────┤
    │ link        │ Citation Needed  │ [●]#e67 │ ✎ ↑↓ 🗑 │
    │ clarify     │ Clarify          │ [●]#349 │ ✎ ↑↓ 🗑 │
    │ expand      │ Expand           │ [●]#2ec │ ✎ ↑↓ 🗑 │
    │ question    │ Question         │ [●]#9b5 │ ✎ ↑↓ 🗑 │
    │ controversy │ Controversy      │ [●]#e74 │ ✎ ↑↓ 🗑 │
    │ blocker     │ Blocker          │ [●]#c03 │ ✎ ↑↓ 🗑 │
    └─────────────────────────────────────────────────────┘
    [+ Add Type]

  ─────────────────────────────────────────────────────

  ▼ Structured Properties (Level 2)

    Properties available when using structured stub syntax.

    ┌─────────────────────────────────────────────────────┐
    │ Key         │ Display Name │ Type   │ Enum Values  │
    ├─────────────────────────────────────────────────────┤
    │ stub_form   │ Form         │ enum   │ [Edit]       │
    │ priority    │ Priority     │ enum   │ [Edit]       │
    │ assignees   │ Assignees    │ array  │ -            │
    └─────────────────────────────────────────────────────┘
    [+ Add Property]

  ─────────────────────────────────────────────────────

  ▼ Anchor Settings

    Anchor Prefix
    ┌─────────────────────────────────────┐
    │ stub                                │
    └─────────────────────────────────────┘
    Anchors will be: ^stub-<id>

    ID Generation Style
    (•) Random (like Obsidian): ^stub-a1b2c3
    ( ) Type-prefixed: ^stub-link-a1b2
    ( ) Sequential: ^stub-001, ^stub-002

    Random ID Length [====•=] 6

  ─────────────────────────────────────────────────────

  ▼ Inline Highlighting

    [x] Highlight ^stub-* anchors in editor

    Same settings as annotation highlights (shared)

  ─────────────────────────────────────────────────────

  ▼ Import/Export

    [Export Configuration]  [Import Configuration]
```

### 6.2 Type Editor Modal

When clicking "Edit" (✎) on a stub type:

```
┌────────────────────────────────────────────┐
│ Edit Stub Type                         [x] │
├────────────────────────────────────────────┤
│                                            │
│ Type Key (YAML)                            │
│ ┌────────────────────────────────────────┐ │
│ │ link                                   │ │
│ └────────────────────────────────────────┘ │
│ Used in frontmatter: - link: "..."         │
│                                            │
│ Display Name                               │
│ ┌────────────────────────────────────────┐ │
│ │ Citation Needed                        │ │
│ └────────────────────────────────────────┘ │
│                                            │
│ Color                    [Same picker as   │
│ [●] #e67e22  [Pick]       annotations]     │
│                                            │
│ Icon (Lucide)                              │
│ ┌────────────────────────────────────────┐ │
│ │ link                    [🔗 preview]   │ │
│ └────────────────────────────────────────┘ │
│                                            │
│ Default Properties (Level 2)               │
│ ┌────────────────────────────────────────┐ │
│ │ stub_form: persistent                  │ │
│ └────────────────────────────────────────┘ │
│                                            │
│              [Cancel]  [Save]              │
└────────────────────────────────────────────┘
```

### 6.3 Enum Editor Modal

When editing enum values for a Level 2 property:

```
┌────────────────────────────────────────────┐
│ Edit Enum: stub_form                   [x] │
├────────────────────────────────────────────┤
│                                            │
│ Allowed Values                             │
│ ┌────────────────────────────────────────┐ │
│ │ transient                    [↑] [🗑]  │ │
│ │ persistent                   [↑] [🗑]  │ │
│ │ blocking                     [↑] [🗑]  │ │
│ │ structural                   [↑] [🗑]  │ │
│ └────────────────────────────────────────┘ │
│ ┌────────────────────────────────────────┐ │
│ │ Add new value...                       │ │
│ └────────────────────────────────────────┘ │
│                                            │
│              [Cancel]  [Save]              │
└────────────────────────────────────────────┘
```

---

## 7. Implementation Phases

### Phase 1: Core Infrastructure (MVP)

**Scope**:
- Configuration schema and settings UI (both levels)
- Frontmatter parser (compact + structured syntax)
- Basic sidebar panel with stub listing (mirror annotations UX)
- Anchor parser (find `^stub-*` in content)
- Basic click-to-navigate

**Deliverables**:
- `stubs-parser.ts` with tests
- `stubs-store.ts` reactive state
- Settings tab with type/property configuration
- Basic `stubs-panel.svelte`
- `anchor-parser.ts`

### Phase 2: Bidirectional Sync

**Scope**:
- Sync engine implementation
- Orphan detection and resolution UI
- Auto-generate anchor IDs
- Insert stub command with dropdown

**Deliverables**:
- `stubs-sync.ts`
- `anchor-generator.ts`
- `orphan-resolver.svelte`
- Command registration

### Phase 3: Inline Decorations

**Scope**:
- CodeMirror decorations for `^stub-*` anchors
- Color highlighting matching stub type
- Hover tooltips
- Reuse annotation decoration patterns

**Deliverables**:
- `stubs-decorations.ts`
- Shared decoration utilities
- CSS generation per stub type

### Phase 4: Polish & Edge Cases

**Scope**:
- Error handling and validation
- Unknown type handling
- Performance optimization
- Import/export configuration
- Documentation

**Deliverables**:
- Validation utilities
- Config import/export
- User documentation

---

## 8. Success Criteria

### 8.1 Functional Acceptance

- [ ] User can configure custom stub types (Level 1) in settings
- [ ] User can configure structured properties with custom enums (Level 2)
- [ ] User can create stubs via command with type dropdown
- [ ] Anchor IDs auto-generate like Obsidian's block IDs
- [ ] Manual `^stub-custom-id` anchors are recognized
- [ ] Stubs appear in sidebar grouped by type with correct colors
- [ ] Clicking stub navigates to `^stub-*` anchor in document
- [ ] `^stub-*` anchors are highlighted with type colors (like annotations)
- [ ] Bidirectional sync keeps frontmatter and anchors in sync
- [ ] Orphaned stubs/anchors are detected with resolution UI
- [ ] Compact and structured syntax both work
- [ ] Settings persist across sessions
- [ ] Configuration can be exported/imported

### 8.2 Performance Criteria

- [ ] Sidebar updates within 100ms of change
- [ ] Sync completes within 200ms for typical documents
- [ ] Documents with 100+ stubs remain responsive
- [ ] No memory leaks on document switching

### 8.3 Compatibility Criteria

- [ ] Existing annotation features unchanged
- [ ] No collision with footnotes or block references
- [ ] Works with Obsidian 1.0.0+
- [ ] No conflicts with Dataview, Templater

---

## 9. Appendix

### A. Example Documents

#### A.1 Simple Document with Compact Stubs

```yaml
---
title: "API Documentation"
stubs:
  - link: "Add OAuth 2.0 specification reference"
    anchor: "^stub-oauth"
  - expand: "Add rate limiting examples"
    anchor: "^stub-ratelimit"
  - question: "Should we deprecate v1 endpoints?"
    anchor: "^stub-deprecate"
---

# API Documentation

## Authentication

Our API uses OAuth 2.0 for authentication. ^stub-oauth

## Rate Limiting

Requests are limited to prevent abuse. ^stub-ratelimit

## Deprecation

Older endpoints may be deprecated. ^stub-deprecate
```

#### A.2 Document with Structured Stubs

```yaml
---
title: "Architecture Decision"
stubs:
  - controversy:
      description: "Microservices vs monolith for v1"
      stub_form: blocking
      priority: critical
      anchor: "^stub-arch-decision"
  - link:
      description: "CAP theorem reference needed"
      anchor: "^stub-cap"
  - expand: "Add deployment diagram"
    anchor: "^stub-deploy"
---

# Architecture Decision Record

## Introduction

We need to decide on the system architecture. ^stub-arch-decision

## Trade-offs

Microservices offer flexibility but add complexity.
The CAP theorem constrains our choices. ^stub-cap

## Deployment

Deployment process goes here. ^stub-deploy
```

### B. Anchor Syntax Comparison

| Feature | Obsidian Native | Our Stub Anchors |
|---------|-----------------|------------------|
| Footnote | `[^1]`, `[^note]` | N/A (different) |
| Block ID | `^block-id` | `^stub-<id>` |
| Block embed | `![[page#^block]]` | Not applicable |
| Auto-generate | "Copy link to block" | "Insert stub" command |
| Manual ID | Type `^my-id` | Type `^stub-my-id` |

### C. References

- [J-Editorial Stubs Standard](../../../Llull/Standards/J-Editorial/framework/02-practice/stubs/spec-stubs-standard.md)
- [J-Editorial Inline Syntax](../../../Llull/Standards/J-Editorial/framework/02-practice/stubs/spec-inline-syntax.md)
- [obsidian-enhanced-annotations GitHub](https://github.com/ycnmhd/obsidian-enhanced-annotations)

---

**Document History**:
- 2025-12-01 v0.1.0: Initial draft
- 2025-12-01 v0.2.0: Revised with user feedback
  - Added bidirectional sync as core feature
  - Clarified `^stub-` prefix for collision avoidance
  - Added anchor auto-generation matching Obsidian semantics
  - Confirmed 2-level configuration (both levels customizable)
  - Added inline highlighting matching annotation behavior
