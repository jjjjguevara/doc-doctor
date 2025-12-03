# PRD: Full MCP Integration & Custom Prompts

## Overview

Implement full MCP integration for the Obsidian plugin with:
1. MCP client for all document operations
2. Custom prompt schema with YAML configuration
3. All commands exposed to Obsidian command palette
4. Context-aware execution (selection vs full content)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     OBSIDIAN PLUGIN                              │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ Command      │  │ Prompt       │  │ Settings             │   │
│  │ Palette      │  │ Loader       │  │ Manager              │   │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘   │
│         │                 │                      │               │
│         └─────────────────┼──────────────────────┘               │
│                           │                                      │
│                           ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    MCP CLIENT                                ││
│  │                                                              ││
│  │  • Spawns dd-mcp binary                                     ││
│  │  • JSON-RPC communication                                   ││
│  │  • Tool execution                                           ││
│  │  • Connection health monitoring                             ││
│  └─────────────────────────────────────────────────────────────┘│
│                           │                                      │
└───────────────────────────┼──────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      MCP SERVER                                  │
│                     (dd-mcp binary)                              │
│                                                                  │
│  28 Tools: parse, analyze, add_stub, resolve_stub, etc.         │
└─────────────────────────────────────────────────────────────────┘
```

---

## 1. MCP Client

### File: `src/mcp/mcp-client.ts`

```typescript
interface MCPClientConfig {
  binaryPath: string;        // Path to dd-mcp binary
  timeout: number;           // Request timeout (ms)
  autoReconnect: boolean;    // Auto-reconnect on disconnect
}

interface MCPClient {
  // Lifecycle
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  isConnected(): boolean;

  // Tool execution
  callTool<T>(name: string, args: Record<string, unknown>): Promise<T>;
  listTools(): Promise<MCPTool[]>;

  // Events
  on(event: 'connected' | 'disconnected' | 'error', handler: Function): void;
}

// Usage example:
const client = new MCPClient({ binaryPath: '/path/to/dd-mcp' });
await client.connect();

const result = await client.callTool('add_stub', {
  content: documentContent,
  stub_type: 'expand',
  description: 'Add more details here',
});
```

### Auto-discovery of Binary

```typescript
// Search order for dd-mcp binary:
const BINARY_SEARCH_PATHS = [
  // 1. Configured path in settings
  settings.mcp?.binaryPath,
  // 2. Cargo bin (installed via cargo install)
  '~/.cargo/bin/dd-mcp',
  // 3. Homebrew (future)
  '/opt/homebrew/bin/dd-mcp',
  '/usr/local/bin/dd-mcp',
  // 4. Plugin folder
  `${plugin.manifest.dir}/bin/dd-mcp`,
];
```

---

## 2. Custom Prompt Schema

### Schema Definition

Prompts can be defined in YAML files within the vault:

```yaml
# .doc-doctor/prompts.yaml (or prompts/*.yaml)

# Single file with multiple prompts
prompts:
  - id: analyze-annotate
    name: "Analyze & Annotate"
    icon: "search"
    description: "Full document analysis with stub suggestions"
    category: analysis  # analysis | editing | review | custom

    # Context requirements
    context:
      requires_selection: false
      requires_file: true
      file_types: [md]

    # System prompt extension (appended to default)
    system_extension: |
      ## Task: Comprehensive Document Analysis

      Analyze the current document and identify areas that need work.
      For each issue found:
      1. Determine the appropriate stub type
      2. Identify the specific location (create an inline anchor)
      3. Write a clear, actionable description

      Focus on:
      - Missing context or background
      - Unsupported claims
      - Unclear sections
      - Missing connections

    # Optional: Override entire system prompt
    # system_override: |
    #   Custom complete system prompt...

    # Post-processing behavior
    behavior:
      confirm_before_apply: true
      auto_insert_anchors: true
      show_preview: true

  - id: expand-section
    name: "Expand Section"
    icon: "plus-circle"
    description: "Expand the selected section with more detail"
    category: editing

    context:
      requires_selection: true  # Must have text selected
      selection_type: block     # block | inline | any

    system_extension: |
      ## Task: Section Expansion

      The user has selected a section they want to expand.
      Analyze the selection and:
      1. Identify what's missing or underdeveloped
      2. Add expand stubs with specific guidance
      3. Suggest what content should be added

    behavior:
      confirm_before_apply: true
      show_preview: true

  - id: quick-citation
    name: "Find Citations"
    icon: "link"
    description: "Find citations for claims in selection"
    category: review
    hotkey: "Mod+Shift+C"  # Suggested default hotkey

    context:
      requires_selection: true

    system_extension: |
      ## Task: Citation Finding

      For the selected text, identify:
      1. Claims that need citations
      2. Suggest potential sources
      3. Add citation stubs with specific guidance

    behavior:
      confirm_before_apply: false  # Quick action
      auto_insert_anchors: true
```

### Unstructured Format (Simple)

For users who want simpler configuration:

```yaml
# .doc-doctor/prompts/my-prompt.yaml

# Simple format - just the extension
id: my-custom-prompt
name: "My Custom Analysis"

# Markdown content is the prompt
prompt: |
  Analyze this document for technical accuracy.
  Focus on:
  - API specifications
  - Code examples
  - Version numbers

  Flag anything that looks outdated.
```

### Prompt Loading

```typescript
interface PromptDefinition {
  id: string;
  name: string;
  icon?: string;
  description?: string;
  category: 'analysis' | 'editing' | 'review' | 'custom';

  context: {
    requires_selection: boolean;
    requires_file: boolean;
    selection_type?: 'block' | 'inline' | 'any';
    file_types?: string[];
  };

  // Prompt content
  system_extension?: string;
  system_override?: string;

  // Behavior
  behavior: {
    confirm_before_apply: boolean;
    auto_insert_anchors: boolean;
    show_preview: boolean;
  };

  // Optional hotkey suggestion
  hotkey?: string;

  // Source tracking
  source: 'builtin' | 'vault' | 'plugin';
  filePath?: string;
}

// Loader
class PromptLoader {
  private builtinPrompts: PromptDefinition[];
  private vaultPrompts: PromptDefinition[];

  async loadFromVault(vaultPath: string): Promise<void>;
  getAll(): PromptDefinition[];
  getById(id: string): PromptDefinition | undefined;
  getByCategory(category: string): PromptDefinition[];
  getContextualPrompts(context: ExecutionContext): PromptDefinition[];
}
```

---

## 3. Command Palette Integration

### Command Registry

All operations exposed as Obsidian commands:

```typescript
// Core document operations
'doc-doctor:parse-document'
'doc-doctor:analyze-document'
'doc-doctor:validate-document'
'doc-doctor:calculate-health'
'doc-doctor:calculate-usefulness'

// Stub operations
'doc-doctor:add-stub'              // Opens type selector
'doc-doctor:add-stub-expand'       // Direct type shortcuts
'doc-doctor:add-stub-link'
'doc-doctor:add-stub-verify'
'doc-doctor:add-stub-question'
'doc-doctor:add-stub-blocker'
'doc-doctor:resolve-stub'          // Resolve stub at cursor/selection
'doc-doctor:update-stub'           // Modify stub properties
'doc-doctor:list-stubs'            // Show all stubs

// Anchor operations
'doc-doctor:add-anchor'            // Add anchor at cursor
'doc-doctor:link-anchor'           // Link anchor to stub
'doc-doctor:unlink-anchor'         // Unlink anchor from stub
'doc-doctor:find-anchors'          // List all anchors

// AI operations (one per prompt)
'doc-doctor:ai-analyze-annotate'
'doc-doctor:ai-suggest-improvements'
'doc-doctor:ai-expand-section'
'doc-doctor:ai-find-citations'
'doc-doctor:ai-find-related'
'doc-doctor:ai-custom'             // Opens prompt selector

// Navigation
'doc-doctor:next-stub'
'doc-doctor:prev-stub'
'doc-doctor:goto-stub'             // Opens stub picker

// Vault operations
'doc-doctor:scan-vault'
'doc-doctor:find-blocking-stubs'
'doc-doctor:batch-analyze'
```

### Context-Aware Execution

```typescript
interface ExecutionContext {
  // File context
  file: TFile | null;
  content: string;

  // Selection context
  hasSelection: boolean;
  selection: string;
  selectionRange: { start: number; end: number };

  // Cursor context
  cursorPosition: number;
  currentLine: number;
  currentLineContent: string;

  // Stub context
  stubAtCursor: ParsedStub | null;
  anchorAtCursor: string | null;
}

// Command handler with context
function createContextAwareCommand(
  plugin: Plugin,
  promptId: string,
): Command {
  return {
    id: `doc-doctor:ai-${promptId}`,
    name: `Doc Doctor: ${prompt.name}`,

    // Check if command should be available
    checkCallback: (checking: boolean) => {
      const ctx = getExecutionContext(plugin);
      const prompt = promptLoader.getById(promptId);

      // Validate context requirements
      if (prompt.context.requires_file && !ctx.file) {
        return false;
      }
      if (prompt.context.requires_selection && !ctx.hasSelection) {
        return false;
      }

      if (!checking) {
        executePrompt(prompt, ctx);
      }
      return true;
    },
  };
}
```

### Dynamic Command Registration

```typescript
// Register commands dynamically based on loaded prompts
function registerPromptCommands(plugin: Plugin): void {
  const prompts = promptLoader.getAll();

  for (const prompt of prompts) {
    plugin.addCommand({
      id: `doc-doctor:ai-${prompt.id}`,
      name: `Doc Doctor: ${prompt.name}`,
      icon: prompt.icon,

      // Editor callback for selection-aware commands
      editorCheckCallback: (checking, editor, view) => {
        const ctx = buildContext(editor, view);

        // Check requirements
        if (!meetsRequirements(prompt, ctx)) {
          return false;
        }

        if (!checking) {
          executePromptWithMCP(prompt, ctx);
        }
        return true;
      },
    });
  }
}
```

---

## 4. Settings Additions

### New Settings Section

```typescript
interface MCPSettings {
  enabled: boolean;
  binaryPath: string;          // '' = auto-detect
  autoConnect: boolean;
  connectionTimeout: number;

  // Status (not persisted)
  connectionStatus?: 'connected' | 'disconnected' | 'error';
  lastError?: string;
}

interface PromptSettings {
  // Custom prompts location
  promptsPath: string;         // Default: '.doc-doctor/prompts'
  watchForChanges: boolean;    // Hot-reload prompts

  // Prompt behavior
  defaultCategory: string;
  showBuiltinPrompts: boolean;

  // Execution defaults
  confirmBeforeApply: boolean;
  showPreviewPanel: boolean;
  autoInsertAnchors: boolean;
}
```

### Settings UI

```typescript
// MCP Section
containerEl.createEl('h2', { text: 'MCP Integration' });

new Setting(containerEl)
  .setName('Enable MCP')
  .setDesc('Use MCP server for document operations (recommended)')
  .addToggle(toggle => toggle
    .setValue(settings.mcp.enabled)
    .onChange(value => updateSetting('mcp.enabled', value))
  );

new Setting(containerEl)
  .setName('Binary Path')
  .setDesc('Path to dd-mcp binary (leave empty for auto-detect)')
  .addText(text => text
    .setPlaceholder('Auto-detect')
    .setValue(settings.mcp.binaryPath)
  )
  .addButton(button => button
    .setButtonText('Test')
    .onClick(() => testMCPConnection())
  );

// Connection status indicator
const statusEl = containerEl.createDiv('mcp-status');
updateStatusIndicator(statusEl, settings.mcp.connectionStatus);

// Prompts Section
containerEl.createEl('h2', { text: 'Custom Prompts' });

new Setting(containerEl)
  .setName('Prompts Folder')
  .setDesc('Vault folder containing custom prompt definitions')
  .addText(text => text
    .setPlaceholder('.doc-doctor/prompts')
    .setValue(settings.prompts.promptsPath)
  );

new Setting(containerEl)
  .setName('Show Built-in Prompts')
  .setDesc('Include default prompts alongside custom ones')
  .addToggle(toggle => toggle
    .setValue(settings.prompts.showBuiltinPrompts)
  );

// List loaded prompts
const promptsList = containerEl.createDiv('prompts-list');
for (const prompt of promptLoader.getAll()) {
  const item = promptsList.createDiv('prompt-item');
  item.createSpan({ text: prompt.name });
  item.createSpan({ text: prompt.source, cls: 'prompt-source' });
}
```

---

## 5. Implementation Order

### Phase 1: MCP Client (Priority: High)
1. Create `src/mcp/mcp-client.ts`
2. Add MCP settings to settings type
3. Add MCP settings UI
4. Wire MCP client to plugin lifecycle

### Phase 2: Prompt Schema (Priority: High)
1. Create `src/llm/prompt-schema.ts` - type definitions
2. Create `src/llm/prompt-loader.ts` - YAML loader
3. Create `src/llm/builtin-prompts.ts` - default prompts
4. Add prompt settings to UI

### Phase 3: Command Integration (Priority: Medium)
1. Create `src/commands/ai-commands.ts`
2. Add context builder
3. Register all commands dynamically
4. Add hotkey suggestions

### Phase 4: Execution Flow (Priority: Medium)
1. Modify LLMService to use prompts
2. Add preview panel for confirmations
3. Wire MCP for stub operations
4. Add error handling and retry

---

## 6. Built-in Prompts

```yaml
# src/llm/builtin-prompts.yaml

prompts:
  - id: analyze-annotate
    name: "Analyze & Annotate"
    icon: "search"
    category: analysis
    context:
      requires_selection: false
      requires_file: true
    system_extension: |
      ## Task: Comprehensive Document Analysis
      ...
    behavior:
      confirm_before_apply: true
      show_preview: true

  - id: suggest-improvements
    name: "Suggest Improvements"
    icon: "lightbulb"
    category: analysis
    context:
      requires_selection: false
      requires_file: true
    system_extension: |
      ## Task: Prioritized Improvement Suggestions
      ...

  - id: expand-section
    name: "Expand Section"
    icon: "plus-circle"
    category: editing
    context:
      requires_selection: true
    system_extension: |
      ## Task: Section Expansion
      ...

  - id: find-citations
    name: "Find Citations"
    icon: "link"
    category: review
    context:
      requires_selection: true
    system_extension: |
      ## Task: Citation Discovery
      ...

  - id: find-related
    name: "Find Related Docs"
    icon: "file-search"
    category: analysis
    context:
      requires_selection: false
      requires_file: true
    system_extension: |
      ## Task: Document Connection Analysis
      ...

  - id: verify-claims
    name: "Verify Claims"
    icon: "check-circle"
    category: review
    context:
      requires_selection: true
    system_extension: |
      ## Task: Claim Verification
      ...
```

---

## 7. File Structure

```
src/
├── mcp/
│   ├── index.ts
│   ├── mcp-client.ts          # MCP client implementation
│   ├── mcp-types.ts           # MCP type definitions
│   └── mcp-tools.ts           # Tool wrapper functions
├── llm/
│   ├── index.ts
│   ├── llm-service.ts         # Modified to use prompts
│   ├── llm-prompts.ts         # Existing prompt builder
│   ├── prompt-schema.ts       # Prompt type definitions
│   ├── prompt-loader.ts       # YAML loader
│   └── builtin-prompts.yaml   # Default prompts
├── commands/
│   ├── ai-commands.ts         # AI command registration
│   ├── stub-commands.ts       # Stub command registration
│   └── context-builder.ts     # Execution context
└── settings/
    └── settings-type.ts       # Updated with MCP/prompt settings
```
