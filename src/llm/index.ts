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
