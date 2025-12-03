/**
 * AI Commands - Command Registration
 *
 * Registers all AI-related commands to the Obsidian command palette.
 */

import { Command, Editor, MarkdownView, Notice } from 'obsidian';
import type LabeledAnnotations from '../../main';
import { PromptLoader } from '../../llm/prompt-loader';
import { PromptDefinition } from '../../llm/prompt-schema';
import { MCPClient, MCPTools } from '../../mcp';
import { buildExecutionContext, hasValidDocument, getAnalysisContent } from './context-builder';

/**
 * AI Command IDs
 */
export const AI_COMMAND_PREFIX = 'doc-doctor:ai-';

/**
 * Core document operation command IDs
 */
export const CORE_COMMANDS = {
    PARSE_DOCUMENT: 'doc-doctor:parse-document',
    ANALYZE_DOCUMENT: 'doc-doctor:analyze-document',
    VALIDATE_DOCUMENT: 'doc-doctor:validate-document',
    CALCULATE_HEALTH: 'doc-doctor:calculate-health',
    CALCULATE_USEFULNESS: 'doc-doctor:calculate-usefulness',

    // Stub operations
    ADD_STUB: 'doc-doctor:add-stub',
    ADD_STUB_EXPAND: 'doc-doctor:add-stub-expand',
    ADD_STUB_LINK: 'doc-doctor:add-stub-link',
    ADD_STUB_VERIFY: 'doc-doctor:add-stub-verify',
    ADD_STUB_QUESTION: 'doc-doctor:add-stub-question',
    ADD_STUB_BLOCKER: 'doc-doctor:add-stub-blocker',
    RESOLVE_STUB: 'doc-doctor:resolve-stub',
    UPDATE_STUB: 'doc-doctor:update-stub',
    LIST_STUBS: 'doc-doctor:list-stubs',

    // Anchor operations
    ADD_ANCHOR: 'doc-doctor:add-anchor',
    LINK_ANCHOR: 'doc-doctor:link-anchor',
    UNLINK_ANCHOR: 'doc-doctor:unlink-anchor',
    FIND_ANCHORS: 'doc-doctor:find-anchors',

    // Navigation
    NEXT_STUB: 'doc-doctor:next-stub',
    PREV_STUB: 'doc-doctor:prev-stub',
    GOTO_STUB: 'doc-doctor:goto-stub',

    // Vault operations
    SCAN_VAULT: 'doc-doctor:scan-vault',
    FIND_BLOCKING_STUBS: 'doc-doctor:find-blocking-stubs',
    BATCH_ANALYZE: 'doc-doctor:batch-analyze',

    // AI operations
    AI_CUSTOM: 'doc-doctor:ai-custom',
};

/**
 * Register all AI commands
 */
export function registerAICommands(
    plugin: LabeledAnnotations,
    promptLoader: PromptLoader,
    mcpClient: MCPClient | null
): void {
    // Register prompt-based AI commands
    registerPromptCommands(plugin, promptLoader, mcpClient);

    // Register core document operations
    registerCoreCommands(plugin, mcpClient);
}

/**
 * Register commands for each loaded prompt
 */
function registerPromptCommands(
    plugin: LabeledAnnotations,
    promptLoader: PromptLoader,
    mcpClient: MCPClient | null
): void {
    const prompts = promptLoader.getAll();

    for (const prompt of prompts) {
        const command = createPromptCommand(plugin, prompt, promptLoader, mcpClient);
        plugin.addCommand(command);
    }

    // Register custom prompt picker command
    plugin.addCommand({
        id: CORE_COMMANDS.AI_CUSTOM,
        name: 'Doc Doctor: Custom Prompt...',
        icon: 'message-square',
        callback: () => {
            // TODO: Open prompt picker modal
            new Notice('Custom prompt picker not yet implemented');
        },
    });
}

/**
 * Create a command for a specific prompt
 */
function createPromptCommand(
    plugin: LabeledAnnotations,
    prompt: PromptDefinition,
    promptLoader: PromptLoader,
    mcpClient: MCPClient | null
): Command {
    return {
        id: `${AI_COMMAND_PREFIX}${prompt.id}`,
        name: `Doc Doctor: ${prompt.name}`,
        icon: prompt.icon,

        // Use editorCheckCallback for context-aware availability
        editorCheckCallback: (checking: boolean, editor: Editor, view: MarkdownView): boolean => {
            const ctx = buildExecutionContext(plugin.app);

            // Check if requirements are met
            if (!promptLoader.meetsRequirements(prompt, ctx)) {
                return false;
            }

            if (!checking) {
                executePrompt(plugin, prompt, ctx, mcpClient);
            }

            return true;
        },
    };
}

/**
 * Execute a prompt with the current context
 */
async function executePrompt(
    plugin: LabeledAnnotations,
    prompt: PromptDefinition,
    ctx: ReturnType<typeof buildExecutionContext>,
    mcpClient: MCPClient | null
): Promise<void> {
    // Check if MCP is available
    if (mcpClient && !mcpClient.isConnected()) {
        new Notice('MCP not connected. Check settings.');
        return;
    }

    const content = getAnalysisContent(ctx);

    if (!content) {
        new Notice('No content to analyze');
        return;
    }

    new Notice(`Running: ${prompt.name}...`);

    try {
        // TODO: Implement full prompt execution with LLM
        // For now, just show a preview of what would happen

        if (mcpClient) {
            const tools = new MCPTools(mcpClient);
            const analysis = await tools.analyzeDocument(content);

            // Show analysis results
            console.debug('[AI Command] Analysis result:', analysis);
            new Notice(
                `Health: ${(analysis.dimensions.health * 100).toFixed(0)}% | ` +
                `Stubs: ${analysis.dimensions.stub_count} | ` +
                `Blocking: ${analysis.dimensions.blocking_count}`
            );
        } else {
            // Fallback without MCP
            new Notice('MCP not available. Enable in settings.');
        }
    } catch (error) {
        console.error('[AI Command] Error:', error);
        new Notice(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
}

/**
 * Register core document operation commands
 */
function registerCoreCommands(
    plugin: LabeledAnnotations,
    mcpClient: MCPClient | null
): void {
    // Parse Document
    plugin.addCommand({
        id: CORE_COMMANDS.PARSE_DOCUMENT,
        name: 'Doc Doctor: Parse Document',
        icon: 'file-text',
        editorCallback: async (editor: Editor, view: MarkdownView) => {
            if (!mcpClient?.isConnected()) {
                new Notice('MCP not connected');
                return;
            }

            const tools = new MCPTools(mcpClient);
            const content = editor.getValue();

            try {
                const result = await tools.parseDocument(content);
                console.log('[Parse]', result);
                new Notice(`Parsed: ${result.title || 'Untitled'} (${result.stubs.length} stubs)`);
            } catch (error) {
                new Notice(`Parse error: ${error}`);
            }
        },
    });

    // Analyze Document
    plugin.addCommand({
        id: CORE_COMMANDS.ANALYZE_DOCUMENT,
        name: 'Doc Doctor: Analyze Document',
        icon: 'activity',
        editorCallback: async (editor: Editor, view: MarkdownView) => {
            if (!mcpClient?.isConnected()) {
                new Notice('MCP not connected');
                return;
            }

            const tools = new MCPTools(mcpClient);
            const content = editor.getValue();

            try {
                const result = await tools.analyzeDocument(content);
                const health = (result.dimensions.health * 100).toFixed(0);
                const margin = (result.dimensions.usefulness.margin * 100).toFixed(0);
                new Notice(`Health: ${health}% | Margin: ${margin}% | Stubs: ${result.dimensions.stub_count}`);
            } catch (error) {
                new Notice(`Analysis error: ${error}`);
            }
        },
    });

    // Validate Document
    plugin.addCommand({
        id: CORE_COMMANDS.VALIDATE_DOCUMENT,
        name: 'Doc Doctor: Validate Document',
        icon: 'check-circle',
        editorCallback: async (editor: Editor, view: MarkdownView) => {
            if (!mcpClient?.isConnected()) {
                new Notice('MCP not connected');
                return;
            }

            const tools = new MCPTools(mcpClient);
            const content = editor.getValue();

            try {
                const result = await tools.validateDocument(content);
                if (result.valid) {
                    new Notice('Document is valid!');
                } else {
                    new Notice(`Validation errors: ${result.errors.length}`);
                    result.errors.forEach(e => console.warn('[Validation]', e));
                }
            } catch (error) {
                new Notice(`Validation error: ${error}`);
            }
        },
    });

    // Add Stub (opens type selector)
    plugin.addCommand({
        id: CORE_COMMANDS.ADD_STUB,
        name: 'Doc Doctor: Add Stub...',
        icon: 'plus-square',
        editorCallback: async (editor: Editor, view: MarkdownView) => {
            // TODO: Open stub type selector modal
            new Notice('Stub selector not yet implemented. Use specific stub commands.');
        },
    });

    // Quick stub commands
    const stubTypes = [
        { id: 'expand', name: 'Expand', icon: 'expand' },
        { id: 'link', name: 'Link', icon: 'link' },
        { id: 'verify', name: 'Verify', icon: 'check' },
        { id: 'question', name: 'Question', icon: 'help-circle' },
    ];

    for (const stubType of stubTypes) {
        plugin.addCommand({
            id: `doc-doctor:add-stub-${stubType.id}`,
            name: `Doc Doctor: Add ${stubType.name} Stub`,
            icon: stubType.icon,
            editorCallback: async (editor: Editor, view: MarkdownView) => {
                if (!mcpClient?.isConnected()) {
                    new Notice('MCP not connected');
                    return;
                }

                const tools = new MCPTools(mcpClient);
                const content = editor.getValue();
                const selection = editor.getSelection();
                const description = selection || `TODO: ${stubType.name} this section`;

                try {
                    const result = await tools.addStub(content, stubType.id, description);
                    editor.setValue(result.updated_content);
                    new Notice(`Added ${stubType.name} stub`);
                } catch (error) {
                    new Notice(`Error adding stub: ${error}`);
                }
            },
        });
    }

    // List Stubs
    plugin.addCommand({
        id: CORE_COMMANDS.LIST_STUBS,
        name: 'Doc Doctor: List Stubs',
        icon: 'list',
        editorCallback: async (editor: Editor, view: MarkdownView) => {
            if (!mcpClient?.isConnected()) {
                new Notice('MCP not connected');
                return;
            }

            const tools = new MCPTools(mcpClient);
            const content = editor.getValue();

            try {
                const result = await tools.listStubs(content);
                if (result.stubs.length === 0) {
                    new Notice('No stubs in document');
                } else {
                    new Notice(`Found ${result.stubs.length} stubs`);
                    console.log('[Stubs]', result.stubs);
                }
            } catch (error) {
                new Notice(`Error listing stubs: ${error}`);
            }
        },
    });

    // Find Anchors
    plugin.addCommand({
        id: CORE_COMMANDS.FIND_ANCHORS,
        name: 'Doc Doctor: Find Anchors',
        icon: 'anchor',
        editorCallback: async (editor: Editor, view: MarkdownView) => {
            if (!mcpClient?.isConnected()) {
                new Notice('MCP not connected');
                return;
            }

            const tools = new MCPTools(mcpClient);
            const content = editor.getValue();

            try {
                const result = await tools.findStubAnchors(content);
                if (result.anchors.length === 0) {
                    new Notice('No anchors found');
                } else {
                    new Notice(`Found ${result.anchors.length} anchors`);
                    console.log('[Anchors]', result.anchors);
                }
            } catch (error) {
                new Notice(`Error finding anchors: ${error}`);
            }
        },
    });

    // Calculate Health
    plugin.addCommand({
        id: CORE_COMMANDS.CALCULATE_HEALTH,
        name: 'Doc Doctor: Calculate Health',
        icon: 'heart-pulse',
        editorCallback: async (editor: Editor, view: MarkdownView) => {
            if (!mcpClient?.isConnected()) {
                new Notice('MCP not connected');
                return;
            }

            const tools = new MCPTools(mcpClient);
            const content = editor.getValue();

            try {
                const analysis = await tools.analyzeDocument(content);
                const health = (analysis.dimensions.health * 100).toFixed(1);
                new Notice(`Document Health: ${health}%`);
            } catch (error) {
                new Notice(`Error calculating health: ${error}`);
            }
        },
    });

    // Calculate Usefulness
    plugin.addCommand({
        id: CORE_COMMANDS.CALCULATE_USEFULNESS,
        name: 'Doc Doctor: Calculate Usefulness',
        icon: 'trending-up',
        editorCallback: async (editor: Editor, view: MarkdownView) => {
            if (!mcpClient?.isConnected()) {
                new Notice('MCP not connected');
                return;
            }

            const tools = new MCPTools(mcpClient);
            const content = editor.getValue();

            try {
                const analysis = await tools.analyzeDocument(content);
                const margin = (analysis.dimensions.usefulness.margin * 100).toFixed(1);
                const meetsGate = analysis.dimensions.usefulness.meets_gate ? 'Yes' : 'No';
                new Notice(`Usefulness Margin: ${margin}% | Meets Gate: ${meetsGate}`);
            } catch (error) {
                new Notice(`Error calculating usefulness: ${error}`);
            }
        },
    });
}
