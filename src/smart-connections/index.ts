/**
 * Smart Connections Module
 *
 * Provides semantic search and related notes functionality.
 * Uses Smart Connections plugin when available, with keyword fallback.
 */

// Types
export type {
    RelatedNote,
    LinkSuggestion,
    DuplicateCandidate,
    StubContext,
    SmartConnectionsStatus,
    SmartConnectionsSettings,
    ISmartConnectionsService,
    ExploreViewState,
} from './types';

export { DEFAULT_SMART_CONNECTIONS_SETTINGS, DEFAULT_EXPLORE_VIEW_STATE } from './types';

// Service
export { SmartConnectionsService, createSmartConnectionsService } from './service';

// Direct API (for advanced use cases)
export { DirectSmartConnectionsAPI } from './direct-api';

// Fallback (for advanced use cases)
export { FallbackSearchService } from './fallback';
