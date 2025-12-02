/**
 * Bidirectional Sync Engine
 *
 * Keeps frontmatter stubs and inline ^stub-* anchors synchronized.
 * The frontmatter is the source of truth; inline anchors are references.
 */

import { TFile, App, Editor, MarkdownView, parseYaml as obsidianParseYaml } from 'obsidian';
import {
    ParsedStub,
    InlineAnchor,
    LinkedPair,
    SyncState,
    SyncError,
    StubsConfiguration,
} from '../stubs-types';
import { parseStubsFrontmatter } from './stubs-parser';
import { getValidAnchors, generateAnchorId, insertAnchorAtLine, removeAnchorFromContent } from './anchor-utils';

// =============================================================================
// FRONTMATTER UTILITIES
// =============================================================================

/**
 * Extract frontmatter from file content
 */
export function extractFrontmatter(content: string): {
    frontmatter: Record<string, unknown>;
    frontmatterRaw: string;
    contentStart: number;
    hasFrontmatter: boolean;
} {
    const match = content.match(/^---\n([\s\S]*?)\n---\n?/);

    if (!match) {
        return {
            frontmatter: {},
            frontmatterRaw: '',
            contentStart: 0,
            hasFrontmatter: false,
        };
    }

    const frontmatterRaw = match[1];
    const contentStart = match[0].length;

    // Parse YAML - we'll use a simple approach here
    // In production, use a proper YAML parser
    let frontmatter: Record<string, unknown> = {};
    try {
        // Use Obsidian's built-in YAML parser if available
        frontmatter = parseYaml(frontmatterRaw);
    } catch (e) {
        console.warn('Failed to parse frontmatter:', e);
    }

    return {
        frontmatter,
        frontmatterRaw,
        contentStart,
        hasFrontmatter: true,
    };
}

/**
 * Parse YAML using Obsidian's built-in parser
 */
function parseYaml(yamlString: string): Record<string, unknown> {
    try {
        const result = obsidianParseYaml(yamlString);
        return result && typeof result === 'object' ? result as Record<string, unknown> : {};
    } catch (e) {
        console.warn('Failed to parse YAML:', e);
        return {};
    }
}

// =============================================================================
// SYNC ENGINE
// =============================================================================

/**
 * Perform full synchronization between frontmatter and content
 */
export async function performSync(
    app: App,
    file: TFile,
    content: string,
    config: StubsConfiguration
): Promise<SyncState> {
    const errors: SyncError[] = [];

    // 1. Extract and parse frontmatter
    const { frontmatter, contentStart, hasFrontmatter } = extractFrontmatter(content);

    if (!hasFrontmatter) {
        return {
            stubs: [],
            anchors: [],
            linked: [],
            orphanedStubs: [],
            orphanedAnchors: [],
            lastSyncTime: Date.now(),
            errors: [],
        };
    }

    // 2. Parse stubs from frontmatter
    const parseResult = parseStubsFrontmatter(frontmatter, config);
    const stubs = parseResult.stubs;

    // Add parse errors to sync errors
    for (const error of parseResult.errors) {
        errors.push({
            type: 'parse_error',
            message: error.message,
            location: error.line ? { line: error.line, ch: 0 } : undefined,
        });
    }

    // 3. Parse anchors from content (after frontmatter)
    const documentContent = content.slice(contentStart);
    const anchors = getValidAnchors(documentContent, config.anchors);

    // Adjust anchor positions to account for frontmatter offset
    for (const anchor of anchors) {
        anchor.position.offset += contentStart;
    }

    // 4. Build linking
    const { linked, orphanedStubs, orphanedAnchors } = linkStubsAndAnchors(stubs, anchors, config);

    return {
        stubs,
        anchors,
        linked,
        orphanedStubs,
        orphanedAnchors,
        lastSyncTime: Date.now(),
        errors,
    };
}

/**
 * Link stubs with their corresponding anchors
 */
function linkStubsAndAnchors(
    stubs: ParsedStub[],
    anchors: InlineAnchor[],
    config: StubsConfiguration
): {
    linked: LinkedPair[];
    orphanedStubs: ParsedStub[];
    orphanedAnchors: InlineAnchor[];
} {
    const linked: LinkedPair[] = [];
    const orphanedStubs: ParsedStub[] = [];
    const orphanedAnchors: InlineAnchor[] = [];

    // Build anchor lookup
    const anchorMap = new Map<string, InlineAnchor>();
    for (const anchor of anchors) {
        anchorMap.set(anchor.id, anchor);
    }

    // Build set of stub anchors
    const stubAnchorIds = new Set<string>();
    for (const stub of stubs) {
        if (stub.anchor) {
            stubAnchorIds.add(stub.anchor);
        }
    }

    // Check each stub
    for (const stub of stubs) {
        if (stub.anchor) {
            const anchor = anchorMap.get(stub.anchor);
            if (anchor) {
                // Mark as linked
                stub.anchorResolved = true;
                anchor.hasStub = true;
                anchor.stubType = stub.type;
                anchor.stubDescription = stub.description;

                linked.push({ stub, anchor });
            } else {
                // Stub has anchor reference but anchor not found in content
                orphanedStubs.push(stub);
            }
        }
        // Note: Stubs without anchor property are valid (not orphaned)
    }

    // Check each anchor for orphans
    const prefix = `^${config.anchors.prefix}-`;
    for (const anchor of anchors) {
        if (anchor.id.startsWith(prefix) && !anchor.hasStub) {
            orphanedAnchors.push(anchor);
        }
    }

    return { linked, orphanedStubs, orphanedAnchors };
}

// =============================================================================
// FRONTMATTER MANIPULATION
// =============================================================================

/**
 * Add a new stub to frontmatter using Obsidian's processFrontMatter API
 * This preserves all other frontmatter properties
 */
export async function addStubToFrontmatter(
    app: App,
    file: TFile,
    stub: {
        type: string;
        description: string;
        anchor: string;
        properties?: Record<string, unknown>;
    },
    config: StubsConfiguration
): Promise<void> {
    await app.fileManager.processFrontMatter(file, (frontmatter) => {
        // Get existing stubs array or create new
        let stubs = (frontmatter[config.frontmatterKey] as unknown[]) || [];
        if (!Array.isArray(stubs)) {
            stubs = [];
        }

        // Create stub entry
        const hasProperties = stub.properties && Object.keys(stub.properties).length > 0;

        let newEntry: Record<string, unknown>;
        if (!hasProperties) {
            // Compact syntax
            newEntry = {
                [stub.type]: stub.description,
                anchor: stub.anchor,
            };
        } else {
            // Structured syntax
            newEntry = {
                [stub.type]: {
                    description: stub.description,
                    anchor: stub.anchor,
                    ...stub.properties,
                },
            };
        }

        stubs.push(newEntry);
        frontmatter[config.frontmatterKey] = stubs;
    });
}

/**
 * Remove a stub from frontmatter by anchor ID using direct string manipulation.
 * This preserves all other frontmatter formatting.
 */
export async function removeStubFromFrontmatter(
    app: App,
    file: TFile,
    anchorId: string,
    config: StubsConfiguration
): Promise<void> {
    const content = await app.vault.read(file);
    const newContent = removeStubEntryByAnchor(content, anchorId, config.frontmatterKey);

    if (newContent !== content) {
        await app.vault.modify(file, newContent);
    }
}

/**
 * Remove a stub from frontmatter by type and description using direct string manipulation.
 * This preserves all other frontmatter formatting.
 */
export async function removeStubFromFrontmatterByContent(
    app: App,
    file: TFile,
    stubType: string,
    description: string,
    config: StubsConfiguration
): Promise<void> {
    const content = await app.vault.read(file);
    const newContent = removeStubEntryByTypeAndDescription(content, stubType, description, config.frontmatterKey);

    if (newContent !== content) {
        await app.vault.modify(file, newContent);
    }
}

/**
 * Find stub entries in frontmatter and return their line ranges.
 * Handles both indented ("  - ") and non-indented ("- ") array styles.
 */
function findStubEntries(lines: string[], stubsKeyLine: number): Array<{
    startLine: number;
    endLine: number;
    content: string;
}> {
    const entries: Array<{ startLine: number; endLine: number; content: string }> = [];
    let i = stubsKeyLine + 1;

    // Detect indentation style from first entry
    let entryIndent = '';
    for (let k = stubsKeyLine + 1; k < lines.length; k++) {
        const line = lines[k];
        if (line === '---') break;
        const match = line.match(/^(\s*)- /);
        if (match) {
            entryIndent = match[1];
            break;
        }
        // Skip empty lines
        if (line.trim() === '') continue;
        // Hit another key, no entries
        if (line.match(/^[a-zA-Z_]/)) break;
    }

    const entryPattern = new RegExp(`^${entryIndent}- `);
    const continuationIndent = entryIndent + '  '; // continuation lines have more indent

    while (i < lines.length) {
        const line = lines[i];

        // Check if we've left the stubs section (new top-level key or end of frontmatter)
        if (line.match(/^[a-zA-Z_]/) || line === '---') {
            break;
        }

        // Check for array entry start
        if (entryPattern.test(line)) {
            const startLine = i;
            let endLine = i;

            // Find where this entry ends
            let j = i + 1;
            while (j < lines.length) {
                const nextLine = lines[j];

                // End of frontmatter
                if (nextLine === '---') break;

                // New top-level key (no leading whitespace, has colon)
                if (nextLine.match(/^[a-zA-Z_][\w]*:/)) break;

                // New array entry at same level
                if (entryPattern.test(nextLine)) break;

                // Continuation of this entry (more indented or empty)
                if (nextLine.startsWith(continuationIndent) || nextLine.trim() === '') {
                    endLine = j;
                    j++;
                } else {
                    break;
                }
            }

            // Collect the entry content
            const entryLines = lines.slice(startLine, endLine + 1);
            entries.push({
                startLine,
                endLine,
                content: entryLines.join('\n'),
            });

            i = endLine + 1;
        } else {
            i++;
        }
    }

    return entries;
}

/**
 * Remove a stub entry by anchor ID using direct string manipulation
 */
function removeStubEntryByAnchor(content: string, anchorId: string, stubsKey: string): string {
    const lines = content.split('\n');

    // Find frontmatter boundaries
    if (lines[0] !== '---') return content;

    const endIndex = lines.findIndex((line, idx) => idx > 0 && line === '---');
    if (endIndex === -1) return content;

    // Find stubs key line - match "stubs:" with optional trailing whitespace
    const stubsKeyLine = lines.findIndex((line, idx) => {
        if (idx <= 0 || idx >= endIndex) return false;
        const trimmed = line.trimEnd();
        return trimmed === `${stubsKey}:` || line.startsWith(`${stubsKey}: `);
    });
    if (stubsKeyLine === -1) return content;

    // Find all stub entries
    const entries = findStubEntries(lines, stubsKeyLine);

    // Find entry with matching anchor
    const entryToRemove = entries.find(entry => {
        return entry.content.includes(`anchor: ${anchorId}`) ||
               entry.content.includes(`anchor: "${anchorId}"`) ||
               entry.content.includes(`anchor: '${anchorId}'`);
    });

    if (!entryToRemove) return content;

    // Remove the entry lines
    const newLines = [
        ...lines.slice(0, entryToRemove.startLine),
        ...lines.slice(entryToRemove.endLine + 1),
    ];

    // Check if stubs array is now empty (only has the key line with no entries)
    const remainingEntries = findStubEntries(newLines, stubsKeyLine);
    if (remainingEntries.length === 0) {
        // Remove the stubs key line entirely
        newLines.splice(stubsKeyLine, 1);
    }

    return newLines.join('\n');
}

/**
 * Remove a stub entry by type and description using direct string manipulation
 */
function removeStubEntryByTypeAndDescription(
    content: string,
    stubType: string,
    description: string,
    stubsKey: string
): string {
    const lines = content.split('\n');

    // Find frontmatter boundaries
    if (lines[0] !== '---') return content;

    const endIndex = lines.findIndex((line, idx) => idx > 0 && line === '---');
    if (endIndex === -1) return content;

    // Find stubs key line - match "stubs:" with optional trailing whitespace
    const stubsKeyLine = lines.findIndex((line, idx) => {
        if (idx <= 0 || idx >= endIndex) return false;
        const trimmed = line.trimEnd();
        return trimmed === `${stubsKey}:` || line.startsWith(`${stubsKey}: `);
    });
    if (stubsKeyLine === -1) return content;

    // Find all stub entries
    const entries = findStubEntries(lines, stubsKeyLine);

    // Escape special regex characters in description
    const escapedDesc = description.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

    // Find entry with matching type and description
    const entryToRemove = entries.find(entry => {
        // Compact syntax: "  - type: description" or "  - type: "description""
        const compactPattern = new RegExp(`^\\s*-\\s*${stubType}:\\s*["']?${escapedDesc}["']?\\s*$`, 'm');
        if (compactPattern.test(entry.content)) return true;

        // Also check for type key followed by description
        const hasType = entry.content.includes(`- ${stubType}:`) || entry.content.includes(`- ${stubType} :`);
        const hasDesc = entry.content.includes(`description: ${description}`) ||
                       entry.content.includes(`description: "${description}"`) ||
                       entry.content.includes(`description: '${description}'`) ||
                       entry.content.includes(`: ${description}`) ||
                       entry.content.includes(`: "${description}"`) ||
                       entry.content.includes(`: '${description}'`);

        // For compact syntax, the description is the value directly after type:
        if (hasType) {
            // Check if type line contains the description directly
            const typeLinePattern = new RegExp(`-\\s*${stubType}:\\s*["']?${escapedDesc}["']?`);
            if (typeLinePattern.test(entry.content)) return true;
        }

        return hasType && hasDesc;
    });

    if (!entryToRemove) return content;

    // Remove the entry lines
    const newLines = [
        ...lines.slice(0, entryToRemove.startLine),
        ...lines.slice(entryToRemove.endLine + 1),
    ];

    // Check if stubs array is now empty
    const remainingEntries = findStubEntries(newLines, stubsKeyLine);
    if (remainingEntries.length === 0) {
        // Remove the stubs key line entirely
        newLines.splice(stubsKeyLine, 1);
    }

    return newLines.join('\n');
}

/**
 * Update stub anchor in frontmatter using Obsidian's processFrontMatter API
 */
export async function updateStubAnchor(
    app: App,
    file: TFile,
    oldAnchorId: string,
    newAnchorId: string,
    config: StubsConfiguration
): Promise<void> {
    await app.fileManager.processFrontMatter(file, (frontmatter) => {
        let stubs = (frontmatter[config.frontmatterKey] as unknown[]) || [];
        if (!Array.isArray(stubs)) return;

        // Update anchor in stubs
        for (const stub of stubs) {
            if (typeof stub !== 'object' || stub === null) continue;

            const stubObj = stub as Record<string, unknown>;

            // Check top-level anchor
            if (stubObj.anchor === oldAnchorId) {
                stubObj.anchor = newAnchorId;
            }

            // Check structured anchor
            for (const [key, value] of Object.entries(stubObj)) {
                if (typeof value === 'object' && value !== null) {
                    const valueObj = value as Record<string, unknown>;
                    if (valueObj.anchor === oldAnchorId) {
                        valueObj.anchor = newAnchorId;
                    }
                }
            }
        }
    });
}

/**
 * Simple frontmatter serializer
 * Note: In production, use a proper YAML serializer
 */
function serializeFrontmatter(frontmatter: Record<string, unknown>): string {
    // This is a placeholder - in the actual plugin, use:
    // import { stringifyYaml } from 'obsidian';
    // or import yaml from 'js-yaml';

    const lines: string[] = [];

    for (const [key, value] of Object.entries(frontmatter)) {
        if (value === undefined) continue;

        if (key === 'stubs' && Array.isArray(value)) {
            lines.push('stubs:');
            for (const stub of value) {
                lines.push(...serializeStubEntry(stub as Record<string, unknown>));
            }
        } else if (typeof value === 'string') {
            lines.push(`${key}: "${value}"`);
        } else if (typeof value === 'number' || typeof value === 'boolean') {
            lines.push(`${key}: ${value}`);
        } else if (Array.isArray(value)) {
            lines.push(`${key}:`);
            for (const item of value) {
                if (typeof item === 'string') {
                    lines.push(`  - "${item}"`);
                } else {
                    lines.push(`  - ${JSON.stringify(item)}`);
                }
            }
        } else if (typeof value === 'object' && value !== null) {
            lines.push(`${key}:`);
            for (const [subKey, subValue] of Object.entries(value)) {
                if (typeof subValue === 'string') {
                    lines.push(`  ${subKey}: "${subValue}"`);
                } else {
                    lines.push(`  ${subKey}: ${JSON.stringify(subValue)}`);
                }
            }
        }
    }

    return lines.join('\n') + '\n';
}

/**
 * Serialize a single stub entry
 */
function serializeStubEntry(stub: Record<string, unknown>): string[] {
    const lines: string[] = [];

    // Find the type key (not 'anchor')
    const typeKey = Object.keys(stub).find((k) => k !== 'anchor');
    if (!typeKey) return lines;

    const value = stub[typeKey];
    const anchor = stub.anchor;

    if (typeof value === 'string') {
        // Compact syntax
        lines.push(`  - ${typeKey}: "${value}"`);
        if (anchor) {
            lines.push(`    anchor: "${anchor}"`);
        }
    } else if (typeof value === 'object' && value !== null) {
        // Structured syntax
        lines.push(`  - ${typeKey}:`);
        const valueObj = value as Record<string, unknown>;

        for (const [propKey, propValue] of Object.entries(valueObj)) {
            if (typeof propValue === 'string') {
                lines.push(`      ${propKey}: "${propValue}"`);
            } else if (Array.isArray(propValue)) {
                lines.push(`      ${propKey}:`);
                for (const item of propValue) {
                    if (typeof item === 'string') {
                        lines.push(`        - "${item}"`);
                    } else {
                        lines.push(`        - ${JSON.stringify(item)}`);
                    }
                }
            } else {
                lines.push(`      ${propKey}: ${JSON.stringify(propValue)}`);
            }
        }
    }

    return lines;
}

// =============================================================================
// CONTENT MANIPULATION
// =============================================================================

/**
 * Insert stub at cursor position
 * Returns the generated anchor ID
 */
export async function insertStubAtCursor(
    app: App,
    editor: Editor,
    view: MarkdownView,
    stub: {
        type: string;
        description: string;
        properties?: Record<string, unknown>;
    },
    config: StubsConfiguration,
    customAnchorId?: string
): Promise<string | null> {
    const file = view.file;
    if (!file) return null;

    // Get existing anchors to ensure uniqueness
    const content = await app.vault.read(file);
    const { contentStart } = extractFrontmatter(content);
    const documentContent = content.slice(contentStart);
    const existingAnchors = new Set(
        getValidAnchors(documentContent, config.anchors).map((a) => a.id)
    );

    // Generate or use custom anchor ID
    const anchorId = customAnchorId || generateAnchorId(config.anchors, stub.type, existingAnchors);

    // Insert anchor at end of current line
    const cursor = editor.getCursor();
    const line = editor.getLine(cursor.line);
    const newLine = line.trimEnd() + ' ' + anchorId;
    editor.setLine(cursor.line, newLine);

    // Add stub to frontmatter
    await addStubToFrontmatter(app, file, {
        type: stub.type,
        description: stub.description,
        anchor: anchorId,
        properties: stub.properties,
    }, config);

    return anchorId;
}

/**
 * Remove anchor from content and optionally its stub
 */
export async function removeAnchor(
    app: App,
    file: TFile,
    anchorId: string,
    removeStub: boolean,
    config: StubsConfiguration
): Promise<void> {
    let content = await app.vault.read(file);

    // Remove from content
    content = removeAnchorFromContent(content, anchorId);

    // Remove from frontmatter if requested
    if (removeStub) {
        const { frontmatter, contentStart } = extractFrontmatter(content);
        // ... handle frontmatter update
    }

    await app.vault.modify(file, content);
}

// =============================================================================
// ORPHAN RESOLUTION
// =============================================================================

/**
 * Resolve orphaned stub (stub exists, anchor missing)
 */
export async function resolveOrphanedStub(
    app: App,
    file: TFile,
    stub: ParsedStub,
    strategy: 'delete' | 'reinsert',
    editor?: Editor,
    config?: StubsConfiguration
): Promise<void> {
    if (strategy === 'delete' && config) {
        await removeStubFromFrontmatter(app, file, stub.anchor!, config);
    } else if (strategy === 'reinsert' && editor && stub.anchor) {
        // Insert anchor at current cursor position
        const cursor = editor.getCursor();
        const line = editor.getLine(cursor.line);
        editor.setLine(cursor.line, line.trimEnd() + ' ' + stub.anchor);
    }
}

/**
 * Resolve orphaned anchor (anchor exists, stub missing)
 */
export async function resolveOrphanedAnchor(
    app: App,
    file: TFile,
    anchor: InlineAnchor,
    strategy: 'create_stub' | 'delete' | 'convert',
    stubInfo?: { type: string; description: string },
    config?: StubsConfiguration
): Promise<void> {
    if (strategy === 'create_stub' && stubInfo && config) {
        await addStubToFrontmatter(app, file, {
            type: stubInfo.type,
            description: stubInfo.description,
            anchor: anchor.id,
        }, config);
    } else if (strategy === 'delete') {
        let content = await app.vault.read(file);
        content = removeAnchorFromContent(content, anchor.id);
        await app.vault.modify(file, content);
    } else if (strategy === 'convert' && config) {
        // Convert ^stub-xxx to ^xxx (remove stub prefix)
        let content = await app.vault.read(file);
        const prefix = `^${config.anchors.prefix}-`;
        const newAnchorId = anchor.id.replace(prefix, '^');
        content = content.replace(anchor.id, newAnchorId);
        await app.vault.modify(file, content);
    }
}

// =============================================================================
// COMMAND HELPERS (simplified API for commands)
// =============================================================================

/**
 * Insert stub at cursor position (command-friendly API)
 * Uses processFrontMatter to preserve existing frontmatter
 * Returns the updated sync state
 */
export async function insertStubAtCursorCommand(
    app: App,
    file: TFile,
    content: string,
    config: StubsConfiguration,
    cursorLine: number,
    stubType: string,
    description: string,
    properties?: Record<string, unknown>
): Promise<{
    syncState: SyncState;
    newContent: string;
    anchorId: string;
} | null> {
    // Get existing anchors to ensure uniqueness
    const { contentStart } = extractFrontmatter(content);
    const documentContent = content.slice(contentStart);
    const existingAnchors = new Set(
        getValidAnchors(documentContent, config.anchors).map((a) => a.id)
    );

    // Generate anchor ID
    const anchorId = generateAnchorId(config.anchors, stubType, existingAnchors);

    // First, insert anchor at the end of the cursor line in the content
    const lines = content.split('\n');

    // cursorLine is 0-indexed from the editor, which includes frontmatter
    if (cursorLine < lines.length) {
        lines[cursorLine] = lines[cursorLine].trimEnd() + ' ' + anchorId;
    }

    // Write the content with anchor first
    let newContent = lines.join('\n');
    await app.vault.modify(file, newContent);

    // Now use processFrontMatter to safely add the stub entry
    await app.fileManager.processFrontMatter(file, (frontmatter) => {
        let stubs = (frontmatter[config.frontmatterKey] as unknown[]) || [];
        if (!Array.isArray(stubs)) {
            stubs = [];
        }

        const hasProperties = properties && Object.keys(properties).length > 0;
        let newEntry: Record<string, unknown>;

        if (!hasProperties) {
            newEntry = {
                [stubType]: description,
                anchor: anchorId,
            };
        } else {
            newEntry = {
                [stubType]: {
                    description,
                    anchor: anchorId,
                    ...properties,
                },
            };
        }

        stubs.push(newEntry);
        frontmatter[config.frontmatterKey] = stubs;
    });

    // Read the final content after frontmatter update
    newContent = await app.vault.read(file);

    // Sync and return
    const syncState = await performSync(app, file, newContent, config);

    return {
        syncState,
        newContent,
        anchorId,
    };
}

// Note: insertStubAtCursor is now replaced by insertStubAtCursorCommand for command usage

/**
 * Resolve orphaned stub by creating anchor at specified line
 */
export async function resolveOrphanedStubCommand(
    app: App,
    file: TFile,
    content: string,
    config: StubsConfiguration,
    stubId: string,
    targetLine: number
): Promise<{
    syncState: SyncState;
    newContent: string;
} | null> {
    // Find the stub in current sync
    const currentSync = await performSync(app, file, content, config);
    const stub = currentSync.orphanedStubs.find((s) => s.id === stubId);

    if (!stub || !stub.anchor) {
        return null;
    }

    // Insert anchor at target line
    const { contentStart } = extractFrontmatter(content);
    const lines = content.split('\n');
    const frontmatterLines = content.slice(0, contentStart).split('\n').length;
    const contentLineIndex = targetLine + frontmatterLines;

    if (contentLineIndex < lines.length) {
        lines[contentLineIndex] = lines[contentLineIndex].trimEnd() + ' ' + stub.anchor;
    }

    const newContent = lines.join('\n');

    // Re-sync
    const syncState = await performSync(app, file, newContent, config);

    return {
        syncState,
        newContent,
    };
}
