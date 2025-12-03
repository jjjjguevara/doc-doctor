/**
 * Prompt Schema - Type Definitions
 *
 * Schema for custom YAML prompt definitions.
 * Taxonomy based on J-Editorial Agent Task Taxonomy.
 */

// =============================================================================
// TAXONOMY TYPES (from ref-agent-taxonomy.md)
// =============================================================================

/**
 * Task family classification for routing to appropriate capabilities
 */
export type TaskFamily =
    | 'generative'      // Family I: Generate new content (draft, cut, idea)
    | 'combinatorial'   // Family II: Retrieve and combine existing info (source, check, link)
    | 'synoptic'        // Family III: Analyze and synthesize across sources (data, model, fix)
    | 'operational'     // Family IV: Multi-step workflows with tool use (move, complex check)
    | 'learning';       // Family V: Meta-level system improvement

/**
 * Vector type for routing (maps to stub types)
 */
export type VectorType =
    | 'draft'   // Content generation (generative)
    | 'cut'     // Summarization (generative)
    | 'idea'    // Ideation/brainstorming (generative)
    | 'source'  // Citation search (combinatorial)
    | 'check'   // Fact verification (combinatorial)
    | 'link'    // Graph construction (combinatorial)
    | 'data'    // Data analysis (synoptic)
    | 'model'   // Modeling/projections (synoptic)
    | 'fix'     // Reconciliation (synoptic)
    | 'move';   // Refactoring (operational)

/**
 * Reliability tier for review pattern selection
 */
export type ReliabilityTier = 'high' | 'medium' | 'low';

// =============================================================================
// PROMPT DEFINITION
// =============================================================================

/**
 * Prompt category
 */
export type PromptCategory = 'analysis' | 'editing' | 'review' | 'custom';

/**
 * Selection type requirement
 */
export type SelectionType = 'block' | 'inline' | 'any';

/**
 * Context requirements for a prompt
 */
export interface PromptContext {
    /** Whether selection is required */
    requires_selection: boolean;

    /** Whether a file must be open */
    requires_file: boolean;

    /** Type of selection required (if requires_selection is true) */
    selection_type?: SelectionType;

    /** Allowed file types (extensions without dot) */
    file_types?: string[];
}

/**
 * Prompt behavior configuration
 */
export interface PromptBehavior {
    /** Ask for confirmation before applying changes */
    confirm_before_apply: boolean;

    /** Automatically insert anchors at suggested locations */
    auto_insert_anchors: boolean;

    /** Show preview panel with proposed changes */
    show_preview: boolean;
}

/**
 * Full prompt definition (structured format)
 */
export interface PromptDefinition {
    /** Unique identifier */
    id: string;

    /** Display name */
    name: string;

    /** Lucide icon name */
    icon?: string;

    /** Description of what this prompt does */
    description?: string;

    /** Category for organization */
    category: PromptCategory;

    /** Context requirements */
    context: PromptContext;

    /** System prompt extension (appended to base system prompt) */
    system_extension?: string;

    /** Complete system prompt override (replaces base prompt entirely) */
    system_override?: string;

    /** Behavior configuration */
    behavior: PromptBehavior;

    /** Suggested hotkey (Obsidian format, e.g., "Mod+Shift+A") */
    hotkey?: string;

    /** Source of the prompt definition */
    source: 'builtin' | 'vault' | 'plugin';

    /** File path if loaded from vault */
    filePath?: string;

    // =========================================================================
    // TAXONOMY FIELDS (J-Editorial Agent Task Taxonomy)
    // =========================================================================

    /** Task family classification for capability routing */
    task_family?: TaskFamily;

    /** Primary vector type this prompt addresses */
    vector_type?: VectorType;

    /** Expected reliability tier for review pattern selection */
    reliability?: ReliabilityTier;

    /** Whether this prompt is currently enabled/visible */
    enabled?: boolean;
}

/**
 * Simple prompt definition (unstructured format)
 * For users who want minimal configuration
 */
export interface SimplePromptDefinition {
    /** Unique identifier */
    id: string;

    /** Display name */
    name: string;

    /** Lucide icon name */
    icon?: string;

    /** The prompt content itself */
    prompt: string;
}

/**
 * Multi-prompt YAML file format
 */
export interface PromptsFile {
    prompts: Array<PromptDefinition | SimplePromptDefinition>;
}

/**
 * Partial prompt definition (without source/filePath)
 * Used for builtin prompts before normalization
 */
export type PartialPromptDefinition = Omit<PromptDefinition, 'source' | 'filePath'>;

// =============================================================================
// DEFAULT VALUES
// =============================================================================

/**
 * Default context requirements
 */
export const DEFAULT_CONTEXT: PromptContext = {
    requires_selection: false,
    requires_file: true,
    file_types: ['md'],
};

/**
 * Default behavior
 */
export const DEFAULT_BEHAVIOR: PromptBehavior = {
    confirm_before_apply: true,
    auto_insert_anchors: true,
    show_preview: true,
};

// =============================================================================
// TYPE GUARDS
// =============================================================================

/**
 * Check if a prompt definition is in the simple format
 */
export function isSimplePrompt(prompt: unknown): prompt is SimplePromptDefinition {
    if (typeof prompt !== 'object' || prompt === null) return false;
    const p = prompt as Record<string, unknown>;
    return typeof p.id === 'string' &&
           typeof p.name === 'string' &&
           typeof p.prompt === 'string' &&
           !('context' in p);
}

/**
 * Check if a prompt definition is in the full format (has context object)
 */
export function isFullPrompt(prompt: unknown): prompt is PartialPromptDefinition {
    if (typeof prompt !== 'object' || prompt === null) return false;
    const p = prompt as Record<string, unknown>;
    return typeof p.id === 'string' &&
           typeof p.name === 'string' &&
           typeof p.context === 'object';
}

/**
 * Input type for normalizePrompt - accepts partial or simple prompts
 */
export type PromptInput = PartialPromptDefinition | SimplePromptDefinition;

/**
 * Convert simple or partial prompt to full prompt definition
 */
export function normalizePrompt(prompt: PromptInput, source: 'builtin' | 'vault' | 'plugin', filePath?: string): PromptDefinition {
    if (isFullPrompt(prompt)) {
        return {
            ...prompt,
            source,
            filePath,
            context: { ...DEFAULT_CONTEXT, ...prompt.context },
            behavior: { ...DEFAULT_BEHAVIOR, ...prompt.behavior },
        };
    }

    // Convert simple format to full format
    const simplePrompt = prompt as SimplePromptDefinition;
    return {
        id: simplePrompt.id,
        name: simplePrompt.name,
        icon: simplePrompt.icon,
        description: undefined,
        category: 'custom',
        context: { ...DEFAULT_CONTEXT },
        system_extension: simplePrompt.prompt,
        behavior: { ...DEFAULT_BEHAVIOR },
        source,
        filePath,
    };
}

// =============================================================================
// VALIDATION
// =============================================================================

/**
 * Validation result
 */
export interface PromptValidationResult {
    valid: boolean;
    errors: string[];
    warnings: string[];
}

/**
 * Validate a prompt definition
 */
export function validatePrompt(prompt: unknown): PromptValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];

    if (typeof prompt !== 'object' || prompt === null) {
        return { valid: false, errors: ['Prompt must be an object'], warnings: [] };
    }

    const p = prompt as Record<string, unknown>;

    // Required fields
    if (!p.id || typeof p.id !== 'string') {
        errors.push('Missing or invalid "id" field');
    }
    if (!p.name || typeof p.name !== 'string') {
        errors.push('Missing or invalid "name" field');
    }

    // Must have either prompt (simple) or context (full)
    if (!('prompt' in p) && !('context' in p)) {
        errors.push('Must have either "prompt" (simple format) or "context" (full format)');
    }

    // Validate context if present
    if ('context' in p && typeof p.context === 'object' && p.context !== null) {
        const ctx = p.context as Record<string, unknown>;
        if ('requires_selection' in ctx && typeof ctx.requires_selection !== 'boolean') {
            errors.push('context.requires_selection must be a boolean');
        }
        if ('requires_file' in ctx && typeof ctx.requires_file !== 'boolean') {
            errors.push('context.requires_file must be a boolean');
        }
    }

    // Validate behavior if present
    if ('behavior' in p && typeof p.behavior === 'object' && p.behavior !== null) {
        const beh = p.behavior as Record<string, unknown>;
        if ('confirm_before_apply' in beh && typeof beh.confirm_before_apply !== 'boolean') {
            errors.push('behavior.confirm_before_apply must be a boolean');
        }
    }

    // Validate category if present
    if ('category' in p) {
        const validCategories = ['analysis', 'editing', 'review', 'custom'];
        if (!validCategories.includes(p.category as string)) {
            warnings.push(`Unknown category "${p.category}", defaulting to "custom"`);
        }
    }

    // Check for system prompt content
    if ('context' in p && !('system_extension' in p) && !('system_override' in p)) {
        warnings.push('No system prompt defined (system_extension or system_override)');
    }

    return {
        valid: errors.length === 0,
        errors,
        warnings,
    };
}

// =============================================================================
// PROMPT SETTINGS
// =============================================================================

/**
 * Prompt settings stored in plugin configuration
 */
export interface PromptSettings {
    /** Path to custom prompts folder (vault-relative) */
    promptsPath: string;

    /** Watch for changes and hot-reload */
    watchForChanges: boolean;

    /** Default category filter */
    defaultCategory: PromptCategory | 'all';

    /** Show built-in prompts alongside custom ones */
    showBuiltinPrompts: boolean;

    /** Default confirmation behavior */
    confirmBeforeApply: boolean;

    /** Default preview behavior */
    showPreviewPanel: boolean;

    /** Default anchor insertion behavior */
    autoInsertAnchors: boolean;

    /** IDs of prompts hidden by the user */
    hiddenPromptIds: string[];
}

/**
 * Default prompt settings
 */
export const DEFAULT_PROMPT_SETTINGS = (): PromptSettings => ({
    promptsPath: '.doc-doctor/prompts',
    watchForChanges: true,
    defaultCategory: 'all',
    showBuiltinPrompts: true,
    confirmBeforeApply: true,
    showPreviewPanel: true,
    autoInsertAnchors: true,
    hiddenPromptIds: [],
});
