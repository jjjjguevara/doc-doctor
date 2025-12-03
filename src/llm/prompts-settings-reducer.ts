/**
 * Prompts Settings Reducer
 *
 * Handles custom prompts configuration state updates.
 */

import { PromptSettings, PromptCategory } from './prompt-schema';

/**
 * Prompts settings actions
 */
export type PromptsSettingsActions =
    | { type: 'PROMPTS_SET_PATH'; payload: { path: string } }
    | { type: 'PROMPTS_SET_WATCH_FOR_CHANGES'; payload: { enabled: boolean } }
    | { type: 'PROMPTS_SET_DEFAULT_CATEGORY'; payload: { category: PromptCategory | 'all' } }
    | { type: 'PROMPTS_SET_SHOW_BUILTIN'; payload: { enabled: boolean } }
    | { type: 'PROMPTS_SET_CONFIRM_BEFORE_APPLY'; payload: { enabled: boolean } }
    | { type: 'PROMPTS_SET_SHOW_PREVIEW_PANEL'; payload: { enabled: boolean } }
    | { type: 'PROMPTS_SET_AUTO_INSERT_ANCHORS'; payload: { enabled: boolean } }
    | { type: 'PROMPTS_HIDE_PROMPT'; payload: { promptId: string } }
    | { type: 'PROMPTS_RESTORE_PROMPT'; payload: { promptId: string } }
    | { type: 'PROMPTS_RESTORE_ALL'; payload: Record<string, never> };

/**
 * Prompts settings reducer
 */
export const promptsSettingsReducer = (
    state: PromptSettings,
    action: PromptsSettingsActions
): void => {
    switch (action.type) {
        case 'PROMPTS_SET_PATH':
            state.promptsPath = action.payload.path;
            break;
        case 'PROMPTS_SET_WATCH_FOR_CHANGES':
            state.watchForChanges = action.payload.enabled;
            break;
        case 'PROMPTS_SET_DEFAULT_CATEGORY':
            state.defaultCategory = action.payload.category;
            break;
        case 'PROMPTS_SET_SHOW_BUILTIN':
            state.showBuiltinPrompts = action.payload.enabled;
            break;
        case 'PROMPTS_SET_CONFIRM_BEFORE_APPLY':
            state.confirmBeforeApply = action.payload.enabled;
            break;
        case 'PROMPTS_SET_SHOW_PREVIEW_PANEL':
            state.showPreviewPanel = action.payload.enabled;
            break;
        case 'PROMPTS_SET_AUTO_INSERT_ANCHORS':
            state.autoInsertAnchors = action.payload.enabled;
            break;
        case 'PROMPTS_HIDE_PROMPT':
            if (!state.hiddenPromptIds) {
                state.hiddenPromptIds = [];
            }
            if (!state.hiddenPromptIds.includes(action.payload.promptId)) {
                state.hiddenPromptIds.push(action.payload.promptId);
            }
            break;
        case 'PROMPTS_RESTORE_PROMPT':
            if (state.hiddenPromptIds) {
                state.hiddenPromptIds = state.hiddenPromptIds.filter(
                    (id) => id !== action.payload.promptId
                );
            }
            break;
        case 'PROMPTS_RESTORE_ALL':
            state.hiddenPromptIds = [];
            break;
    }
};
