/**
 * Stub Suggest - Inline Stub Insertion
 *
 * Provides autocomplete suggestions for creating stubs inline.
 * Triggered by ^^ for simple stubs or ^^^ for structured stubs.
 *
 * Flow:
 * 1. User types ^^ or ^^^
 * 2. Dropdown shows stub types
 * 3. User selects type
 * 4. Plugin inserts ^{type}- with cursor positioned after hyphen
 * 5. User types custom ID or presses Enter/moves cursor
 * 6. Plugin finalizes: generates random ID if empty, adds to frontmatter
 */

import {
    App,
    Editor,
    EditorPosition,
    EditorSuggest,
    EditorSuggestContext,
    EditorSuggestTriggerInfo,
    MarkdownView,
} from 'obsidian';
import type LabeledAnnotations from '../../main';
import { getSortedStubTypes } from '../stubs-defaults';
import { StubTypeDefinition } from '../stubs-types';
import { addStubToFrontmatter, performSync } from '../helpers/stubs-sync';
import { updateSyncState } from '../stubs-store';

export type StubCompletion = {
    typeDef: StubTypeDefinition;
    isStructured: boolean;
};

// Generate random ID suffix
function generateRandomId(length: number = 6): string {
    const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
}

/**
 * Inline tooltip for entering stub description (for structured stubs)
 * Similar to Obsidian's footnote tooltip behavior
 */
class StubDescriptionTooltip {
    private container: HTMLElement;
    private input: HTMLInputElement;
    private onSubmit: (description: string) => void;
    private onCancel: () => void;
    private stubType: string;
    private typeDef: StubTypeDefinition;

    constructor(
        app: App,
        editor: Editor,
        position: EditorPosition,
        stubType: string,
        typeDef: StubTypeDefinition,
        onSubmit: (description: string) => void,
        onCancel: () => void
    ) {
        this.stubType = stubType;
        this.typeDef = typeDef;
        this.onSubmit = onSubmit;
        this.onCancel = onCancel;

        // Create tooltip container
        this.container = document.createElement('div');
        this.container.addClass('stub-description-tooltip');

        // Add color indicator
        const indicator = this.container.createSpan('stub-tooltip-indicator');
        indicator.style.backgroundColor = typeDef.color;

        // Add label
        const label = this.container.createSpan('stub-tooltip-label');
        label.setText(typeDef.displayName);

        // Create input field
        this.input = document.createElement('input');
        this.input.type = 'text';
        this.input.placeholder = 'Description...';
        this.input.addClass('stub-tooltip-input');
        this.container.appendChild(this.input);

        // Handle keyboard events
        this.input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                this.submit();
            } else if (e.key === 'Escape') {
                e.preventDefault();
                this.cancel();
            }
        });

        // Handle blur (clicking outside)
        this.input.addEventListener('blur', () => {
            // Small delay to allow click events to fire first
            setTimeout(() => {
                if (this.container.isConnected) {
                    this.submit();
                }
            }, 100);
        });

        // Position the tooltip near the cursor
        this.positionTooltip(editor, position);

        // Add to document and focus
        document.body.appendChild(this.container);
        setTimeout(() => this.input.focus(), 10);
    }

    private positionTooltip(editor: Editor, position: EditorPosition) {
        // Get cursor coordinates from the editor
        // We need to find the editor's DOM element and calculate position
        const editorEl = (editor as any).cm?.dom || document.querySelector('.cm-editor');
        if (!editorEl) {
            // Fallback: position in center of viewport
            this.container.style.position = 'fixed';
            this.container.style.top = '30%';
            this.container.style.left = '50%';
            this.container.style.transform = 'translateX(-50%)';
            return;
        }

        // Try to get cursor position using CodeMirror
        const cm = (editor as any).cm;
        if (cm && cm.coordsAtPos) {
            const coords = cm.coordsAtPos(cm.state.selection.main.head);
            if (coords) {
                this.container.style.position = 'fixed';
                this.container.style.left = `${coords.left}px`;
                this.container.style.top = `${coords.bottom + 5}px`;
                return;
            }
        }

        // Fallback positioning
        const rect = editorEl.getBoundingClientRect();
        this.container.style.position = 'fixed';
        this.container.style.left = `${rect.left + 50}px`;
        this.container.style.top = `${rect.top + 100}px`;
    }

    private submit() {
        const description = this.input.value.trim() || `${this.typeDef.displayName} stub`;
        this.destroy();
        this.onSubmit(description);
    }

    private cancel() {
        this.destroy();
        this.onCancel();
    }

    private destroy() {
        if (this.container.isConnected) {
            this.container.remove();
        }
    }
}

/**
 * Editor suggest for stub insertion
 */
export class StubSuggest extends EditorSuggest<StubCompletion> {
    readonly app: App;
    private plugin: LabeledAnnotations;
    private isStructured: boolean = false;

    // Pending stub info for finalization
    private pendingStub: {
        type: string;
        line: number;
        startCh: number;
        anchorPrefix: string;
        isStructured: boolean;
    } | null = null;

    constructor(app: App, plugin: LabeledAnnotations) {
        super(app);
        this.app = app;
        this.plugin = plugin;

        // Register cursor activity handler to finalize pending stubs
        this.registerCursorHandler();
    }

    /**
     * Register handler to finalize stub when cursor leaves the anchor area
     */
    private registerCursorHandler() {
        // Listen for editor changes to detect when user moves away from pending stub
        this.plugin.registerEvent(
            this.app.workspace.on('editor-change', (editor: Editor) => {
                if (this.pendingStub) {
                    this.checkAndFinalizePendingStub(editor);
                }
            })
        );

        // Also listen for cursor changes
        this.plugin.registerEvent(
            this.app.workspace.on('active-leaf-change', () => {
                if (this.pendingStub) {
                    this.finalizePendingStub();
                }
            })
        );
    }

    /**
     * Check if cursor has moved away from the pending stub anchor
     */
    private checkAndFinalizePendingStub(editor: Editor) {
        if (!this.pendingStub) return;

        const cursor = editor.getCursor();
        const line = editor.getLine(this.pendingStub.line);

        // Check if cursor is still on the same line and within the anchor
        if (cursor.line !== this.pendingStub.line) {
            this.finalizePendingStub(editor);
            return;
        }

        // Find the anchor in the line
        const anchorMatch = line.match(new RegExp(`\\^${this.pendingStub.type}-([a-zA-Z0-9-]*)`, 'g'));
        if (!anchorMatch) {
            this.finalizePendingStub(editor);
            return;
        }
    }

    /**
     * Finalize pending stub by adding to frontmatter
     */
    private async finalizePendingStub(editor?: Editor) {
        if (!this.pendingStub) return;

        const view = this.app.workspace.getActiveViewOfType(MarkdownView);
        if (!view || !view.file) {
            this.pendingStub = null;
            return;
        }

        const config = this.plugin.settings.getValue().stubs;
        if (!config) {
            this.pendingStub = null;
            return;
        }

        // Get the current line content
        const activeEditor = editor || view.editor;
        const line = activeEditor.getLine(this.pendingStub.line);

        // Find the full anchor ID that was entered
        const anchorPattern = new RegExp(`\\^${this.pendingStub.type}-([a-zA-Z0-9-]*)`);
        const match = line.match(anchorPattern);

        if (!match) {
            this.pendingStub = null;
            return;
        }

        let anchorId = match[0];
        const userSuffix = match[1];

        // If user didn't enter a suffix, generate a random one
        if (!userSuffix || userSuffix.length === 0) {
            const randomSuffix = generateRandomId(config.anchors.randomIdLength);
            anchorId = `^${this.pendingStub.type}-${randomSuffix}`;

            // Update the anchor in the editor
            const anchorStart = line.indexOf(match[0]);
            if (anchorStart >= 0) {
                activeEditor.replaceRange(
                    anchorId,
                    { line: this.pendingStub.line, ch: anchorStart },
                    { line: this.pendingStub.line, ch: anchorStart + match[0].length }
                );
            }
        }

        // Get type definition for descriptions and defaults
        const typeDef = getSortedStubTypes(config).find((t) => t.key === this.pendingStub!.type);

        // Default description for compact stubs - use type's defaultStubDescription if available
        const defaultDescription = typeDef?.defaultStubDescription || typeDef?.displayName || this.pendingStub.type;

        // Store pending info before clearing
        const stubType = this.pendingStub.type;
        const isStructured = this.pendingStub.isStructured;

        // Build properties to include based on per-property toggle
        // Only include properties that have includeInStructured: true (explicit)
        const structuredProperties: Record<string, unknown> = {};
        if (isStructured) {
            for (const propDef of Object.values(config.structuredProperties)) {
                // Only include if explicitly set to true (not undefined, not false)
                if (propDef.includeInStructured === true) {
                    // Use the property's default value, or the type's default for this property
                    const typeDefault = typeDef?.defaults?.[propDef.key];
                    let defaultValue = propDef.defaultValue ?? typeDefault;

                    // For types without a default, provide a sensible empty value
                    if (defaultValue === undefined) {
                        switch (propDef.type) {
                            case 'array':
                                defaultValue = [];
                                break;
                            case 'string':
                                defaultValue = '';
                                break;
                            case 'boolean':
                                defaultValue = false;
                                break;
                            case 'number':
                                defaultValue = 0;
                                break;
                            // For enum, skip if no default (user must set one)
                        }
                    }

                    if (defaultValue !== undefined) {
                        structuredProperties[propDef.key] = defaultValue;
                    }
                }
            }
        }

        // Clear pending first to avoid re-entry
        this.pendingStub = null;

        // For structured stubs, show inline description tooltip
        if (isStructured) {
            if (typeDef) {
                new StubDescriptionTooltip(
                    this.app,
                    activeEditor,
                    activeEditor.getCursor(),
                    stubType,
                    typeDef,
                    async (desc) => {
                        await this.addStubToFrontmatter(view, stubType, anchorId, desc, structuredProperties);
                    },
                    () => {
                        // On cancel, still add with default description
                        this.addStubToFrontmatter(view, stubType, anchorId, defaultDescription, structuredProperties);
                    }
                );
            } else {
                await this.addStubToFrontmatter(view, stubType, anchorId, defaultDescription, structuredProperties);
            }
        } else {
            // For simple stubs, add directly with default description (no properties)
            await this.addStubToFrontmatter(view, stubType, anchorId, defaultDescription);
        }
    }

    /**
     * Add stub to frontmatter and sync
     * @param properties - Optional structured properties (for ^^^ structured stubs)
     */
    private async addStubToFrontmatter(
        view: MarkdownView,
        stubType: string,
        anchorId: string,
        description: string,
        properties?: Record<string, unknown>
    ) {
        const file = view.file;
        if (!file) return;

        const config = this.plugin.settings.getValue().stubs;
        if (!config) return;

        try {
            // Request save to ensure editor changes are persisted before modifying frontmatter
            // This prevents the "modified externally" warning
            await view.save();

            // Small delay to ensure file system has synced
            await new Promise(resolve => setTimeout(resolve, 50));

            // Add to frontmatter using processFrontMatter
            await addStubToFrontmatter(this.app, file, {
                type: stubType,
                description,
                anchor: anchorId,
                properties,
            }, config);

            // Sync state
            const content = await this.app.vault.read(file);
            const syncState = await performSync(this.app, file, content, config);
            updateSyncState(syncState);
        } catch (error) {
            console.error('Failed to add stub to frontmatter:', error);
        }
    }

    getSuggestions(context: EditorSuggestContext): StubCompletion[] {
        const config = this.plugin.settings.getValue().stubs;
        if (!config || !config.enabled) {
            return [];
        }

        const sortedTypes = getSortedStubTypes(config);

        return sortedTypes
            .filter((typeDef) =>
                typeDef.displayName.toLowerCase().includes(context.query.toLowerCase()) ||
                typeDef.key.toLowerCase().includes(context.query.toLowerCase())
            )
            .map((typeDef) => ({
                typeDef,
                isStructured: this.isStructured,
            }));
    }

    renderSuggestion(suggestion: StubCompletion, el: HTMLElement): void {
        el.addClass('stub-suggestion');

        // Color indicator
        const indicator = el.createSpan('stub-suggestion-indicator');
        indicator.style.backgroundColor = suggestion.typeDef.color;

        // Type name
        const nameEl = el.createSpan('stub-suggestion-name');
        nameEl.setText(suggestion.typeDef.displayName);

        // Key hint
        const keyEl = el.createSpan('stub-suggestion-key');
        keyEl.setText(suggestion.typeDef.key);

        // Structured indicator
        if (suggestion.isStructured) {
            const structuredEl = el.createSpan('stub-suggestion-structured');
            structuredEl.setText('+ description');
        }

        el.appendChild(indicator);
        el.appendChild(nameEl);
        el.appendChild(keyEl);
    }

    async selectSuggestion(suggestion: StubCompletion): Promise<void> {
        if (!this.context) return;

        const editor = this.context.editor;
        const config = this.plugin.settings.getValue().stubs;
        if (!config) return;

        const stubType = suggestion.typeDef.key;

        // Insert anchor prefix: ^{type}-
        // User will type the ID after the hyphen
        const anchorPrefix = `^${stubType}-`;

        // Replace the trigger with the anchor prefix
        editor.replaceRange(anchorPrefix, this.context.start, this.context.end);

        // Position cursor right after the hyphen for user to type ID
        const newCursor: EditorPosition = {
            line: this.context.start.line,
            ch: this.context.start.ch + anchorPrefix.length,
        };
        editor.setCursor(newCursor);

        // Store pending stub info
        this.pendingStub = {
            type: stubType,
            line: this.context.start.line,
            startCh: this.context.start.ch,
            anchorPrefix,
            isStructured: suggestion.isStructured,
        };

        // Set a timeout to finalize if user doesn't type anything
        // This handles the case where user just presses Enter immediately
        setTimeout(() => {
            if (this.pendingStub && this.pendingStub.type === stubType) {
                this.finalizePendingStub(editor);
            }
        }, 100);
    }

    onTrigger(
        cursor: EditorPosition,
        editor: Editor
    ): EditorSuggestTriggerInfo | null {
        const config = this.plugin.settings.getValue().stubs;
        if (!config || !config.enabled) {
            return null;
        }

        // Check for ^^^ (structured) or ^^ (simple)
        const line = editor.getLine(cursor.line);
        const textBeforeCursor = line.slice(0, cursor.ch);

        // Check for ^^^ first (structured stubs)
        if (textBeforeCursor.endsWith('^^^')) {
            this.isStructured = true;
            return {
                start: { line: cursor.line, ch: cursor.ch - 3 },
                end: cursor,
                query: '',
            };
        }

        // Check for ^^ (simple stubs)
        if (textBeforeCursor.endsWith('^^')) {
            // Make sure it's not part of ^^^
            if (!textBeforeCursor.endsWith('^^^')) {
                this.isStructured = false;
                return {
                    start: { line: cursor.line, ch: cursor.ch - 2 },
                    end: cursor,
                    query: '',
                };
            }
        }

        // Check if we're in an ongoing query after the trigger
        // e.g., "^^li" should still show filtered suggestions
        const triggerMatch = textBeforeCursor.match(/\^\^\^?([a-zA-Z]*)$/);
        if (triggerMatch) {
            const fullMatch = triggerMatch[0];
            const query = triggerMatch[1] || '';
            this.isStructured = fullMatch.startsWith('^^^');

            return {
                start: { line: cursor.line, ch: cursor.ch - fullMatch.length },
                end: cursor,
                query,
            };
        }

        return null;
    }
}

/**
 * CSS styles for stub suggestions and tooltip
 */
export const stubSuggestStyles = `
.stub-suggestion {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
}

.stub-suggestion-indicator {
    width: 8px;
    height: 8px;
    border-radius: 2px;
    flex-shrink: 0;
}

.stub-suggestion-name {
    flex: 1;
}

.stub-suggestion-key {
    font-family: var(--font-monospace);
    font-size: 0.85em;
    color: var(--text-muted);
    background: var(--background-modifier-border);
    padding: 1px 4px;
    border-radius: 3px;
}

.stub-suggestion-structured {
    font-size: 0.75em;
    color: var(--text-accent);
    font-style: italic;
}

/* Inline description tooltip styles */
.stub-description-tooltip {
    position: fixed;
    z-index: var(--layer-popover);
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: var(--background-primary);
    border: 1px solid var(--background-modifier-border);
    border-radius: 6px;
    box-shadow: var(--shadow-s);
    font-size: var(--font-ui-small);
    max-width: 400px;
}

.stub-tooltip-indicator {
    width: 10px;
    height: 10px;
    border-radius: 3px;
    flex-shrink: 0;
}

.stub-tooltip-label {
    font-weight: 500;
    color: var(--text-normal);
    white-space: nowrap;
}

.stub-tooltip-input {
    flex: 1;
    min-width: 150px;
    padding: 4px 8px;
    border: 1px solid var(--background-modifier-border);
    border-radius: 4px;
    background: var(--background-primary);
    color: var(--text-normal);
    font-size: var(--font-ui-small);
    outline: none;
}

.stub-tooltip-input:focus {
    border-color: var(--interactive-accent);
    box-shadow: 0 0 0 2px var(--background-modifier-border-focus);
}

.stub-tooltip-input::placeholder {
    color: var(--text-faint);
}
`;
