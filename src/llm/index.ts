/**
 * LLM Module - Public API
 *
 * Exports for LLM-powered stub suggestions feature.
 */

// Types
export type {
    LLMProvider,
    LLMConfiguration,
    LLMDebugConfig,
    SuggestedStub,
    LLMSuggestionResponse,
    DocumentContext,
    LLMErrorType,
    LLMError,
    RequestHistoryEntry,
} from './llm-types';

export {
    DEFAULT_LLM_CONFIGURATION,
    AVAILABLE_MODELS,
    ERROR_MESSAGES,
} from './llm-types';

// Service
export { LLMService, getLLMService, resetLLMService } from './llm-service';

// Prompts
export {
    DEFAULT_SYSTEM_PROMPT,
    buildUserPrompt,
    validateLLMResponse,
    estimateTokens,
    estimateCost,
    getSortedStubTypesForPrompt,
    getConfiguredTypeKeys,
    buildDocumentContext,
} from './llm-prompts';

// Reducer
export type { LLMSettingsActions } from './llm-settings-reducer';
export { llmSettingsReducer } from './llm-settings-reducer';

// Prompts Settings Reducer
export type { PromptsSettingsActions } from './prompts-settings-reducer';
export { promptsSettingsReducer } from './prompts-settings-reducer';

// Prompt Schema
export type {
    PromptDefinition,
    PromptCategory,
    PromptContext,
    PromptBehavior,
    PromptSettings,
    SimplePromptDefinition,
    PromptsFile,
    SelectionType,
    PartialPromptDefinition,
    PromptInput,
} from './prompt-schema';

export {
    DEFAULT_CONTEXT,
    DEFAULT_BEHAVIOR,
    DEFAULT_PROMPT_SETTINGS,
    isSimplePrompt,
    isFullPrompt,
    normalizePrompt,
    validatePrompt,
} from './prompt-schema';

// Prompt Loader
export { PromptLoader } from './prompt-loader';
export type { ExecutionContext } from './prompt-loader';

// Built-in Prompts
export { BUILTIN_PROMPTS } from './builtin-prompts';
