/**
 * Context Builder
 *
 * Builds execution context from the current editor state.
 */

import { App, MarkdownView, Editor, TFile } from 'obsidian';
import { ExecutionContext } from '../../llm/prompt-loader';

/**
 * Build execution context from the current editor state
 */
export function buildExecutionContext(app: App): ExecutionContext {
    const view = app.workspace.getActiveViewOfType(MarkdownView);
    const editor = view?.editor ?? null;
    const file = view?.file ?? null;

    if (!editor) {
        return {
            file: null,
            content: '',
            hasSelection: false,
            selection: '',
            selectionRange: null,
            cursorPosition: 0,
            currentLine: 0,
            currentLineContent: '',
        };
    }

    const content = editor.getValue();
    const selection = editor.getSelection();
    const hasSelection = selection.length > 0;

    // Get selection range
    let selectionRange: { start: number; end: number } | null = null;
    if (hasSelection) {
        const from = editor.getCursor('from');
        const to = editor.getCursor('to');
        selectionRange = {
            start: editor.posToOffset(from),
            end: editor.posToOffset(to),
        };
    }

    // Get cursor position
    const cursor = editor.getCursor();
    const cursorPosition = editor.posToOffset(cursor);
    const currentLine = cursor.line + 1; // 1-indexed
    const currentLineContent = editor.getLine(cursor.line);

    return {
        file,
        content,
        hasSelection,
        selection,
        selectionRange,
        cursorPosition,
        currentLine,
        currentLineContent,
    };
}

/**
 * Get the content to analyze based on context
 * Returns selection if available, otherwise full content
 */
export function getAnalysisContent(context: ExecutionContext): string {
    if (context.hasSelection && context.selection) {
        return context.selection;
    }
    return context.content;
}

/**
 * Check if context meets basic requirements for document operations
 */
export function hasValidDocument(context: ExecutionContext): boolean {
    return context.file !== null && context.file.extension === 'md';
}

/**
 * Get document frontmatter from content
 */
export function extractFrontmatter(content: string): Record<string, unknown> | null {
    const match = content.match(/^---\n([\s\S]*?)\n---/);
    if (!match) return null;

    // Simple frontmatter parser
    const frontmatter: Record<string, unknown> = {};
    const lines = match[1].split('\n');

    for (const line of lines) {
        const colonIdx = line.indexOf(':');
        if (colonIdx > 0) {
            const key = line.slice(0, colonIdx).trim();
            let value: string | number | boolean = line.slice(colonIdx + 1).trim();

            // Parse value type
            if (value === 'true') value = true;
            else if (value === 'false') value = false;
            else if (!isNaN(Number(value)) && value !== '') value = Number(value);
            // Remove quotes
            else if ((value.startsWith('"') && value.endsWith('"')) ||
                     (value.startsWith("'") && value.endsWith("'"))) {
                value = value.slice(1, -1);
            }

            frontmatter[key] = value;
        }
    }

    return frontmatter;
}

/**
 * Insert text at a specific line in the content
 */
export function insertAtLine(content: string, line: number, text: string): string {
    const lines = content.split('\n');
    // line is 1-indexed
    const idx = Math.max(0, Math.min(line - 1, lines.length));
    lines.splice(idx, 0, text);
    return lines.join('\n');
}

/**
 * Insert anchor at end of a specific line
 */
export function insertAnchorAtLine(content: string, line: number, anchorId: string): string {
    const lines = content.split('\n');
    const idx = Math.max(0, Math.min(line - 1, lines.length - 1));

    // Check if anchor already exists on this line
    if (lines[idx].includes(`^${anchorId}`)) {
        return content; // Already exists
    }

    // Append anchor to end of line
    lines[idx] = `${lines[idx]} ^${anchorId}`;
    return lines.join('\n');
}

/**
 * Generate a unique anchor ID
 */
export function generateAnchorId(prefix = 'stub'): string {
    const timestamp = Date.now().toString(36);
    const random = Math.random().toString(36).slice(2, 6);
    return `${prefix}-${timestamp}-${random}`;
}
