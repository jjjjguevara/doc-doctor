/**
 * Stubs Module - Main Entry Point
 *
 * Exports all stubs functionality for integration with the main plugin.
 */

// Types
export * from './stubs-types';

// Configuration and defaults
export { DEFAULT_STUBS_CONFIGURATION, getStubTypeByKey, getSortedStubTypes } from './stubs-defaults';

// State management
export * from './stubs-store';
export { stubsSettingsReducer } from './stubs-settings-reducer';

// Helpers
export { parseStubsFrontmatter } from './helpers/stubs-parser';
export {
    parseInlineAnchors,
    getValidAnchors,
    generateAnchorId,
    insertAnchorAtLine,
    removeAnchorFromContent,
} from './helpers/anchor-utils';
export {
    performSync,
    insertStubAtCursorCommand,
    resolveOrphanedStub,
    resolveOrphanedStubCommand,
} from './helpers/stubs-sync';
export {
    navigateToAnchor,
    navigateToAnchorDirect,
    navigateToStub,
    navigateToLine,
    highlightAnchor,
} from './helpers/stubs-navigation';

// Editor plugin
export {
    stubsEditorPlugin,
    StubsEditorPlugin,
    triggerStubsDecorationUpdate,
    refreshStubsDecorations,
    stubAnchorStyles,
} from './editor';

// Editor suggest
export { StubSuggest, stubSuggestStyles } from './editor-suggest/stub-suggest';

// Commands
export { registerStubsCommands } from './commands';

// Settings
export { StubsSettings } from './settings/stubs-settings';
