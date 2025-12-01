/**
 * Stubs Editor Decorations
 *
 * CodeMirror 6 decorations for highlighting stub anchors in the editor.
 * Colors are derived from the stub type configuration.
 */

import { RangeSetBuilder } from '@codemirror/state';
import { Decoration, DecorationSet, EditorView } from '@codemirror/view';
import { StubsConfiguration, InlineAnchor, ParsedStub } from '../stubs-types';
import { getStubTypeByKey } from '../stubs-defaults';

// =============================================================================
// DECORATION CACHE
// =============================================================================

/**
 * Cache for created decorations to avoid recreation
 */
const decorationCache = new Map<string, Decoration>();

/**
 * Get or create a decoration for a stub type
 */
function getDecoration(
    config: StubsConfiguration,
    stubType: string | undefined,
    style: 'background' | 'underline' | 'badge' | 'gutter'
): Decoration {
    const cacheKey = `${stubType || 'default'}-${style}`;

    if (decorationCache.has(cacheKey)) {
        return decorationCache.get(cacheKey)!;
    }

    // Get color from stub type or use default
    let color = '#888888';
    if (stubType) {
        const typeDef = getStubTypeByKey(config, stubType);
        if (typeDef) {
            color = typeDef.color;
        }
    }

    let decoration: Decoration;

    switch (style) {
        case 'background':
            decoration = Decoration.mark({
                class: 'cm-stub-anchor cm-stub-anchor-bg',
                attributes: {
                    style: `background-color: ${hexToRgba(color, config.decorations.opacity)}`,
                },
            });
            break;

        case 'underline':
            decoration = Decoration.mark({
                class: 'cm-stub-anchor cm-stub-anchor-underline',
                attributes: {
                    style: `border-bottom: 2px solid ${color}`,
                },
            });
            break;

        case 'badge':
            decoration = Decoration.mark({
                class: 'cm-stub-anchor cm-stub-anchor-badge',
                attributes: {
                    style: `background-color: ${hexToRgba(color, 0.2)}; border: 1px solid ${color}; border-radius: 3px; padding: 0 2px;`,
                },
            });
            break;

        case 'gutter':
            // For gutter style, we still need a minimal mark
            decoration = Decoration.mark({
                class: 'cm-stub-anchor cm-stub-anchor-gutter',
            });
            break;

        default:
            decoration = Decoration.mark({
                class: 'cm-stub-anchor',
            });
    }

    decorationCache.set(cacheKey, decoration);
    return decoration;
}

/**
 * Create decoration for orphaned anchor
 */
function getOrphanedDecoration(): Decoration {
    const cacheKey = 'orphaned';

    if (decorationCache.has(cacheKey)) {
        return decorationCache.get(cacheKey)!;
    }

    const decoration = Decoration.mark({
        class: 'cm-stub-anchor cm-stub-anchor-orphaned',
        attributes: {
            style: 'background-color: rgba(255, 165, 0, 0.3); border-bottom: 2px dashed orange;',
        },
    });

    decorationCache.set(cacheKey, decoration);
    return decoration;
}

/**
 * Clear decoration cache (call when config changes)
 */
export function clearDecorationCache(): void {
    decorationCache.clear();
}

// =============================================================================
// MAIN DECORATION FUNCTION
// =============================================================================

/**
 * Decorate stub anchors in the visible editor range
 * Only highlights anchors that are referenced in frontmatter stubs
 */
export function decorateStubAnchors(
    view: EditorView,
    config: StubsConfiguration,
    stubs: ParsedStub[],
    anchors: InlineAnchor[]
): DecorationSet {
    const builder = new RangeSetBuilder<Decoration>();

    if (!config.decorations.enabled) {
        return builder.finish();
    }

    // Build a map of anchor ID -> stub for quick lookup
    // Only include anchors that are actually referenced in frontmatter
    const anchorToStub = new Map<string, ParsedStub>();
    for (const stub of stubs) {
        if (stub.anchor) {
            anchorToStub.set(stub.anchor, stub);
        }
    }

    // If no stubs have anchors, nothing to decorate
    if (anchorToStub.size === 0) {
        return builder.finish();
    }

    // Collect decorations to sort later (RangeSetBuilder requires sorted ranges)
    const decorations: Array<{ from: number; to: number; decoration: Decoration }> = [];

    // For each visible range, find anchors that match stubs
    for (const { from, to } of view.visibleRanges) {
        const text = view.state.sliceDoc(from, to);

        // Check each stub's anchor to see if it appears in this range
        for (const [anchorId, stub] of anchorToStub) {
            let searchPos = 0;
            while (true) {
                const idx = text.indexOf(anchorId, searchPos);
                if (idx === -1) break;

                const startPos = from + idx;
                const endPos = startPos + anchorId.length;

                // Anchor exists in frontmatter - use stub type color
                const decoration = getDecoration(config, stub.type, config.decorations.style);
                decorations.push({ from: startPos, to: endPos, decoration });

                searchPos = idx + 1;
            }
        }
    }

    // Sort by start position (required by RangeSetBuilder)
    decorations.sort((a, b) => a.from - b.from);

    // Add sorted decorations to builder
    for (const { from, to, decoration } of decorations) {
        builder.add(from, to, decoration);
    }

    return builder.finish();
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Convert hex color to rgba
 */
function hexToRgba(hex: string, alpha: number): string {
    // Remove # if present
    hex = hex.replace(/^#/, '');

    // Parse hex
    let r: number, g: number, b: number;

    if (hex.length === 3) {
        r = parseInt(hex[0] + hex[0], 16);
        g = parseInt(hex[1] + hex[1], 16);
        b = parseInt(hex[2] + hex[2], 16);
    } else if (hex.length === 6) {
        r = parseInt(hex.slice(0, 2), 16);
        g = parseInt(hex.slice(2, 4), 16);
        b = parseInt(hex.slice(4, 6), 16);
    } else {
        return `rgba(136, 136, 136, ${alpha})`;
    }

    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

// =============================================================================
// CSS STYLES
// =============================================================================

/**
 * CSS styles for stub anchor decorations
 */
export const stubAnchorStyles = `
.cm-stub-anchor {
    cursor: pointer;
    transition: all 0.15s ease;
    border-radius: 3px;
}

.cm-stub-anchor:hover {
    opacity: 0.85;
    filter: brightness(1.1);
}

/* Ctrl/Cmd hover state - show it's clickable */
.mod-macos .cm-stub-anchor:hover,
.mod-windows .cm-stub-anchor:hover,
.mod-linux .cm-stub-anchor:hover {
    text-decoration: underline;
    text-decoration-style: dotted;
}

.cm-stub-anchor-bg {
    border-radius: 3px;
    padding: 1px 3px;
}

.cm-stub-anchor-underline {
    text-decoration: none;
    border-bottom-width: 2px;
    border-bottom-style: solid;
}

.cm-stub-anchor-badge {
    font-family: var(--font-monospace);
    font-size: 0.9em;
    padding: 1px 4px;
}

.cm-stub-anchor-orphaned {
    cursor: help;
}

.cm-stub-anchor-gutter {
    /* Minimal styling for gutter-only mode */
}

/* Gutter marker styles */
.cm-stub-gutter-marker {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin: 4px;
}

/* Tooltip hint for Ctrl/Cmd+Click */
.cm-stub-anchor::after {
    content: '';
    position: absolute;
    opacity: 0;
    pointer-events: none;
}
`;
