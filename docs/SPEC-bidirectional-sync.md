# Specification: Bidirectional Sync Engine

**Version**: 0.1.0
**Date**: 2025-12-01
**Status**: Draft

---

## 1. Overview

This document specifies the bidirectional synchronization engine that keeps frontmatter stubs and inline `^stub-*` anchors in sync.

### 1.1 Core Principle

**Single source of truth**: The frontmatter `stubs:` array is the canonical source. Inline anchors are **references** to frontmatter entries. The sync engine ensures they stay consistent.

### 1.2 Sync Scenarios

| Scenario | Trigger | Action |
|----------|---------|--------|
| Create stub via command | User invokes "Insert Stub" | Add to frontmatter + insert anchor at cursor |
| Manual anchor typed | User types `^stub-custom-id` | Detect orphan, prompt to create frontmatter entry |
| Anchor deleted | User deletes `^stub-*` from content | Detect orphan, prompt to delete or re-insert |
| Frontmatter stub deleted | User removes stub from YAML | Detect orphan anchor, prompt to delete or recreate |
| Anchor moved | User cuts/pastes anchor | Update position tracking |
| Document opened | File loaded | Full sync check |

---

## 2. Data Model

### 2.1 Frontmatter Stub

```typescript
interface FrontmatterStub {
  // Stub type key (Level 1)
  type: string;

  // Description (compact syntax value or structured.description)
  description: string;

  // Anchor reference (e.g., "^stub-abc123")
  anchor: string;

  // Level 2 properties (if structured)
  properties: Record<string, unknown>;

  // Parsing metadata
  syntax: 'compact' | 'structured';
  frontmatterLine: number;
}
```

### 2.2 Inline Anchor

```typescript
interface InlineAnchor {
  // The full anchor ID (e.g., "^stub-abc123")
  id: string;

  // Position in document content (after frontmatter)
  position: {
    line: number;      // 0-indexed line in content
    ch: number;        // Character offset in line
    offset: number;    // Absolute offset from content start
  };

  // The line content where anchor appears
  lineContent: string;

  // Whether at end of line (valid position) or mid-line (unusual)
  isEndOfLine: boolean;
}
```

### 2.3 Sync State

```typescript
interface SyncState {
  // Parsed frontmatter stubs
  frontmatterStubs: FrontmatterStub[];

  // Found inline anchors
  inlineAnchors: InlineAnchor[];

  // Successfully linked pairs
  linked: LinkedPair[];

  // Orphaned items
  orphanedStubs: FrontmatterStub[];   // Stub with no matching anchor
  orphanedAnchors: InlineAnchor[];    // Anchor with no matching stub

  // Sync metadata
  lastSyncTime: number;
  syncErrors: SyncError[];
}

interface LinkedPair {
  stub: FrontmatterStub;
  anchor: InlineAnchor;
}

interface SyncError {
  type: 'parse_error' | 'anchor_collision' | 'invalid_anchor_format';
  message: string;
  location?: { line: number; ch: number };
}
```

---

## 3. Anchor Detection

### 3.1 Anchor Pattern

```typescript
// Regex for detecting stub anchors
const STUB_ANCHOR_PATTERN = /\^stub-[a-zA-Z0-9_-]+/g;

// With configurable prefix
function getAnchorPattern(prefix: string): RegExp {
  return new RegExp(`\\^${prefix}-[a-zA-Z0-9_-]+`, 'g');
}
```

### 3.2 Anchor Parser

```typescript
function parseInlineAnchors(
  content: string,
  config: AnchorSettings
): InlineAnchor[] {
  const anchors: InlineAnchor[] = [];
  const pattern = getAnchorPattern(config.prefix);
  const lines = content.split('\n');

  let offset = 0;

  for (let lineNum = 0; lineNum < lines.length; lineNum++) {
    const line = lines[lineNum];
    let match: RegExpExecArray | null;

    // Reset pattern for each line
    pattern.lastIndex = 0;

    while ((match = pattern.exec(line)) !== null) {
      const anchorId = match[0];
      const ch = match.index;

      // Check if at end of line (standard position)
      const isEndOfLine = ch + anchorId.length >= line.trimEnd().length;

      anchors.push({
        id: anchorId,
        position: {
          line: lineNum,
          ch,
          offset: offset + ch,
        },
        lineContent: line,
        isEndOfLine,
      });
    }

    offset += line.length + 1; // +1 for newline
  }

  return anchors;
}
```

### 3.3 Collision Detection

Stub anchors should not collide with:

```typescript
function isValidStubAnchor(
  anchorId: string,
  content: string,
  config: AnchorSettings
): { valid: boolean; reason?: string } {
  const prefix = `^${config.prefix}-`;

  // Must start with configured prefix
  if (!anchorId.startsWith(prefix)) {
    return { valid: false, reason: 'Missing stub prefix' };
  }

  // Check for footnote collision (brackets)
  if (content.includes(`[${anchorId}]`)) {
    return { valid: false, reason: 'Conflicts with footnote syntax' };
  }

  // Check for duplicate anchor IDs
  const pattern = new RegExp(anchorId.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g');
  const matches = content.match(pattern);
  if (matches && matches.length > 1) {
    return { valid: false, reason: 'Duplicate anchor ID' };
  }

  return { valid: true };
}
```

---

## 4. Sync Algorithm

### 4.1 Full Sync

Performed on document open and manual sync command:

```typescript
async function performFullSync(
  file: TFile,
  config: StubsConfiguration
): Promise<SyncResult> {
  const content = await vault.read(file);

  // 1. Parse frontmatter
  const { frontmatter, contentStart } = parseFrontmatter(content);
  const frontmatterStubs = parseStubsFrontmatter(frontmatter, config);

  // 2. Parse content for anchors
  const documentContent = content.slice(contentStart);
  const inlineAnchors = parseInlineAnchors(documentContent, config.anchors);

  // 3. Build anchor lookup
  const anchorMap = new Map(inlineAnchors.map(a => [a.id, a]));
  const stubAnchorSet = new Set(frontmatterStubs.map(s => s.anchor).filter(Boolean));

  // 4. Find linked pairs and orphans
  const linked: LinkedPair[] = [];
  const orphanedStubs: FrontmatterStub[] = [];
  const orphanedAnchors: InlineAnchor[] = [];

  // Check each frontmatter stub
  for (const stub of frontmatterStubs) {
    if (stub.anchor) {
      const anchor = anchorMap.get(stub.anchor);
      if (anchor) {
        linked.push({ stub, anchor });
      } else {
        orphanedStubs.push(stub);
      }
    }
  }

  // Check each inline anchor
  for (const anchor of inlineAnchors) {
    if (anchor.id.startsWith(`^${config.anchors.prefix}-`)) {
      if (!stubAnchorSet.has(anchor.id)) {
        orphanedAnchors.push(anchor);
      }
    }
  }

  return {
    linked,
    orphanedStubs,
    orphanedAnchors,
    frontmatterStubs,
    inlineAnchors,
  };
}
```

### 4.2 Incremental Sync

Performed on document modification:

```typescript
interface DocumentChange {
  from: number;       // Start offset
  to: number;         // End offset
  inserted: string;   // New text
  removed: string;    // Deleted text
}

function handleDocumentChange(
  change: DocumentChange,
  currentState: SyncState,
  config: StubsConfiguration
): SyncAction[] {
  const actions: SyncAction[] = [];

  // Check if change affects an anchor
  const affectedAnchors = currentState.inlineAnchors.filter(anchor => {
    const anchorStart = anchor.position.offset;
    const anchorEnd = anchorStart + anchor.id.length;
    return (change.from <= anchorEnd && change.to >= anchorStart);
  });

  // Check if new anchor was inserted
  const pattern = getAnchorPattern(config.anchors.prefix);
  const newAnchors = change.inserted.match(pattern);

  if (newAnchors) {
    for (const anchorId of newAnchors) {
      // Check if it's a new orphan
      const hasStub = currentState.frontmatterStubs.some(s => s.anchor === anchorId);
      if (!hasStub) {
        actions.push({
          type: 'new_orphan_anchor',
          anchorId,
          suggestedAction: 'prompt_create_stub',
        });
      }
    }
  }

  // Check if anchor was deleted
  for (const anchor of affectedAnchors) {
    if (!change.inserted.includes(anchor.id)) {
      const stub = currentState.frontmatterStubs.find(s => s.anchor === anchor.id);
      if (stub) {
        actions.push({
          type: 'anchor_deleted',
          stub,
          suggestedAction: 'prompt_delete_or_reinsert',
        });
      }
    }
  }

  return actions;
}
```

### 4.3 Sync Actions

```typescript
type SyncAction =
  | { type: 'new_orphan_anchor'; anchorId: string; suggestedAction: 'prompt_create_stub' }
  | { type: 'anchor_deleted'; stub: FrontmatterStub; suggestedAction: 'prompt_delete_or_reinsert' }
  | { type: 'stub_deleted'; anchor: InlineAnchor; suggestedAction: 'prompt_delete_anchor' }
  | { type: 'position_updated'; stub: FrontmatterStub; newPosition: Position }
  | { type: 'sync_complete'; stats: SyncStats };

interface SyncStats {
  linkedCount: number;
  orphanedStubsCount: number;
  orphanedAnchorsCount: number;
  errorsCount: number;
}
```

---

## 5. Orphan Resolution

### 5.1 Resolution Strategies

#### Frontmatter Orphan (stub exists, anchor missing)

Options:
1. **Delete stub**: Remove from frontmatter
2. **Re-insert anchor**: Insert `^stub-<id>` at cursor position
3. **Ignore**: Keep stub without anchor (allowed but warned)

```typescript
async function resolveFrontmatterOrphan(
  stub: FrontmatterStub,
  strategy: 'delete' | 'reinsert' | 'ignore',
  editor?: Editor
): Promise<void> {
  switch (strategy) {
    case 'delete':
      await removeStubFromFrontmatter(stub);
      break;

    case 'reinsert':
      if (!editor) throw new Error('Editor required for reinsert');
      const cursor = editor.getCursor();
      const line = editor.getLine(cursor.line);
      // Insert at end of line (like Obsidian block IDs)
      editor.replaceRange(` ${stub.anchor}`, { line: cursor.line, ch: line.length });
      break;

    case 'ignore':
      // Mark as acknowledged orphan (stored in plugin data)
      await markOrphanAcknowledged(stub.anchor);
      break;
  }
}
```

#### Anchor Orphan (anchor exists, stub missing)

Options:
1. **Create stub**: Add frontmatter entry for this anchor
2. **Delete anchor**: Remove from content
3. **Convert to regular block ID**: Remove `stub-` prefix

```typescript
async function resolveAnchorOrphan(
  anchor: InlineAnchor,
  strategy: 'create_stub' | 'delete' | 'convert',
  stubType?: string,
  description?: string
): Promise<void> {
  switch (strategy) {
    case 'create_stub':
      if (!stubType || !description) {
        throw new Error('Stub type and description required');
      }
      await addStubToFrontmatter({
        type: stubType,
        description,
        anchor: anchor.id,
      });
      break;

    case 'delete':
      await removeAnchorFromContent(anchor);
      break;

    case 'convert':
      // Change ^stub-abc123 to ^abc123
      const newAnchorId = anchor.id.replace(/^\^stub-/, '^');
      await replaceAnchorInContent(anchor, newAnchorId);
      break;
  }
}
```

### 5.2 Resolution UI

```typescript
interface OrphanResolutionModal {
  // Show modal with orphans grouped by type
  show(orphans: {
    frontmatterOrphans: FrontmatterStub[];
    anchorOrphans: InlineAnchor[];
  }): void;

  // User actions
  onResolve(resolution: OrphanResolution): void;
  onResolveAll(strategy: BulkResolutionStrategy): void;
  onDismiss(): void;
}

interface OrphanResolution {
  itemId: string;
  strategy: 'delete' | 'reinsert' | 'create_stub' | 'ignore';
  stubType?: string;
  description?: string;
}

type BulkResolutionStrategy =
  | 'delete_all_orphaned_stubs'
  | 'delete_all_orphaned_anchors'
  | 'ignore_all';
```

---

## 6. Stub Creation Flow

### 6.1 Command: Insert Stub

```typescript
async function insertStubCommand(
  editor: Editor,
  config: StubsConfiguration
): Promise<void> {
  // 1. Show type dropdown
  const stubType = await showStubTypeDropdown(config.stubTypes);
  if (!stubType) return; // User cancelled

  // 2. Get description
  const description = await showDescriptionInput();
  if (!description) return;

  // 3. Generate or get custom anchor ID
  const anchorId = await getAnchorId(config.anchors);

  // 4. Insert anchor at cursor
  const cursor = editor.getCursor();
  const line = editor.getLine(cursor.line);

  // Insert at end of current line (like Obsidian)
  editor.replaceRange(` ${anchorId}`, { line: cursor.line, ch: line.length });

  // 5. Add stub to frontmatter
  await addStubToFrontmatter({
    type: stubType.key,
    description,
    anchor: anchorId,
    properties: stubType.defaults || {},
  });

  // 6. Trigger sync to update state
  await triggerSync();
}

async function getAnchorId(config: AnchorSettings): Promise<string> {
  // Show modal with options:
  // - Auto-generate (default, pre-selected)
  // - Custom ID input

  const result = await showAnchorIdModal({
    autoGeneratedId: generateAnchorId(config),
    allowCustom: true,
  });

  return result.useCustom ? result.customId : result.autoGeneratedId;
}
```

### 6.2 Anchor ID Generation

Matching Obsidian's block ID behavior:

```typescript
function generateAnchorId(config: AnchorSettings): string {
  const prefix = config.prefix; // "stub"

  switch (config.idStyle) {
    case 'random':
      // Like Obsidian: lowercase alphanumeric
      return `^${prefix}-${randomId(config.randomIdLength)}`;

    case 'type-prefixed':
      // Include stub type in ID
      return `^${prefix}-${stubType}-${randomId(4)}`;

    case 'sequential':
      // Auto-increment per document
      return `^${prefix}-${getNextSequentialId()}`;

    default:
      return `^${prefix}-${randomId(6)}`;
  }
}

function randomId(length: number): string {
  // Match Obsidian's style: lowercase + digits
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars[Math.floor(Math.random() * chars.length)];
  }
  return result;
}

// Ensure uniqueness in document
function ensureUniqueAnchorId(
  baseId: string,
  existingAnchors: Set<string>
): string {
  let id = baseId;
  let suffix = 1;

  while (existingAnchors.has(id)) {
    id = `${baseId}${suffix}`;
    suffix++;
  }

  return id;
}
```

---

## 7. Frontmatter Manipulation

### 7.1 Add Stub

```typescript
async function addStubToFrontmatter(
  file: TFile,
  stub: NewStub,
  config: StubsConfiguration
): Promise<void> {
  const content = await vault.read(file);
  const { frontmatter, contentStart, frontmatterEnd } = parseFrontmatter(content);

  // Get existing stubs array or create new
  const stubs = frontmatter[config.frontmatterKey] || [];

  // Create stub entry based on syntax preference
  const newEntry = createStubEntry(stub, config);
  stubs.push(newEntry);

  // Update frontmatter
  frontmatter[config.frontmatterKey] = stubs;

  // Serialize and write
  const newFrontmatter = serializeFrontmatter(frontmatter);
  const newContent = `---\n${newFrontmatter}---\n${content.slice(contentStart)}`;

  await vault.modify(file, newContent);
}

function createStubEntry(stub: NewStub, config: StubsConfiguration): object {
  // Compact syntax if only type, description, and anchor
  const hasExtraProperties = Object.keys(stub.properties || {}).length > 0;

  if (!hasExtraProperties) {
    // Compact: { link: "description", anchor: "^stub-xxx" }
    return {
      [stub.type]: stub.description,
      anchor: stub.anchor,
    };
  }

  // Structured: { link: { description: "...", anchor: "...", ...props } }
  return {
    [stub.type]: {
      description: stub.description,
      anchor: stub.anchor,
      ...stub.properties,
    },
  };
}
```

### 7.2 Remove Stub

```typescript
async function removeStubFromFrontmatter(
  file: TFile,
  anchorId: string,
  config: StubsConfiguration
): Promise<void> {
  const content = await vault.read(file);
  const { frontmatter, contentStart } = parseFrontmatter(content);

  const stubs = frontmatter[config.frontmatterKey] || [];

  // Find and remove stub with matching anchor
  const filteredStubs = stubs.filter(stub => {
    const stubAnchor = getStubAnchor(stub);
    return stubAnchor !== anchorId;
  });

  if (filteredStubs.length === stubs.length) {
    // Nothing removed
    return;
  }

  // Update frontmatter
  if (filteredStubs.length === 0) {
    delete frontmatter[config.frontmatterKey];
  } else {
    frontmatter[config.frontmatterKey] = filteredStubs;
  }

  // Serialize and write
  const newFrontmatter = serializeFrontmatter(frontmatter);
  const newContent = `---\n${newFrontmatter}---\n${content.slice(contentStart)}`;

  await vault.modify(file, newContent);
}
```

---

## 8. Event Handling

### 8.1 Sync Triggers

```typescript
class StubsSyncEngine {
  private plugin: Plugin;
  private debounceTimer: number | null = null;
  private readonly DEBOUNCE_MS = 500;

  constructor(plugin: Plugin) {
    this.plugin = plugin;
    this.registerEvents();
  }

  private registerEvents(): void {
    // File opened
    this.plugin.registerEvent(
      this.plugin.app.workspace.on('file-open', (file) => {
        if (file) this.performFullSync(file);
      })
    );

    // File modified (debounced)
    this.plugin.registerEvent(
      this.plugin.app.vault.on('modify', (file) => {
        this.debouncedSync(file);
      })
    );

    // Editor change (for real-time anchor detection)
    this.plugin.registerEvent(
      this.plugin.app.workspace.on('editor-change', (editor, info) => {
        this.handleEditorChange(editor, info.file);
      })
    );
  }

  private debouncedSync(file: TFile): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    this.debounceTimer = window.setTimeout(() => {
      this.performFullSync(file);
      this.debounceTimer = null;
    }, this.DEBOUNCE_MS);
  }

  private handleEditorChange(editor: Editor, file: TFile | null): void {
    // Quick check for anchor patterns in recent changes
    // More thorough sync happens on debounce
    const cursor = editor.getCursor();
    const line = editor.getLine(cursor.line);

    const pattern = getAnchorPattern(this.config.anchors.prefix);
    if (pattern.test(line)) {
      // Potential anchor change, schedule sync
      this.debouncedSync(file);
    }
  }
}
```

### 8.2 Notification System

```typescript
interface SyncNotification {
  type: 'info' | 'warning' | 'error';
  message: string;
  actions?: NotificationAction[];
  duration?: number;  // Auto-dismiss after ms (0 = sticky)
}

interface NotificationAction {
  label: string;
  callback: () => void;
}

function notifyOrphans(orphans: SyncState): void {
  const { orphanedStubs, orphanedAnchors } = orphans;

  if (orphanedStubs.length === 0 && orphanedAnchors.length === 0) {
    return;
  }

  const message = buildOrphanMessage(orphanedStubs.length, orphanedAnchors.length);

  showNotification({
    type: 'warning',
    message,
    actions: [
      {
        label: 'Resolve',
        callback: () => showOrphanResolutionModal(orphans),
      },
      {
        label: 'Dismiss',
        callback: () => {},
      },
    ],
    duration: 0,  // Sticky until dismissed
  });
}

function buildOrphanMessage(stubCount: number, anchorCount: number): string {
  const parts: string[] = [];

  if (stubCount > 0) {
    parts.push(`${stubCount} stub${stubCount > 1 ? 's' : ''} without anchors`);
  }

  if (anchorCount > 0) {
    parts.push(`${anchorCount} anchor${anchorCount > 1 ? 's' : ''} without stubs`);
  }

  return `Sync issue: ${parts.join(', ')}`;
}
```

---

## 9. Edge Cases

### 9.1 Multiple Anchors Same ID

Should not happen, but handle gracefully:

```typescript
function handleDuplicateAnchors(anchors: InlineAnchor[]): InlineAnchor[] {
  const seen = new Map<string, InlineAnchor>();
  const duplicates: InlineAnchor[] = [];

  for (const anchor of anchors) {
    if (seen.has(anchor.id)) {
      duplicates.push(anchor);
    } else {
      seen.set(anchor.id, anchor);
    }
  }

  if (duplicates.length > 0) {
    // Notify user
    showNotification({
      type: 'error',
      message: `Duplicate anchor IDs found: ${duplicates.map(d => d.id).join(', ')}`,
      actions: [
        {
          label: 'Auto-fix',
          callback: () => autoFixDuplicates(duplicates),
        },
      ],
    });
  }

  // Return only first occurrence of each ID
  return Array.from(seen.values());
}
```

### 9.2 Anchor in Code Block

Don't treat anchors in code blocks as stubs:

```typescript
function isInCodeBlock(line: number, content: string): boolean {
  const lines = content.split('\n');
  let inCodeBlock = false;

  for (let i = 0; i <= line; i++) {
    const currentLine = lines[i];
    if (currentLine.startsWith('```')) {
      inCodeBlock = !inCodeBlock;
    }
  }

  return inCodeBlock;
}

function filterAnchorsInCodeBlocks(
  anchors: InlineAnchor[],
  content: string
): InlineAnchor[] {
  return anchors.filter(anchor => !isInCodeBlock(anchor.position.line, content));
}
```

### 9.3 Frontmatter Parse Errors

Handle gracefully without breaking sync:

```typescript
function safeParseFrontmatter(
  content: string,
  config: StubsConfiguration
): { stubs: FrontmatterStub[]; errors: ParseError[] } {
  try {
    const { frontmatter } = parseFrontmatter(content);
    const result = parseStubsFrontmatter(frontmatter, config);
    return { stubs: result.stubs, errors: result.errors };
  } catch (error) {
    return {
      stubs: [],
      errors: [{
        type: 'yaml_error',
        message: `Failed to parse frontmatter: ${error.message}`,
      }],
    };
  }
}
```

---

## 10. Performance Considerations

### 10.1 Large Documents

For documents with many anchors:

```typescript
const LARGE_DOCUMENT_THRESHOLD = 50000; // characters
const LARGE_ANCHOR_COUNT = 100;

function shouldOptimizeSync(
  content: string,
  anchorCount: number
): boolean {
  return content.length > LARGE_DOCUMENT_THRESHOLD || anchorCount > LARGE_ANCHOR_COUNT;
}

function optimizedAnchorParse(content: string, config: AnchorSettings): InlineAnchor[] {
  // Use streaming/chunked parsing for large documents
  const CHUNK_SIZE = 10000;
  const anchors: InlineAnchor[] = [];

  for (let i = 0; i < content.length; i += CHUNK_SIZE) {
    const chunk = content.slice(i, i + CHUNK_SIZE);
    const chunkAnchors = parseInlineAnchors(chunk, config);

    // Adjust offsets
    for (const anchor of chunkAnchors) {
      anchor.position.offset += i;
    }

    anchors.push(...chunkAnchors);
  }

  return anchors;
}
```

### 10.2 Caching

```typescript
interface SyncCache {
  contentHash: string;
  lastSync: SyncState;
  timestamp: number;
}

const CACHE_TTL_MS = 5000;

function getCachedSync(file: TFile, content: string): SyncState | null {
  const cache = syncCacheMap.get(file.path);

  if (!cache) return null;

  const contentHash = hashContent(content);
  const isStale = Date.now() - cache.timestamp > CACHE_TTL_MS;

  if (cache.contentHash === contentHash && !isStale) {
    return cache.lastSync;
  }

  return null;
}
```

---

## 11. Testing

### 11.1 Sync Test Cases

```typescript
describe('Bidirectional Sync', () => {
  describe('Full Sync', () => {
    it('links matching stubs and anchors', async () => {
      const content = `---
stubs:
  - link: "Test stub"
    anchor: "^stub-test1"
---
Some content ^stub-test1`;

      const result = await performFullSync(mockFile(content), defaultConfig);

      expect(result.linked).toHaveLength(1);
      expect(result.orphanedStubs).toHaveLength(0);
      expect(result.orphanedAnchors).toHaveLength(0);
    });

    it('detects orphaned stub when anchor is missing', async () => {
      const content = `---
stubs:
  - link: "Test stub"
    anchor: "^stub-test1"
---
Some content without anchor`;

      const result = await performFullSync(mockFile(content), defaultConfig);

      expect(result.orphanedStubs).toHaveLength(1);
      expect(result.orphanedStubs[0].anchor).toBe('^stub-test1');
    });

    it('detects orphaned anchor when stub is missing', async () => {
      const content = `---
title: "Test"
---
Some content ^stub-orphan`;

      const result = await performFullSync(mockFile(content), defaultConfig);

      expect(result.orphanedAnchors).toHaveLength(1);
      expect(result.orphanedAnchors[0].id).toBe('^stub-orphan');
    });
  });

  describe('Anchor Detection', () => {
    it('ignores footnotes', () => {
      const content = 'Text with footnote[^1] and stub ^stub-test';
      const anchors = parseInlineAnchors(content, defaultConfig.anchors);

      expect(anchors).toHaveLength(1);
      expect(anchors[0].id).toBe('^stub-test');
    });

    it('ignores anchors in code blocks', () => {
      const content = `
Normal ^stub-valid
\`\`\`
Code ^stub-ignore
\`\`\`
`;
      const anchors = parseInlineAnchors(content, defaultConfig.anchors);
      const filtered = filterAnchorsInCodeBlocks(anchors, content);

      expect(filtered).toHaveLength(1);
      expect(filtered[0].id).toBe('^stub-valid');
    });
  });
});
```

---

## 12. See Also

- [PRD-stubs-support.md](./PRD-stubs-support.md) - Product requirements
- [SPEC-yaml-parsing.md](./SPEC-yaml-parsing.md) - YAML parsing specification
- [SPEC-configuration-schema.md](./SPEC-configuration-schema.md) - Configuration schema

---

**Document History**:
- 2025-12-01: Initial draft
