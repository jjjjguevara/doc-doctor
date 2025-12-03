/**
 * Smart Connections Service - Type Definitions
 *
 * Types for the unified Smart Connections API that provides vault-wide
 * semantic search, related notes discovery, and link suggestions.
 */

import type { ParsedStub } from '../stubs/stubs-types';

// =============================================================================
// CORE RESULT TYPES
// =============================================================================

/**
 * A related note found via semantic search or embedding similarity
 */
export interface RelatedNote {
    /** Full path to the note in vault */
    path: string;
    /** Note title (basename without extension) */
    title: string;
    /** Similarity score (0-1, higher = more similar) */
    similarity: number;
    /** Preview text excerpt showing relevant content */
    excerpt: string;
    /** Keywords that matched (for keyword-based search) */
    matchedKeywords?: string[];
    /** How this result was found */
    method: 'embedding' | 'keyword';
}

/**
 * A suggested wikilink opportunity found in document content
 */
export interface LinkSuggestion {
    /** Line number where the term appears (1-indexed) */
    line: number;
    /** The text that could be linked */
    text: string;
    /** Full path to the suggested link target */
    targetPath: string;
    /** Title of the target note */
    targetTitle: string;
    /** Confidence score for this suggestion (0-1) */
    confidence: number;
    /** Context around the matched text */
    context?: string;
}

/**
 * A potential duplicate note detected via high similarity
 */
export interface DuplicateCandidate {
    /** Full path to the potential duplicate */
    path: string;
    /** Title of the potential duplicate */
    title: string;
    /** Similarity score (0-1, higher = more similar) */
    similarity: number;
    /** Shared phrases or sections between documents */
    sharedContent: string[];
}

/**
 * Context information for resolving a stub via related notes
 */
export interface StubContext {
    /** The stub being analyzed */
    stub: ParsedStub;
    /** Notes that might help resolve this stub */
    relevantNotes: RelatedNote[];
    /** Suggested wikilinks in [[link]] format */
    suggestedLinks: string[];
}

// =============================================================================
// SERVICE STATUS TYPES
// =============================================================================

/**
 * Status of the Smart Connections integration
 */
export interface SmartConnectionsStatus {
    /** Whether Smart Connections plugin is available */
    smartConnections: boolean;
    /** Number of embeddings available (0 if unavailable) */
    embeddingsCount: number;
    /** Whether we're using fallback keyword search */
    fallbackMode: boolean;
    /** Last status check timestamp */
    lastChecked: number;
    /** Any error message */
    error?: string;
}

// =============================================================================
// SERVICE INTERFACE
// =============================================================================

/**
 * Unified interface for Smart Connections operations
 */
export interface ISmartConnectionsService {
    // === Status ===

    /** Check if Smart Connections is available */
    isAvailable(): boolean;

    /** Get detailed status information */
    getStatus(): SmartConnectionsStatus;

    // === Core Search (Direct API - fast) ===

    /** Find notes semantically related to a given note */
    findRelated(notePath: string, limit?: number): Promise<RelatedNote[]>;

    /** Search vault semantically by query string */
    search(query: string, limit?: number): Promise<RelatedNote[]>;

    // === Batch Operations (MCP - async) ===

    /** Suggest wikilinks for content */
    suggestLinks(content: string): Promise<LinkSuggestion[]>;

    /** Detect potential duplicate notes */
    detectDuplicates(notePath: string, threshold?: number): Promise<DuplicateCandidate[]>;

    // === Stub Integration ===

    /** Find context to help resolve a stub */
    findContextForStub(stub: ParsedStub): Promise<StubContext>;

    // === Related Property ===

    /** Get suggested wikilinks for the `related:` frontmatter property */
    getSuggestedRelated(notePath: string, limit?: number): Promise<string[]>;

    /** Add wikilinks to a note's `related:` frontmatter property */
    addToRelatedProperty(notePath: string, wikilinks: string[]): Promise<void>;
}

// =============================================================================
// SETTINGS TYPES
// =============================================================================

/**
 * Smart Connections feature settings
 */
export interface SmartConnectionsSettings {
    /** Whether the Smart Connections integration is enabled */
    enabled: boolean;

    // Related notes
    /** Maximum number of related notes to show (default: 5) */
    relatedNotesLimit: number;
    /** Auto-populate `related:` property when saving (default: false) */
    autoPopulateRelated: boolean;
    /** Minimum similarity threshold for related notes (default: 0.2) */
    relatedThreshold: number;
    /** Frontmatter property name for related notes (default: 'related') */
    relatedPropertyName: string;

    // Duplicate detection
    /** Minimum similarity threshold to consider as duplicate (default: 0.8) */
    duplicateThreshold: number;
    /** Show duplicate warnings in Explore view (default: true) */
    warnOnDuplicates: boolean;

    // Link suggestions
    /** Enable link suggestion feature (default: true) */
    suggestLinks: boolean;
    /** Minimum confidence for link suggestions (default: 0.6) */
    linkSuggestionMinConfidence: number;

    // Performance
    /** Cache search results (default: true) */
    cacheResults: boolean;
    /** Cache duration in minutes (default: 5) */
    cacheDurationMinutes: number;
}

/**
 * Default settings for Smart Connections integration
 */
export const DEFAULT_SMART_CONNECTIONS_SETTINGS: SmartConnectionsSettings = {
    enabled: true,
    relatedNotesLimit: 5,
    autoPopulateRelated: false,
    relatedThreshold: 0.2, // Lowered from 0.7 - Smart Connections scores are often 0.2-0.6
    relatedPropertyName: 'related',
    duplicateThreshold: 0.8,
    warnOnDuplicates: true,
    suggestLinks: true,
    linkSuggestionMinConfidence: 0.6,
    cacheResults: true,
    cacheDurationMinutes: 5,
};

// =============================================================================
// SMART CONNECTIONS PLUGIN API TYPES
// =============================================================================

/**
 * Smart Connections plugin API (external plugin interface)
 * Based on Smart Connections v2.x API
 */
export interface SmartConnectionsPluginAPI {
    /** Find connections for a given file */
    find_connections(file: unknown): Promise<SmartConnectionsResult[]>;
    /** Search by query string */
    search?(query: string): Promise<SmartConnectionsResult[]>;
}

/**
 * Result from Smart Connections plugin API
 */
export interface SmartConnectionsResult {
    /** Path to the connected file */
    path: string;
    /** Basename of the file */
    basename: string;
    /** Similarity score */
    score: number;
    /** Optional excerpt */
    excerpt?: string;
    /** Optional heading within the file */
    heading?: string;
}

// =============================================================================
// EXPLORE VIEW STATE TYPES
// =============================================================================

/**
 * State for the Explore view store
 */
export interface ExploreViewState {
    /** Current active note path */
    activeNotePath: string | null;
    /** Current search query (empty = show related notes) */
    searchQuery: string;
    /** Search/related results */
    results: RelatedNote[];
    /** Whether results are loading */
    isLoading: boolean;
    /** Error message if any */
    error: string | null;
    /** Selected stub for context (from Stubs view) */
    selectedStubForContext: ParsedStub | null;
    /** Stub context results */
    stubContext: StubContext | null;
    /** Duplicate candidates */
    duplicates: DuplicateCandidate[];
    /** Link suggestions */
    linkSuggestions: LinkSuggestion[];
    /** Which sections are expanded */
    expandedSections: {
        relatedNotes: boolean;
        stubContext: boolean;
        duplicates: boolean;
        linkSuggestions: boolean;
    };
    /** Last refresh timestamp */
    lastRefresh: number;
}

/**
 * Default state for Explore view
 */
export const DEFAULT_EXPLORE_VIEW_STATE: ExploreViewState = {
    activeNotePath: null,
    searchQuery: '',
    results: [],
    isLoading: false,
    error: null,
    selectedStubForContext: null,
    stubContext: null,
    duplicates: [],
    linkSuggestions: [],
    expandedSections: {
        relatedNotes: true,
        stubContext: true,
        duplicates: true,
        linkSuggestions: false,
    },
    lastRefresh: 0,
};
