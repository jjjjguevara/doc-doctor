/**
 * Explore View Store
 *
 * Reactive state management for the Explore view using Svelte stores.
 * Manages semantic search, related notes, and stub context state.
 */

import { writable, derived, get } from 'svelte/store';
import type {
    RelatedNote,
    DuplicateCandidate,
    LinkSuggestion,
    StubContext,
} from '../smart-connections/types';
import type { ParsedStub } from '../stubs/stubs-types';

// =============================================================================
// CORE STORES
// =============================================================================

/**
 * Current active note path being explored
 */
export const activeNotePath = writable<string | null>(null);

/**
 * Current note title (for display)
 */
export const activeNoteTitle = writable<string | null>(null);

/**
 * Current search query (empty = show related notes for active note)
 */
export const searchQuery = writable<string>('');

/**
 * Search/related note results
 */
export const results = writable<RelatedNote[]>([]);

/**
 * Loading state
 */
export const isLoading = writable<boolean>(false);

/**
 * Error message if any
 */
export const errorMessage = writable<string | null>(null);

/**
 * Selected stub for context analysis (set from Stubs view)
 */
export const selectedStubForContext = writable<ParsedStub | null>(null);

/**
 * Stub context results
 */
export const stubContext = writable<StubContext | null>(null);

/**
 * Duplicate candidates
 */
export const duplicates = writable<DuplicateCandidate[]>([]);

/**
 * Link suggestions
 */
export const linkSuggestions = writable<LinkSuggestion[]>([]);

/**
 * Last refresh timestamp
 */
export const lastRefresh = writable<number>(0);

// =============================================================================
// SECTION EXPANSION STATE
// =============================================================================

/**
 * Which sections are expanded
 */
export const expandedSections = writable<{
    relatedNotes: boolean;
    stubContext: boolean;
    duplicates: boolean;
    linkSuggestions: boolean;
}>({
    relatedNotes: true,
    stubContext: true,
    duplicates: true,
    linkSuggestions: false,
});

// =============================================================================
// DERIVED STORES
// =============================================================================

/**
 * Whether we're in search mode (query is not empty)
 */
export const isSearchMode = derived(searchQuery, ($query) => $query.trim().length > 0);

/**
 * Results count for display
 */
export const resultsCount = derived(results, ($results) => $results.length);

/**
 * Whether there are any results to show
 */
export const hasResults = derived(results, ($results) => $results.length > 0);

/**
 * Whether there's an active stub context
 */
export const hasStubContext = derived(stubContext, ($ctx) => $ctx !== null);

/**
 * Whether there are duplicate warnings
 */
export const hasDuplicates = derived(duplicates, ($dups) => $dups.length > 0);

/**
 * Whether there are link suggestions
 */
export const hasLinkSuggestions = derived(linkSuggestions, ($suggestions) => $suggestions.length > 0);

/**
 * Combined state for quick access
 */
export const exploreState = derived(
    [
        activeNotePath,
        activeNoteTitle,
        searchQuery,
        results,
        isLoading,
        errorMessage,
        selectedStubForContext,
        stubContext,
        duplicates,
        linkSuggestions,
        lastRefresh,
    ],
    ([
        $path,
        $title,
        $query,
        $results,
        $loading,
        $error,
        $stub,
        $stubCtx,
        $dups,
        $links,
        $refresh,
    ]) => ({
        activeNotePath: $path,
        activeNoteTitle: $title,
        searchQuery: $query,
        results: $results,
        isLoading: $loading,
        error: $error,
        selectedStubForContext: $stub,
        stubContext: $stubCtx,
        duplicates: $dups,
        linkSuggestions: $links,
        lastRefresh: $refresh,
    }),
);

// =============================================================================
// ACTIONS
// =============================================================================

/**
 * Set the active note being explored
 */
export function setActiveNote(path: string | null, title?: string): void {
    activeNotePath.set(path);
    activeNoteTitle.set(title ?? path?.split('/').pop()?.replace('.md', '') ?? null);
}

/**
 * Set search query
 */
export function setSearchQuery(query: string): void {
    searchQuery.set(query);
}

/**
 * Clear search query (return to related notes mode)
 */
export function clearSearch(): void {
    searchQuery.set('');
}

/**
 * Set results
 */
export function setResults(newResults: RelatedNote[]): void {
    results.set(newResults);
    lastRefresh.set(Date.now());
}

/**
 * Set loading state
 */
export function setLoading(loading: boolean): void {
    isLoading.set(loading);
}

/**
 * Set error
 */
export function setError(error: string | null): void {
    errorMessage.set(error);
}

/**
 * Clear error
 */
export function clearError(): void {
    errorMessage.set(null);
}

/**
 * Set stub for context analysis
 */
export function setStubForContext(stub: ParsedStub | null): void {
    selectedStubForContext.set(stub);
}

/**
 * Set stub context results
 */
export function setStubContext(context: StubContext | null): void {
    stubContext.set(context);
}

/**
 * Set duplicates
 */
export function setDuplicates(dups: DuplicateCandidate[]): void {
    duplicates.set(dups);
}

/**
 * Set link suggestions
 */
export function setLinkSuggestions(suggestions: LinkSuggestion[]): void {
    linkSuggestions.set(suggestions);
}

/**
 * Toggle section expansion
 */
export function toggleSection(
    section: 'relatedNotes' | 'stubContext' | 'duplicates' | 'linkSuggestions',
): void {
    expandedSections.update((current) => ({
        ...current,
        [section]: !current[section],
    }));
}

/**
 * Expand a section
 */
export function expandSection(
    section: 'relatedNotes' | 'stubContext' | 'duplicates' | 'linkSuggestions',
): void {
    expandedSections.update((current) => ({
        ...current,
        [section]: true,
    }));
}

/**
 * Collapse a section
 */
export function collapseSection(
    section: 'relatedNotes' | 'stubContext' | 'duplicates' | 'linkSuggestions',
): void {
    expandedSections.update((current) => ({
        ...current,
        [section]: false,
    }));
}

/**
 * Reset all state (on view close or note change)
 */
export function resetExploreState(): void {
    searchQuery.set('');
    results.set([]);
    isLoading.set(false);
    errorMessage.set(null);
    selectedStubForContext.set(null);
    stubContext.set(null);
    duplicates.set([]);
    linkSuggestions.set([]);
    lastRefresh.set(0);
}

/**
 * Clear results only (keep other state)
 */
export function clearResults(): void {
    results.set([]);
    duplicates.set([]);
    linkSuggestions.set([]);
}

// =============================================================================
// HELPERS
// =============================================================================

/**
 * Get current active note path
 */
export function getActiveNotePath(): string | null {
    return get(activeNotePath);
}

/**
 * Get current search query
 */
export function getSearchQuery(): string {
    return get(searchQuery);
}

/**
 * Check if currently loading
 */
export function isCurrentlyLoading(): boolean {
    return get(isLoading);
}
