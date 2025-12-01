/**
 * Stubs Commands
 *
 * Obsidian commands for managing stubs.
 */

import { App, Command, Editor, MarkdownView, MarkdownFileInfo, Notice, Modal, Setting } from 'obsidian';
import type LabeledAnnotations from '../../main';
import {
    performSync,
    insertStubAtCursorCommand,
    resolveOrphanedStubCommand as resolveOrphanedStubFn,
} from '../helpers/stubs-sync';
import {
    syncState,
    stubsConfig,
    updateSyncState,
    selectStub,
    getStubById,
} from '../stubs-store';
import { get } from 'svelte/store';
import { getSortedStubTypes } from '../stubs-defaults';
import { generateAnchorId } from '../helpers/anchor-utils';
import { navigateToStub, navigateToAnchor } from '../helpers/stubs-navigation';

/**
 * Register all stubs commands
 */
export function registerStubsCommands(plugin: LabeledAnnotations): void {
    // Sync stubs
    plugin.addCommand({
        id: 'm-stubs:sync-stubs',
        name: 'Sync stubs with document',
        callback: () => syncStubsCommand(plugin),
    });

    // Insert stub at cursor
    plugin.addCommand({
        id: 'm-stubs:insert-stub',
        name: 'Insert new stub at cursor',
        editorCallback: (editor, ctx) => {
            const view = ctx instanceof MarkdownView ? ctx : plugin.app.workspace.getActiveViewOfType(MarkdownView);
            if (view) insertStubCommand(plugin, editor, view);
        },
    });

    // Navigate to next stub
    plugin.addCommand({
        id: 'm-stubs:next-stub',
        name: 'Go to next stub',
        callback: () => navigateStubCommand(plugin, 'next'),
    });

    // Navigate to previous stub
    plugin.addCommand({
        id: 'm-stubs:prev-stub',
        name: 'Go to previous stub',
        callback: () => navigateStubCommand(plugin, 'prev'),
    });

    // Resolve orphaned stub (create anchor)
    plugin.addCommand({
        id: 'm-stubs:resolve-orphaned-stub',
        name: 'Resolve orphaned stub (create anchor)',
        editorCallback: (editor, ctx) => {
            const view = ctx instanceof MarkdownView ? ctx : plugin.app.workspace.getActiveViewOfType(MarkdownView);
            if (view) resolveOrphanedStubCommand(plugin, editor, view);
        },
    });

    // Remove orphaned anchor
    plugin.addCommand({
        id: 'm-stubs:remove-orphaned-anchor',
        name: 'Remove orphaned anchor',
        editorCallback: (editor, ctx) => {
            const view = ctx instanceof MarkdownView ? ctx : plugin.app.workspace.getActiveViewOfType(MarkdownView);
            if (view) removeOrphanedAnchorCommand(plugin, editor, view);
        },
    });

    // Quick add stub from selection
    plugin.addCommand({
        id: 'm-stubs:add-stub-from-selection',
        name: 'Create stub from selection',
        editorCallback: (editor, ctx) => {
            const view = ctx instanceof MarkdownView ? ctx : plugin.app.workspace.getActiveViewOfType(MarkdownView);
            if (view) addStubFromSelectionCommand(plugin, editor, view);
        },
    });
}

/**
 * Sync stubs command
 */
async function syncStubsCommand(plugin: LabeledAnnotations): Promise<void> {
    const view = plugin.app.workspace.getActiveViewOfType(MarkdownView);
    if (!view) {
        new Notice('No active markdown file');
        return;
    }

    const file = view.file;
    if (!file) {
        new Notice('No file open');
        return;
    }

    const config = get(stubsConfig);
    if (!config) {
        new Notice('Stubs not configured');
        return;
    }

    try {
        const content = await plugin.app.vault.read(file);
        const result = await performSync(plugin.app, file, content, config);
        updateSyncState(result);
        new Notice(`Synced ${result.stubs.length} stubs`);
    } catch (error) {
        console.error('Sync error:', error);
        new Notice('Failed to sync stubs');
    }
}

/**
 * Insert stub command - opens modal to select type and enter description
 */
function insertStubCommand(
    plugin: LabeledAnnotations,
    editor: Editor,
    view: MarkdownView
): void {
    const config = get(stubsConfig);
    if (!config) {
        new Notice('Stubs not configured');
        return;
    }

    // Open modal to get stub details
    new InsertStubModal(plugin.app, config, (type, description) => {
        insertStubAtCursorWithDetails(plugin, editor, view, type, description);
    }).open();
}

/**
 * Insert stub at cursor with provided details
 */
async function insertStubAtCursorWithDetails(
    plugin: LabeledAnnotations,
    editor: Editor,
    view: MarkdownView,
    stubType: string,
    description: string
): Promise<void> {
    const config = get(stubsConfig);
    if (!config) return;

    const file = view.file;
    if (!file) {
        new Notice('No file open');
        return;
    }

    try {
        const content = await plugin.app.vault.read(file);
        const cursorLine = editor.getCursor().line;

        const result = await insertStubAtCursorCommand(
            plugin.app,
            file,
            content,
            config,
            cursorLine,
            stubType,
            description
        );

        if (result) {
            // Update store
            updateSyncState(result.syncState);

            // Write updated content
            await plugin.app.vault.modify(file, result.newContent);

            // Notify user
            new Notice(`Added ${stubType} stub`);
        }
    } catch (error) {
        console.error('Insert stub error:', error);
        new Notice('Failed to insert stub');
    }
}

/**
 * Navigate to next/previous stub
 */
function navigateStubCommand(plugin: LabeledAnnotations, direction: 'next' | 'prev'): void {
    const state = get(syncState);
    if (state.stubs.length === 0) {
        new Notice('No stubs in document');
        return;
    }

    const view = plugin.app.workspace.getActiveViewOfType(MarkdownView);
    if (!view) {
        new Notice('No active markdown file');
        return;
    }

    const editor = view.editor;
    const currentLine = editor.getCursor().line;

    // Find stubs with resolved anchors and sort by line
    const stubsWithAnchors = state.stubs
        .filter((s) => s.anchorResolved && s.anchor)
        .map((s) => {
            const anchor = state.anchors.find((a) => a.id === s.anchor);
            return { stub: s, line: anchor?.position.line ?? -1 };
        })
        .filter((s) => s.line >= 0)
        .sort((a, b) => a.line - b.line);

    if (stubsWithAnchors.length === 0) {
        new Notice('No stubs with anchors');
        return;
    }

    let targetStub;

    if (direction === 'next') {
        // Find first stub after current line
        targetStub = stubsWithAnchors.find((s) => s.line > currentLine);
        // Wrap around to first if at end
        if (!targetStub) {
            targetStub = stubsWithAnchors[0];
        }
    } else {
        // Find last stub before current line
        const before = stubsWithAnchors.filter((s) => s.line < currentLine);
        targetStub = before[before.length - 1];
        // Wrap around to last if at beginning
        if (!targetStub) {
            targetStub = stubsWithAnchors[stubsWithAnchors.length - 1];
        }
    }

    if (targetStub) {
        selectStub(targetStub.stub.id);
        navigateToStub(plugin.app, targetStub.stub.id);
    }
}

/**
 * Resolve orphaned stub by creating anchor at cursor
 */
async function resolveOrphanedStubCommand(
    plugin: LabeledAnnotations,
    editor: Editor,
    view: MarkdownView
): Promise<void> {
    const state = get(syncState);
    const config = get(stubsConfig);

    if (!config) {
        new Notice('Stubs not configured');
        return;
    }

    if (state.orphanedStubs.length === 0) {
        new Notice('No orphaned stubs to resolve');
        return;
    }

    const file = view.file;
    if (!file) {
        new Notice('No file open');
        return;
    }

    // Open modal to select which stub to resolve
    new SelectStubModal(plugin.app, state.orphanedStubs, async (selectedStub) => {
        try {
            const content = await plugin.app.vault.read(file);
            const cursorLine = editor.getCursor().line;

            const result = await resolveOrphanedStubFn(
                plugin.app,
                file,
                content,
                config,
                selectedStub.id,
                cursorLine
            );

            if (result) {
                updateSyncState(result.syncState);
                await plugin.app.vault.modify(file, result.newContent);
                new Notice(`Linked stub to line ${cursorLine + 1}`);
            }
        } catch (error) {
            console.error('Resolve stub error:', error);
            new Notice('Failed to resolve stub');
        }
    }).open();
}

/**
 * Remove orphaned anchor from content
 */
async function removeOrphanedAnchorCommand(
    plugin: LabeledAnnotations,
    editor: Editor,
    view: MarkdownView
): Promise<void> {
    const state = get(syncState);

    if (state.orphanedAnchors.length === 0) {
        new Notice('No orphaned anchors to remove');
        return;
    }

    const file = view.file;
    if (!file) {
        new Notice('No file open');
        return;
    }

    // Open modal to select which anchor to remove
    new SelectAnchorModal(plugin.app, state.orphanedAnchors, async (selectedAnchor) => {
        try {
            let content = await plugin.app.vault.read(file);

            // Remove anchor from content
            const anchorPattern = new RegExp(`\\s*${escapeRegExp(selectedAnchor.id)}`, 'g');
            content = content.replace(anchorPattern, '');

            await plugin.app.vault.modify(file, content);

            // Re-sync
            const config = get(stubsConfig);
            if (config) {
                const result = await performSync(plugin.app, file, content, config);
                updateSyncState(result);
            }

            new Notice(`Removed anchor ${selectedAnchor.id}`);
        } catch (error) {
            console.error('Remove anchor error:', error);
            new Notice('Failed to remove anchor');
        }
    }).open();
}

/**
 * Create stub from selected text
 */
async function addStubFromSelectionCommand(
    plugin: LabeledAnnotations,
    editor: Editor,
    view: MarkdownView
): Promise<void> {
    const selection = editor.getSelection();
    if (!selection || selection.trim().length === 0) {
        new Notice('Please select some text first');
        return;
    }

    const config = get(stubsConfig);
    if (!config) {
        new Notice('Stubs not configured');
        return;
    }

    // Open modal to select type (description is from selection)
    new SelectTypeModal(plugin.app, config, async (stubType) => {
        await insertStubAtCursorWithDetails(plugin, editor, view, stubType, selection.trim());
    }).open();
}

// =============================================================================
// MODALS
// =============================================================================

/**
 * Modal for inserting a new stub
 */
class InsertStubModal extends Modal {
    private config: any;
    private onSubmit: (type: string, description: string) => void;
    private selectedType: string = '';
    private description: string = '';

    constructor(
        app: App,
        config: any,
        onSubmit: (type: string, description: string) => void
    ) {
        super(app);
        this.config = config;
        this.onSubmit = onSubmit;

        const types = getSortedStubTypes(config);
        if (types.length > 0) {
            this.selectedType = types[0].key;
        }
    }

    onOpen() {
        const { contentEl } = this;
        contentEl.createEl('h2', { text: 'Insert New Stub' });

        const types = getSortedStubTypes(this.config);
        const typeOptions: Record<string, string> = {};
        for (const type of types) {
            typeOptions[type.key] = type.displayName;
        }

        new Setting(contentEl)
            .setName('Type')
            .addDropdown((dropdown) => {
                dropdown.addOptions(typeOptions);
                dropdown.setValue(this.selectedType);
                dropdown.onChange((value) => {
                    this.selectedType = value;
                });
            });

        new Setting(contentEl)
            .setName('Description')
            .addText((text) => {
                text.setPlaceholder('What needs to be done?');
                text.onChange((value) => {
                    this.description = value;
                });
                // Focus on text input
                setTimeout(() => text.inputEl.focus(), 50);
            });

        new Setting(contentEl)
            .addButton((btn) => {
                btn.setButtonText('Insert')
                    .setCta()
                    .onClick(() => {
                        if (this.description.trim()) {
                            this.close();
                            this.onSubmit(this.selectedType, this.description.trim());
                        } else {
                            new Notice('Please enter a description');
                        }
                    });
            })
            .addButton((btn) => {
                btn.setButtonText('Cancel').onClick(() => this.close());
            });
    }

    onClose() {
        this.contentEl.empty();
    }
}

/**
 * Modal for selecting a stub type
 */
class SelectTypeModal extends Modal {
    private config: any;
    private onSubmit: (type: string) => void;

    constructor(app: App, config: any, onSubmit: (type: string) => void) {
        super(app);
        this.config = config;
        this.onSubmit = onSubmit;
    }

    onOpen() {
        const { contentEl } = this;
        contentEl.createEl('h2', { text: 'Select Stub Type' });

        const types = getSortedStubTypes(this.config);

        for (const type of types) {
            new Setting(contentEl)
                .setName(type.displayName)
                .setDesc(type.description || '')
                .addButton((btn) => {
                    btn.setButtonText('Select')
                        .setCta()
                        .onClick(() => {
                            this.close();
                            this.onSubmit(type.key);
                        });
                });
        }
    }

    onClose() {
        this.contentEl.empty();
    }
}

/**
 * Modal for selecting an orphaned stub
 */
class SelectStubModal extends Modal {
    private stubs: any[];
    private onSubmit: (stub: any) => void;

    constructor(app: App, stubs: any[], onSubmit: (stub: any) => void) {
        super(app);
        this.stubs = stubs;
        this.onSubmit = onSubmit;
    }

    onOpen() {
        const { contentEl } = this;
        contentEl.createEl('h2', { text: 'Select Stub to Link' });

        for (const stub of this.stubs) {
            new Setting(contentEl)
                .setName(stub.description)
                .setDesc(`Type: ${stub.type}`)
                .addButton((btn) => {
                    btn.setButtonText('Link Here')
                        .setCta()
                        .onClick(() => {
                            this.close();
                            this.onSubmit(stub);
                        });
                });
        }
    }

    onClose() {
        this.contentEl.empty();
    }
}

/**
 * Modal for selecting an orphaned anchor
 */
class SelectAnchorModal extends Modal {
    private anchors: any[];
    private onSubmit: (anchor: any) => void;

    constructor(app: App, anchors: any[], onSubmit: (anchor: any) => void) {
        super(app);
        this.anchors = anchors;
        this.onSubmit = onSubmit;
    }

    onOpen() {
        const { contentEl } = this;
        contentEl.createEl('h2', { text: 'Select Anchor to Remove' });

        for (const anchor of this.anchors) {
            new Setting(contentEl)
                .setName(anchor.id)
                .setDesc(`Line ${anchor.position.line + 1}`)
                .addButton((btn) => {
                    btn.setButtonText('Remove')
                        .setWarning()
                        .onClick(() => {
                            this.close();
                            this.onSubmit(anchor);
                        });
                });
        }
    }

    onClose() {
        this.contentEl.empty();
    }
}

// =============================================================================
// HELPERS
// =============================================================================

function escapeRegExp(string: string): string {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
