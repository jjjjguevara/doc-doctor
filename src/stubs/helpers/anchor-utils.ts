/**
 * Anchor Parser and Generator
 *
 * Handles detection of ^stub-* anchors in document content and
 * generation of new anchor IDs (matching Obsidian's block ID behavior).
 */

import { InlineAnchor, AnchorSettings } from '../stubs-types';

// =============================================================================
// ANCHOR DETECTION
// =============================================================================

/**
 * Get regex pattern for detecting stub anchors
 * Matches any ^{type}-{id} pattern where type is alphabetic and id is alphanumeric
 * This matches anchors like ^link-abc123, ^clarify-xyz, ^stub-001, etc.
 */
export function getAnchorPattern(prefix: string): RegExp {
    // Match ^[type]-[id] where type is alphabetic (stub type key) and id is alphanumeric
    // This allows matching ^link-abc, ^clarify-xyz, ^stub-001, etc.
    return new RegExp(`\\^[a-zA-Z]+-[a-zA-Z0-9_-]+`, 'g');
}

/**
 * Parse all stub anchors from document content
 */
export function parseInlineAnchors(
    content: string,
    anchorSettings: AnchorSettings
): InlineAnchor[] {
    const anchors: InlineAnchor[] = [];
    const pattern = getAnchorPattern(anchorSettings.prefix);
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

            // Check if at end of line (standard Obsidian position)
            const remainingText = line.slice(ch + anchorId.length).trim();
            const isEndOfLine = remainingText.length === 0;

            // Skip if inside code block (we'll filter these later)
            anchors.push({
                id: anchorId,
                position: {
                    line: lineNum,
                    ch,
                    offset: offset + ch,
                },
                lineContent: line,
                isEndOfLine,
                hasStub: false, // Will be updated during sync
            });
        }

        offset += line.length + 1; // +1 for newline
    }

    return anchors;
}

/**
 * Filter out anchors that are inside code blocks
 */
export function filterAnchorsInCodeBlocks(
    anchors: InlineAnchor[],
    content: string
): InlineAnchor[] {
    const lines = content.split('\n');
    const codeBlockLines = new Set<number>();

    let inCodeBlock = false;
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i].trim();
        if (line.startsWith('```')) {
            inCodeBlock = !inCodeBlock;
            codeBlockLines.add(i);
        } else if (inCodeBlock) {
            codeBlockLines.add(i);
        }
    }

    return anchors.filter((anchor) => !codeBlockLines.has(anchor.position.line));
}

/**
 * Filter out anchors that are inside inline code
 */
export function filterAnchorsInInlineCode(
    anchors: InlineAnchor[],
    content: string
): InlineAnchor[] {
    const lines = content.split('\n');

    return anchors.filter((anchor) => {
        const line = lines[anchor.position.line];
        const anchorStart = anchor.position.ch;
        const anchorEnd = anchorStart + anchor.id.length;

        // Check if anchor is inside backticks
        let inCode = false;
        for (let i = 0; i < line.length; i++) {
            if (line[i] === '`') {
                inCode = !inCode;
            }
            if (i >= anchorStart && i < anchorEnd && inCode) {
                return false;
            }
        }
        return true;
    });
}

/**
 * Get all valid stub anchors from content
 */
export function getValidAnchors(
    content: string,
    anchorSettings: AnchorSettings
): InlineAnchor[] {
    let anchors = parseInlineAnchors(content, anchorSettings);
    anchors = filterAnchorsInCodeBlocks(anchors, content);
    anchors = filterAnchorsInInlineCode(anchors, content);
    return anchors;
}

// =============================================================================
// ANCHOR GENERATION
// =============================================================================

/**
 * Generate a random alphanumeric ID (like Obsidian's block IDs)
 */
export function randomAlphanumeric(length: number): string {
    const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
}

/**
 * Generate a new anchor ID based on settings
 */
export function generateAnchorId(
    anchorSettings: AnchorSettings,
    stubType?: string,
    existingAnchors?: Set<string>
): string {
    const { prefix, idStyle, randomIdLength } = anchorSettings;
    let baseId: string;

    switch (idStyle) {
        case 'random':
            baseId = `^${prefix}-${randomAlphanumeric(randomIdLength)}`;
            break;

        case 'type-prefixed':
            if (stubType) {
                baseId = `^${prefix}-${stubType}-${randomAlphanumeric(4)}`;
            } else {
                baseId = `^${prefix}-${randomAlphanumeric(randomIdLength)}`;
            }
            break;

        case 'type-only':
            // Type-only: ^link-a1b2 (no prefix, just type)
            if (stubType) {
                baseId = `^${stubType}-${randomAlphanumeric(randomIdLength)}`;
            } else {
                // Fall back to prefix style if no type provided
                baseId = `^${prefix}-${randomAlphanumeric(randomIdLength)}`;
            }
            break;

        case 'sequential':
            // For sequential, we need existing anchors to find next number
            baseId = `^${prefix}-${getNextSequentialId(existingAnchors, prefix)}`;
            break;

        default:
            baseId = `^${prefix}-${randomAlphanumeric(6)}`;
    }

    // Ensure uniqueness
    return ensureUniqueAnchorId(baseId, existingAnchors);
}

/**
 * Get next sequential ID number
 */
function getNextSequentialId(existingAnchors: Set<string> | undefined, prefix: string): string {
    if (!existingAnchors || existingAnchors.size === 0) {
        return '001';
    }

    const pattern = new RegExp(`^\\^${prefix}-(\\d+)$`);
    let maxNum = 0;

    for (const anchor of existingAnchors) {
        const match = anchor.match(pattern);
        if (match) {
            const num = parseInt(match[1], 10);
            if (num > maxNum) {
                maxNum = num;
            }
        }
    }

    return String(maxNum + 1).padStart(3, '0');
}

/**
 * Ensure anchor ID is unique by appending suffix if needed
 */
function ensureUniqueAnchorId(
    baseId: string,
    existingAnchors: Set<string> | undefined
): string {
    if (!existingAnchors || !existingAnchors.has(baseId)) {
        return baseId;
    }

    let id = baseId;
    let suffix = 1;

    while (existingAnchors.has(id)) {
        id = `${baseId}${suffix}`;
        suffix++;
    }

    return id;
}

// =============================================================================
// ANCHOR VALIDATION
// =============================================================================

/**
 * Check if an anchor ID is valid for stub use
 */
export function isValidStubAnchor(
    anchorId: string,
    anchorSettings: AnchorSettings
): { valid: boolean; reason?: string } {
    const prefix = `^${anchorSettings.prefix}-`;

    // Must start with configured prefix
    if (!anchorId.startsWith(prefix)) {
        return { valid: false, reason: `Anchor must start with "${prefix}"` };
    }

    // Must have something after the prefix
    const idPart = anchorId.slice(prefix.length);
    if (idPart.length === 0) {
        return { valid: false, reason: 'Anchor ID cannot be empty' };
    }

    // ID part must be alphanumeric with hyphens/underscores
    if (!/^[a-zA-Z0-9_-]+$/.test(idPart)) {
        return { valid: false, reason: 'Anchor ID can only contain letters, numbers, hyphens, and underscores' };
    }

    return { valid: true };
}

/**
 * Check for duplicate anchors in content
 */
export function findDuplicateAnchors(anchors: InlineAnchor[]): Map<string, InlineAnchor[]> {
    const groups = new Map<string, InlineAnchor[]>();

    for (const anchor of anchors) {
        const existing = groups.get(anchor.id);
        if (existing) {
            existing.push(anchor);
        } else {
            groups.set(anchor.id, [anchor]);
        }
    }

    // Return only duplicates
    const duplicates = new Map<string, InlineAnchor[]>();
    for (const [id, anchorList] of groups) {
        if (anchorList.length > 1) {
            duplicates.set(id, anchorList);
        }
    }

    return duplicates;
}

// =============================================================================
// ANCHOR MANIPULATION
// =============================================================================

/**
 * Insert anchor at end of line in content
 */
export function insertAnchorAtLine(
    content: string,
    lineNumber: number,
    anchorId: string
): string {
    const lines = content.split('\n');

    if (lineNumber < 0 || lineNumber >= lines.length) {
        return content;
    }

    const line = lines[lineNumber];
    // Insert anchor at end of line with a space separator
    lines[lineNumber] = line.trimEnd() + ' ' + anchorId;

    return lines.join('\n');
}

/**
 * Remove anchor from content
 */
export function removeAnchorFromContent(
    content: string,
    anchorId: string
): string {
    // Remove the anchor and any preceding whitespace
    const pattern = new RegExp(`\\s*${escapeRegExp(anchorId)}`, 'g');
    return content.replace(pattern, '');
}

/**
 * Replace anchor in content
 */
export function replaceAnchorInContent(
    content: string,
    oldAnchorId: string,
    newAnchorId: string
): string {
    const pattern = new RegExp(escapeRegExp(oldAnchorId), 'g');
    return content.replace(pattern, newAnchorId);
}

/**
 * Escape special regex characters
 */
function escapeRegExp(string: string): string {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

// =============================================================================
// POSITION UTILITIES
// =============================================================================

/**
 * Find anchor by ID in content
 */
export function findAnchorById(
    anchors: InlineAnchor[],
    anchorId: string
): InlineAnchor | undefined {
    return anchors.find((a) => a.id === anchorId);
}

/**
 * Get line and column from absolute offset
 */
export function offsetToPosition(
    content: string,
    offset: number
): { line: number; ch: number } {
    const lines = content.split('\n');
    let currentOffset = 0;

    for (let line = 0; line < lines.length; line++) {
        const lineLength = lines[line].length + 1; // +1 for newline
        if (currentOffset + lineLength > offset) {
            return {
                line,
                ch: offset - currentOffset,
            };
        }
        currentOffset += lineLength;
    }

    // Past end of content
    return {
        line: lines.length - 1,
        ch: lines[lines.length - 1].length,
    };
}

/**
 * Get absolute offset from line and column
 */
export function positionToOffset(
    content: string,
    line: number,
    ch: number
): number {
    const lines = content.split('\n');
    let offset = 0;

    for (let i = 0; i < line && i < lines.length; i++) {
        offset += lines[i].length + 1; // +1 for newline
    }

    return offset + ch;
}
