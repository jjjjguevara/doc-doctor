/**
 * Stubs Editor Plugin
 *
 * CodeMirror 6 plugin that decorates stub anchors in the editor.
 * Supports click-to-navigate for stub anchors in edit mode.
 */

import {
    Decoration,
    DecorationSet,
    EditorView,
    PluginSpec,
    PluginValue,
    ViewPlugin,
    ViewUpdate,
} from '@codemirror/view';
import { RangeSet } from '@codemirror/state';
import { decorateStubAnchors, clearDecorationCache } from './stubs-decorations';
import { syncState, stubsConfig, getStubByAnchorId } from '../stubs-store';
import { get } from 'svelte/store';
import type LabeledAnnotations from '../../main';

/**
 * Stubs Editor Plugin class
 */
export class StubsEditorPlugin implements PluginValue {
    static plugin: LabeledAnnotations;
    decorations: DecorationSet;

    constructor(view: EditorView) {
        this.decorations = this.buildDecorations(view);
    }

    update(update: ViewUpdate) {
        // Rebuild decorations if:
        // - Document changed
        // - Viewport changed
        if (update.docChanged || update.viewportChanged) {
            this.decorations = this.buildDecorations(update.view);
        }
    }

    destroy() {
        // Cleanup if needed
    }

    /**
     * Build decorations for stub anchors
     */
    private buildDecorations(view: EditorView): DecorationSet {
        const config = get(stubsConfig);
        const state = get(syncState);

        if (!config || !config.decorations.enabled) {
            return RangeSet.empty;
        }

        return decorateStubAnchors(view, config, state.stubs, state.anchors);
    }
}

/**
 * Handle click events on stub anchors
 * Ctrl/Cmd+Click navigates to the stub definition in frontmatter
 */
function handleStubAnchorClick(view: EditorView, pos: number, event: MouseEvent): boolean {
    // Only handle Ctrl+Click (Windows/Linux) or Cmd+Click (Mac)
    if (!event.ctrlKey && !event.metaKey) {
        return false;
    }

    const config = get(stubsConfig);
    const state = get(syncState);

    if (!config || !state.stubs.length) {
        return false;
    }

    // Get the text around the click position to find if we clicked on an anchor
    const line = view.state.doc.lineAt(pos);
    const lineText = line.text;

    // Find all stub anchors in this line
    for (const stub of state.stubs) {
        if (!stub.anchor) continue;

        const anchorIndex = lineText.indexOf(stub.anchor);
        if (anchorIndex !== -1) {
            const anchorStart = line.from + anchorIndex;
            const anchorEnd = anchorStart + stub.anchor.length;

            // Check if click was within this anchor
            if (pos >= anchorStart && pos <= anchorEnd) {
                // Show stub info or navigate to frontmatter
                // For now, we'll select the stub in the sidebar
                if (StubsEditorPlugin.plugin) {
                    // Import dynamically to avoid circular deps
                    import('../stubs-store').then(({ selectStub }) => {
                        selectStub(stub.id);
                    });
                }
                return true;
            }
        }
    }

    return false;
}

/**
 * Plugin specification with event handlers
 */
const pluginSpec: PluginSpec<StubsEditorPlugin> = {
    decorations: (value: StubsEditorPlugin) => value.decorations,
    eventHandlers: {
        click: (event: MouseEvent, view: EditorView) => {
            const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
            if (pos !== null) {
                return handleStubAnchorClick(view, pos, event);
            }
            return false;
        },
    },
};

/**
 * Exported editor plugin
 */
export const stubsEditorPlugin = ViewPlugin.fromClass(StubsEditorPlugin, pluginSpec);

/**
 * Trigger decoration update in editor (forces re-render)
 */
export function triggerStubsDecorationUpdate(view: EditorView): void {
    // Force a no-op transaction to trigger update
    view.dispatch({});
}

/**
 * Force clear decoration cache and rebuild
 */
export function refreshStubsDecorations(view: EditorView): void {
    clearDecorationCache();
    triggerStubsDecorationUpdate(view);
}

// Stub annotation export for backward compat
export const stubsAnnotation = {};
