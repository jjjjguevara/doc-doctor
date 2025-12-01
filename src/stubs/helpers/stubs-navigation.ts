/**
 * Stubs Navigation Helper
 *
 * Handles navigation from sidebar to anchor locations in the editor.
 * Uses Obsidian's block reference format: [[filename#^anchor-id]]
 */

import { App, MarkdownView, EditorPosition, TFile, WorkspaceLeaf } from 'obsidian';
import { getAnchorById, syncState } from '../stubs-store';
import { get } from 'svelte/store';

/**
 * Get the markdown view for a file, even if sidebar is active
 */
function getMarkdownView(app: App): MarkdownView | null {
    // First try the active view
    const activeView = app.workspace.getActiveViewOfType(MarkdownView);
    if (activeView) {
        return activeView;
    }

    // If sidebar is active, find the markdown leaf another way
    // Get the active file first
    const activeFile = app.workspace.getActiveFile();
    if (!activeFile) {
        return null;
    }

    // Find a leaf that has this file open
    let markdownView: MarkdownView | null = null;
    app.workspace.iterateAllLeaves((leaf: WorkspaceLeaf) => {
        if (markdownView) return; // Already found
        const view = leaf.view;
        if (view instanceof MarkdownView && view.file?.path === activeFile.path) {
            markdownView = view;
        }
    });

    if (markdownView) {
        return markdownView;
    }

    // Last resort: find any markdown leaf and use it
    app.workspace.iterateAllLeaves((leaf: WorkspaceLeaf) => {
        if (markdownView) return;
        const view = leaf.view;
        if (view instanceof MarkdownView) {
            markdownView = view;
        }
    });

    return markdownView;
}

/**
 * Navigate to an anchor in the active document using Obsidian's link system
 * Anchor format: ^anchor-id (e.g., ^link-abc123)
 * Link format: [[filename#^anchor-id]]
 */
export function navigateToAnchor(app: App, anchorId: string): boolean {
    const view = getMarkdownView(app);
    if (!view || !view.file) {
        console.warn('No markdown view or file found');
        return false;
    }

    const file = view.file;
    const editor = view.editor;

    // First, reveal/activate the markdown leaf
    const leaf = view.leaf;
    if (leaf) {
        app.workspace.setActiveLeaf(leaf, { focus: true });
    }

    // Try direct search in the document (most reliable)
    // Skip frontmatter section to find inline anchors only
    if (editor) {
        const content = editor.getValue();
        const lines = content.split('\n');

        // Find where frontmatter ends
        let contentStartLine = 0;
        if (lines[0] === '---') {
            for (let i = 1; i < lines.length; i++) {
                if (lines[i] === '---') {
                    contentStartLine = i + 1;
                    break;
                }
            }
        }

        // Search only in content (after frontmatter)
        for (let lineNum = contentStartLine; lineNum < lines.length; lineNum++) {
            const line = lines[lineNum];
            const anchorIndex = line.indexOf(anchorId);
            if (anchorIndex !== -1) {
                const pos: EditorPosition = {
                    line: lineNum,
                    ch: anchorIndex,
                };

                editor.setCursor(pos);
                editor.scrollIntoView(
                    {
                        from: pos,
                        to: { line: pos.line, ch: pos.ch + anchorId.length },
                    },
                    true
                );
                editor.focus();
                return true;
            }
        }
    }

    // Fallback: Use Obsidian's link navigation
    // The anchorId includes the ^ prefix (e.g., "^link-abc123")
    // For Obsidian's openLinkText, we need the format: "filename#^anchor-id"
    const blockRef = anchorId.startsWith('^') ? anchorId : `^${anchorId}`;
    const linkText = `${file.basename}#${blockRef}`;

    app.workspace.openLinkText(linkText, file.path, false);

    return true;
}

/**
 * Navigate to a stub's anchor (if it exists)
 */
export function navigateToStub(app: App, stubId: string): boolean {
    const state = get(syncState);
    const stub = state.stubs.find((s) => s.id === stubId);

    if (!stub) {
        console.warn(`Stub not found: ${stubId}`);
        return false;
    }

    if (!stub.anchor) {
        console.warn(`Stub has no anchor: ${stubId}`);
        return false;
    }

    return navigateToAnchor(app, stub.anchor);
}

/**
 * Navigate to an orphaned stub's definition in frontmatter
 * Orphaned stubs exist in frontmatter but have no matching inline anchor
 */
export function navigateToOrphanedStub(app: App, anchorId: string): boolean {
    const view = getMarkdownView(app);
    if (!view || !view.file) {
        console.warn('No markdown view or file found');
        return false;
    }

    const editor = view.editor;
    if (!editor) {
        return false;
    }

    // First, reveal/activate the markdown leaf
    const leaf = view.leaf;
    if (leaf) {
        app.workspace.setActiveLeaf(leaf, { focus: true });
    }

    const content = editor.getValue();
    const lines = content.split('\n');

    // Find frontmatter boundaries
    if (lines[0] !== '---') {
        console.warn('No frontmatter found');
        return false;
    }

    let frontmatterEndLine = -1;
    for (let i = 1; i < lines.length; i++) {
        if (lines[i] === '---') {
            frontmatterEndLine = i;
            break;
        }
    }

    if (frontmatterEndLine === -1) {
        console.warn('Invalid frontmatter');
        return false;
    }

    // Search for the anchor in frontmatter only
    for (let lineNum = 1; lineNum < frontmatterEndLine; lineNum++) {
        const line = lines[lineNum];
        // Look for anchor: ^anchor-id pattern
        if (line.includes(anchorId) || line.includes(`anchor: ${anchorId}`) || line.includes(`anchor: "${anchorId}"`)) {
            const pos: EditorPosition = {
                line: lineNum,
                ch: 0,
            };

            editor.setCursor(pos);
            editor.scrollIntoView(
                {
                    from: pos,
                    to: { line: pos.line, ch: line.length },
                },
                true
            );
            editor.focus();
            return true;
        }
    }

    // Fallback: just go to the stubs section header
    for (let lineNum = 1; lineNum < frontmatterEndLine; lineNum++) {
        const line = lines[lineNum];
        if (line.match(/^stubs:/)) {
            const pos: EditorPosition = {
                line: lineNum,
                ch: 0,
            };

            editor.setCursor(pos);
            editor.scrollIntoView({ from: pos, to: pos }, true);
            editor.focus();
            return true;
        }
    }

    console.warn(`Anchor not found in frontmatter: ${anchorId}`);
    return false;
}

/**
 * Navigate to a stub's definition in frontmatter
 * Used for cycling between inline anchor and frontmatter definition
 */
export function navigateToStubFrontmatter(app: App, stubId: string): boolean {
    const state = get(syncState);
    const stub = state.stubs.find((s) => s.id === stubId);

    if (!stub) {
        console.warn(`Stub not found: ${stubId}`);
        return false;
    }

    // Use the anchor ID to find in frontmatter
    if (stub.anchor) {
        return navigateToOrphanedStub(app, stub.anchor);
    }

    // Fallback: try to find by description
    const view = getMarkdownView(app);
    if (!view || !view.file) {
        return false;
    }

    const editor = view.editor;
    if (!editor) {
        return false;
    }

    // Activate the leaf
    const leaf = view.leaf;
    if (leaf) {
        app.workspace.setActiveLeaf(leaf, { focus: true });
    }

    const content = editor.getValue();
    const lines = content.split('\n');

    // Find frontmatter boundaries
    if (lines[0] !== '---') {
        return false;
    }

    let frontmatterEndLine = -1;
    for (let i = 1; i < lines.length; i++) {
        if (lines[i] === '---') {
            frontmatterEndLine = i;
            break;
        }
    }

    if (frontmatterEndLine === -1) {
        return false;
    }

    // Search for the description in frontmatter
    for (let lineNum = 1; lineNum < frontmatterEndLine; lineNum++) {
        const line = lines[lineNum];
        if (line.includes(stub.description)) {
            const pos = { line: lineNum, ch: 0 };
            editor.setCursor(pos);
            editor.scrollIntoView({ from: pos, to: { line: pos.line, ch: line.length } }, true);
            editor.focus();
            return true;
        }
    }

    // Final fallback: go to stubs section header
    for (let lineNum = 1; lineNum < frontmatterEndLine; lineNum++) {
        const line = lines[lineNum];
        if (line.match(/^stubs:/)) {
            const pos = { line: lineNum, ch: 0 };
            editor.setCursor(pos);
            editor.scrollIntoView({ from: pos, to: pos }, true);
            editor.focus();
            return true;
        }
    }

    return false;
}

/**
 * Navigate to anchor by searching the document directly
 * Fallback method if Obsidian's link system doesn't work
 */
export function navigateToAnchorDirect(app: App, anchorId: string): boolean {
    const view = app.workspace.getActiveViewOfType(MarkdownView);
    if (!view) {
        return false;
    }

    const editor = view.editor;
    if (!editor) {
        return false;
    }

    const content = editor.getValue();
    const lines = content.split('\n');

    for (let lineNum = 0; lineNum < lines.length; lineNum++) {
        const line = lines[lineNum];
        const anchorIndex = line.indexOf(anchorId);
        if (anchorIndex !== -1) {
            const pos: EditorPosition = {
                line: lineNum,
                ch: anchorIndex,
            };

            editor.setCursor(pos);
            editor.scrollIntoView(
                {
                    from: pos,
                    to: { line: pos.line, ch: pos.ch + anchorId.length },
                },
                true
            );
            editor.focus();
            return true;
        }
    }

    console.warn(`Anchor not found in document: ${anchorId}`);
    return false;
}

/**
 * Navigate to line number in editor
 */
export function navigateToLine(app: App, lineNumber: number): boolean {
    const view = app.workspace.getActiveViewOfType(MarkdownView);
    if (!view) {
        return false;
    }

    const editor = view.editor;
    if (!editor) {
        return false;
    }

    const pos: EditorPosition = {
        line: lineNumber,
        ch: 0,
    };

    editor.setCursor(pos);
    editor.scrollIntoView({ from: pos, to: pos }, true);
    editor.focus();

    return true;
}

/**
 * Highlight anchor text in editor temporarily
 */
export function highlightAnchor(app: App, anchorId: string, durationMs: number = 2000): void {
    const view = app.workspace.getActiveViewOfType(MarkdownView);
    if (!view) {
        return;
    }

    const anchor = getAnchorById(anchorId);
    if (!anchor) {
        return;
    }

    const editor = view.editor;
    if (!editor) {
        return;
    }

    const from = { line: anchor.position.line, ch: anchor.position.ch };
    const to = { line: anchor.position.line, ch: anchor.position.ch + anchorId.length };

    // Select the anchor text
    editor.setSelection(from, to);

    // Clear selection after duration
    setTimeout(() => {
        const currentFrom = editor.getCursor('from');
        const currentTo = editor.getCursor('to');

        // Only clear if selection hasn't changed
        if (
            currentFrom.line === from.line &&
            currentFrom.ch === from.ch &&
            currentTo.line === to.line &&
            currentTo.ch === to.ch
        ) {
            editor.setCursor(to);
        }
    }, durationMs);
}

/**
 * Get the current cursor line
 */
export function getCurrentLine(app: App): number | null {
    const view = app.workspace.getActiveViewOfType(MarkdownView);
    if (!view) {
        return null;
    }

    const editor = view.editor;
    if (!editor) {
        return null;
    }

    return editor.getCursor().line;
}

/**
 * Find the nearest anchor to the cursor
 */
export function findNearestAnchor(app: App): string | null {
    const currentLine = getCurrentLine(app);
    if (currentLine === null) {
        return null;
    }

    const state = get(syncState);
    if (state.anchors.length === 0) {
        return null;
    }

    // Find anchor on current line or closest above
    let nearestAnchor = null;
    let nearestDistance = Infinity;

    for (const anchor of state.anchors) {
        const distance = currentLine - anchor.position.line;
        if (distance >= 0 && distance < nearestDistance) {
            nearestDistance = distance;
            nearestAnchor = anchor.id;
        }
    }

    return nearestAnchor;
}
