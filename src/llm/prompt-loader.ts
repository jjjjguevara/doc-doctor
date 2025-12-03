/**
 * Prompt Loader - YAML Prompt File Loader
 *
 * Loads and manages custom prompts from vault YAML files.
 */

import { App, TFile, TFolder, Notice } from 'obsidian';
import {
    PromptDefinition,
    PromptCategory,
    PromptsFile,
    PromptInput,
    normalizePrompt,
    validatePrompt,
    isSimplePrompt,
} from './prompt-schema';
import { BUILTIN_PROMPTS } from './builtin-prompts';

// Simple YAML parser for prompt files
// We use a basic implementation to avoid external dependencies
function parseYaml(content: string): unknown {
    // Obsidian has built-in YAML parsing via the metadataCache
    // For standalone parsing, we'll use a simple approach
    try {
        // Remove YAML frontmatter delimiters if present
        let yaml = content.trim();
        if (yaml.startsWith('---')) {
            yaml = yaml.slice(3);
            const endIdx = yaml.indexOf('---');
            if (endIdx !== -1) {
                yaml = yaml.slice(0, endIdx);
            }
        }

        // Use JSON.parse for JSON-formatted YAML (common subset)
        // For full YAML, we'd need the js-yaml library
        // This is a simplified implementation
        return parseSimpleYaml(yaml);
    } catch {
        return null;
    }
}

/**
 * Simple YAML parser for common prompt file patterns
 * Handles the subset of YAML commonly used in prompt files
 */
function parseSimpleYaml(content: string): unknown {
    const lines = content.split('\n');
    const result: Record<string, unknown> = {};
    let currentKey: string | null = null;
    let currentArray: unknown[] | null = null;
    let currentObject: Record<string, unknown> | null = null;
    let inMultiline = false;
    let multilineContent = '';
    let indentLevel = 0;

    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        const trimmed = line.trim();

        // Skip empty lines and comments
        if (!trimmed || trimmed.startsWith('#')) {
            if (inMultiline) {
                multilineContent += '\n';
            }
            continue;
        }

        // Check for multiline continuation
        if (inMultiline) {
            const lineIndent = line.search(/\S/);
            if (lineIndent > indentLevel) {
                multilineContent += (multilineContent ? '\n' : '') + trimmed;
                continue;
            } else {
                // End of multiline
                if (currentKey) {
                    result[currentKey] = multilineContent.trim();
                }
                inMultiline = false;
                multilineContent = '';
            }
        }

        // Parse key-value pairs
        const colonIdx = trimmed.indexOf(':');
        if (colonIdx > 0) {
            const key = trimmed.slice(0, colonIdx).trim();
            let value = trimmed.slice(colonIdx + 1).trim();

            // Handle multiline indicator
            if (value === '|' || value === '>') {
                inMultiline = true;
                currentKey = key;
                indentLevel = line.search(/\S/);
                continue;
            }

            // Handle array indicator
            if (value === '') {
                // Could be start of array or nested object
                currentKey = key;
                continue;
            }

            // Remove quotes
            if ((value.startsWith('"') && value.endsWith('"')) ||
                (value.startsWith("'") && value.endsWith("'"))) {
                value = value.slice(1, -1);
            }

            // Parse value type
            if (value === 'true') {
                result[key] = true;
            } else if (value === 'false') {
                result[key] = false;
            } else if (value === 'null') {
                result[key] = null;
            } else if (!isNaN(Number(value)) && value !== '') {
                result[key] = Number(value);
            } else {
                result[key] = value;
            }
        }

        // Handle array items
        if (trimmed.startsWith('- ')) {
            const arrayValue = trimmed.slice(2).trim();
            if (currentKey && !result[currentKey]) {
                result[currentKey] = [];
            }
            if (currentKey && Array.isArray(result[currentKey])) {
                (result[currentKey] as unknown[]).push(arrayValue);
            }
        }
    }

    // Handle any remaining multiline content
    if (inMultiline && currentKey) {
        result[currentKey] = multilineContent.trim();
    }

    return result;
}

/**
 * Execution context for prompt filtering
 */
export interface ExecutionContext {
    /** Current file (null if no file open) */
    file: TFile | null;

    /** Document content */
    content: string;

    /** Whether text is selected */
    hasSelection: boolean;

    /** Selected text */
    selection: string;

    /** Selection range */
    selectionRange: { start: number; end: number } | null;

    /** Cursor position */
    cursorPosition: number;

    /** Current line number */
    currentLine: number;

    /** Current line content */
    currentLineContent: string;
}

/**
 * Prompt Loader class
 * Manages loading and caching of prompt definitions
 */
export class PromptLoader {
    private builtinPrompts: PromptDefinition[] = [];
    private vaultPrompts: PromptDefinition[] = [];
    private app: App;
    private promptsPath: string;
    private showBuiltin: boolean;

    constructor(app: App, promptsPath: string, showBuiltin = true) {
        this.app = app;
        this.promptsPath = promptsPath;
        this.showBuiltin = showBuiltin;
        this.builtinPrompts = BUILTIN_PROMPTS.map(p => normalizePrompt(p, 'builtin'));
    }

    /**
     * Load prompts from vault
     */
    async loadFromVault(): Promise<void> {
        this.vaultPrompts = [];

        // Check if prompts folder exists
        const folder = this.app.vault.getAbstractFileByPath(this.promptsPath);
        if (!folder || !(folder instanceof TFolder)) {
            console.debug('[PromptLoader] Prompts folder not found:', this.promptsPath);
            return;
        }

        // Load all YAML files from the folder
        const files = folder.children.filter(
            (f): f is TFile => f instanceof TFile && (f.extension === 'yaml' || f.extension === 'yml')
        );

        for (const file of files) {
            try {
                const prompts = await this.loadPromptFile(file);
                this.vaultPrompts.push(...prompts);
            } catch (error) {
                console.error(`[PromptLoader] Error loading ${file.path}:`, error);
                new Notice(`Error loading prompt file: ${file.name}`);
            }
        }

        console.debug(`[PromptLoader] Loaded ${this.vaultPrompts.length} prompts from vault`);
    }

    /**
     * Load prompts from a single YAML file
     */
    private async loadPromptFile(file: TFile): Promise<PromptDefinition[]> {
        const content = await this.app.vault.read(file);
        const parsed = parseYaml(content);

        if (!parsed || typeof parsed !== 'object') {
            console.warn(`[PromptLoader] Invalid YAML in ${file.path}`);
            return [];
        }

        const prompts: PromptDefinition[] = [];
        const data = parsed as Record<string, unknown>;

        // Check for multi-prompt format (prompts array)
        if ('prompts' in data && Array.isArray(data.prompts)) {
            for (const prompt of data.prompts) {
                const validation = validatePrompt(prompt);
                if (validation.valid) {
                    prompts.push(normalizePrompt(prompt as PromptInput, 'vault', file.path));
                } else {
                    console.warn(`[PromptLoader] Invalid prompt in ${file.path}:`, validation.errors);
                }
                if (validation.warnings.length > 0) {
                    console.debug(`[PromptLoader] Warnings for prompt in ${file.path}:`, validation.warnings);
                }
            }
        } else {
            // Single prompt format
            const validation = validatePrompt(data);
            if (validation.valid) {
                prompts.push(normalizePrompt(data as PromptInput, 'vault', file.path));
            } else {
                console.warn(`[PromptLoader] Invalid prompt file ${file.path}:`, validation.errors);
            }
        }

        return prompts;
    }

    /**
     * Get all available prompts
     */
    getAll(): PromptDefinition[] {
        const all = this.showBuiltin
            ? [...this.builtinPrompts, ...this.vaultPrompts]
            : [...this.vaultPrompts];

        // Deduplicate by ID (vault prompts override builtin)
        const byId = new Map<string, PromptDefinition>();
        for (const prompt of all) {
            byId.set(prompt.id, prompt);
        }

        return Array.from(byId.values());
    }

    /**
     * Get a prompt by ID
     */
    getById(id: string): PromptDefinition | undefined {
        // Check vault prompts first (they override builtin)
        const vaultPrompt = this.vaultPrompts.find(p => p.id === id);
        if (vaultPrompt) return vaultPrompt;

        if (this.showBuiltin) {
            return this.builtinPrompts.find(p => p.id === id);
        }

        return undefined;
    }

    /**
     * Get prompts by category
     */
    getByCategory(category: PromptCategory): PromptDefinition[] {
        return this.getAll().filter(p => p.category === category);
    }

    /**
     * Get prompts that match the current execution context
     */
    getContextualPrompts(context: ExecutionContext): PromptDefinition[] {
        return this.getAll().filter(prompt => this.meetsRequirements(prompt, context));
    }

    /**
     * Check if a prompt's requirements are met by the context
     */
    meetsRequirements(prompt: PromptDefinition, context: ExecutionContext): boolean {
        const { context: req } = prompt;

        // Check file requirement
        if (req.requires_file && !context.file) {
            return false;
        }

        // Check selection requirement
        if (req.requires_selection && !context.hasSelection) {
            return false;
        }

        // Check file type if specified
        if (req.file_types && req.file_types.length > 0 && context.file) {
            if (!req.file_types.includes(context.file.extension)) {
                return false;
            }
        }

        return true;
    }

    /**
     * Update configuration
     */
    configure(promptsPath: string, showBuiltin: boolean): void {
        this.promptsPath = promptsPath;
        this.showBuiltin = showBuiltin;
    }

    /**
     * Get builtin prompts (for testing/display)
     */
    getBuiltinPrompts(): PromptDefinition[] {
        return [...this.builtinPrompts];
    }

    /**
     * Get vault prompts (for testing/display)
     */
    getVaultPrompts(): PromptDefinition[] {
        return [...this.vaultPrompts];
    }
}
