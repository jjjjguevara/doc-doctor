/**
 * Smart Connections Service
 *
 * Unified service that provides semantic search, related notes discovery,
 * and link suggestions. Uses a hybrid approach:
 * - Direct Plugin API for fast, UI-driven operations
 * - Fallback keyword search when Smart Connections is unavailable
 *
 * MCP integration for batch operations will be added in a later phase.
 */

import { App, TFile } from 'obsidian';
import { DirectSmartConnectionsAPI } from './direct-api';
import { FallbackSearchService } from './fallback';
import {
    DEFAULT_SMART_CONNECTIONS_SETTINGS,
    type ISmartConnectionsService,
    type RelatedNote,
    type LinkSuggestion,
    type DuplicateCandidate,
    type StubContext,
    type SmartConnectionsStatus,
    type SmartConnectionsSettings,
} from './types';
import type { ParsedStub } from '../stubs/stubs-types';

// =============================================================================
// SMART CONNECTIONS SERVICE
// =============================================================================

/**
 * Unified Smart Connections service
 *
 * Provides a single interface for all semantic search and related notes
 * functionality. Automatically uses the best available method:
 * 1. Smart Connections plugin API (embedding-based)
 * 2. Fallback keyword search
 */
export class SmartConnectionsService implements ISmartConnectionsService {
    private directApi: DirectSmartConnectionsAPI;
    private fallbackSearch: FallbackSearchService;
    private settings: SmartConnectionsSettings;

    // Cache for related notes
    private cache = new Map<
        string,
        { results: RelatedNote[]; timestamp: number; method: 'embedding' | 'keyword' }
    >();

    constructor(
        private app: App,
        settings?: Partial<SmartConnectionsSettings>,
    ) {
        this.directApi = new DirectSmartConnectionsAPI(app);
        this.fallbackSearch = new FallbackSearchService(app);
        this.settings = { ...DEFAULT_SMART_CONNECTIONS_SETTINGS, ...settings };
    }

    // =========================================================================
    // SETTINGS
    // =========================================================================

    /**
     * Update service settings
     */
    updateSettings(settings: Partial<SmartConnectionsSettings>): void {
        this.settings = { ...this.settings, ...settings };
        // Clear cache when settings change
        this.cache.clear();
    }

    /**
     * Get current settings
     */
    getSettings(): SmartConnectionsSettings {
        return { ...this.settings };
    }

    // =========================================================================
    // STATUS
    // =========================================================================

    /**
     * Check if Smart Connections (embedding-based) is available
     */
    isAvailable(): boolean {
        return this.settings.enabled && this.directApi.isAvailable();
    }

    /**
     * Get detailed status information
     */
    getStatus(): SmartConnectionsStatus {
        if (!this.settings.enabled) {
            return {
                smartConnections: false,
                embeddingsCount: 0,
                fallbackMode: true,
                lastChecked: Date.now(),
                error: 'Smart Connections integration is disabled',
            };
        }
        return this.directApi.getStatus();
    }

    // =========================================================================
    // CORE SEARCH
    // =========================================================================

    /**
     * Find notes semantically related to a given note
     *
     * Uses Smart Connections if available, falls back to keyword search.
     *
     * @param notePath - Path to the source note
     * @param limit - Maximum number of results (default from settings)
     * @returns Array of related notes with similarity scores
     */
    async findRelated(notePath: string, limit?: number): Promise<RelatedNote[]> {
        if (!this.settings.enabled) {
            return [];
        }

        const maxResults = limit ?? this.settings.relatedNotesLimit;

        // Check cache
        const cacheKey = `related:${notePath}:${maxResults}`;
        const cached = this.getCachedResults(cacheKey);
        if (cached) {
            return cached;
        }

        let results: RelatedNote[];

        // Try Smart Connections first
        if (this.directApi.isAvailable()) {
            results = await this.directApi.findRelated(notePath, maxResults);
        } else {
            // Fallback to keyword search
            results = await this.fallbackSearch.findRelated(notePath, maxResults);
        }

        // Filter by threshold
        const beforeFilter = results.length;
        results = results.filter((note) => note.similarity >= this.settings.relatedThreshold);
        console.log(
            `[SmartConnections] Filtered ${beforeFilter} results to ${results.length} (threshold: ${this.settings.relatedThreshold})`,
        );
        if (beforeFilter > 0 && results.length === 0) {
            console.warn('[SmartConnections] All results filtered out! Consider lowering relatedThreshold.');
        }

        // Cache results
        this.setCachedResults(cacheKey, results);

        return results;
    }

    /**
     * Search vault semantically by query string
     *
     * @param query - Search query
     * @param limit - Maximum number of results
     * @returns Array of matching notes with similarity scores
     */
    async search(query: string, limit?: number): Promise<RelatedNote[]> {
        if (!this.settings.enabled || !query.trim()) {
            return [];
        }

        const maxResults = limit ?? this.settings.relatedNotesLimit * 2;

        // Check cache
        const cacheKey = `search:${query}:${maxResults}`;
        const cached = this.getCachedResults(cacheKey);
        if (cached) {
            return cached;
        }

        let results: RelatedNote[];

        // Try Smart Connections first
        if (this.directApi.isAvailable()) {
            results = await this.directApi.search(query, maxResults);

            // If Smart Connections search returns empty, fallback
            if (results.length === 0) {
                results = await this.fallbackSearch.search(query, maxResults);
            }
        } else {
            results = await this.fallbackSearch.search(query, maxResults);
        }

        // Cache results
        this.setCachedResults(cacheKey, results);

        return results;
    }

    // =========================================================================
    // BATCH OPERATIONS (MCP - TO BE IMPLEMENTED)
    // =========================================================================

    /**
     * Suggest wikilinks for content
     *
     * TODO: Implement MCP integration for batch link suggestions
     *
     * @param content - Document content to analyze
     * @returns Array of link suggestions
     */
    async suggestLinks(content: string): Promise<LinkSuggestion[]> {
        // Placeholder - will be implemented with MCP integration
        console.log('[SmartConnections] suggestLinks not yet implemented');
        return [];
    }

    /**
     * Detect potential duplicate notes
     *
     * TODO: Implement MCP integration for duplicate detection
     *
     * @param notePath - Path to the note to check
     * @param threshold - Similarity threshold (default from settings)
     * @returns Array of duplicate candidates
     */
    async detectDuplicates(notePath: string, threshold?: number): Promise<DuplicateCandidate[]> {
        if (!this.settings.warnOnDuplicates) {
            return [];
        }

        const dupThreshold = threshold ?? this.settings.duplicateThreshold;

        // Simple implementation using findRelated
        const related = await this.findRelated(notePath, 10);

        // Filter to high-similarity results that could be duplicates
        return related
            .filter((note) => note.similarity >= dupThreshold)
            .map((note) => ({
                path: note.path,
                title: note.title,
                similarity: note.similarity,
                sharedContent: note.matchedKeywords || [],
            }));
    }

    // =========================================================================
    // STUB INTEGRATION
    // =========================================================================

    /**
     * Find context to help resolve a stub
     *
     * Searches for notes that might contain information relevant to the stub.
     *
     * @param stub - The stub to find context for
     * @returns Context with relevant notes and suggested links
     */
    async findContextForStub(stub: ParsedStub): Promise<StubContext> {
        // Build search query from stub description and type
        const query = `${stub.type} ${stub.description}`;

        // Search for relevant notes
        const relevantNotes = await this.search(query, 5);

        // Generate suggested wikilinks from top results
        const suggestedLinks = relevantNotes.slice(0, 3).map((note) => `[[${note.title}]]`);

        return {
            stub,
            relevantNotes,
            suggestedLinks,
        };
    }

    // =========================================================================
    // RELATED PROPERTY
    // =========================================================================

    /**
     * Get suggested wikilinks for the `related:` frontmatter property
     *
     * @param notePath - Path to the source note
     * @param limit - Maximum number of suggestions
     * @returns Array of wikilinks in [[link]] format
     */
    async getSuggestedRelated(notePath: string, limit?: number): Promise<string[]> {
        const maxResults = limit ?? this.settings.relatedNotesLimit;
        const related = await this.findRelated(notePath, maxResults);
        return related.map((note) => `[[${note.title}]]`);
    }

    /**
     * Add wikilinks to a note's related frontmatter property
     *
     * Uses the configured property name (default: 'related')
     *
     * @param notePath - Path to the note to update
     * @param wikilinks - Array of wikilinks to add (in [[link]] format)
     */
    async addToRelatedProperty(notePath: string, wikilinks: string[]): Promise<void> {
        const file = this.app.vault.getAbstractFileByPath(notePath);
        if (!file || !(file instanceof TFile)) {
            throw new Error(`File not found: ${notePath}`);
        }

        // Get the configured property name
        const propName = this.settings.relatedPropertyName || 'related';

        // Read current content
        const content = await this.app.vault.read(file);

        // Parse frontmatter
        const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);
        if (!frontmatterMatch) {
            // No frontmatter - create it
            const newFrontmatter = `---\n${propName}:\n${wikilinks.map((l) => `  - "${l}"`).join('\n')}\n---\n\n`;
            await this.app.vault.modify(file, newFrontmatter + content);
            return;
        }

        const frontmatter = frontmatterMatch[1];
        const afterFrontmatter = content.slice(frontmatterMatch[0].length);

        // Escape special regex characters in property name
        const escapedPropName = propName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

        // Check if property already exists
        const propRegex = new RegExp(`${escapedPropName}:\\s*\\n((?:\\s+-.*\\n)*)`);
        if (frontmatter.includes(`${propName}:`)) {
            // Parse existing values
            const relatedMatch = frontmatter.match(propRegex);
            if (relatedMatch) {
                // Get existing links
                const existingLinks = new Set(
                    relatedMatch[1]
                        .split('\n')
                        .map((line) => line.trim().replace(/^-\s*["']?|["']?$/g, ''))
                        .filter(Boolean),
                );

                // Add new links (avoid duplicates)
                for (const link of wikilinks) {
                    existingLinks.add(link);
                }

                // Build new section
                const newRelated = `${propName}:\n${[...existingLinks].map((l) => `  - "${l}"`).join('\n')}\n`;

                // Replace in frontmatter
                const newFrontmatter = frontmatter.replace(propRegex, newRelated);
                await this.app.vault.modify(file, `---\n${newFrontmatter}\n---${afterFrontmatter}`);
            }
        } else {
            // Add property to frontmatter
            const newRelated = `${propName}:\n${wikilinks.map((l) => `  - "${l}"`).join('\n')}\n`;
            const newFrontmatter = frontmatter + '\n' + newRelated;
            await this.app.vault.modify(file, `---\n${newFrontmatter}\n---${afterFrontmatter}`);
        }
    }

    // =========================================================================
    // CACHE MANAGEMENT
    // =========================================================================

    /**
     * Get cached results if still valid
     */
    private getCachedResults(key: string): RelatedNote[] | null {
        if (!this.settings.cacheResults) {
            return null;
        }

        const cached = this.cache.get(key);
        if (!cached) {
            return null;
        }

        const maxAge = this.settings.cacheDurationMinutes * 60 * 1000;
        if (Date.now() - cached.timestamp > maxAge) {
            this.cache.delete(key);
            return null;
        }

        return cached.results;
    }

    /**
     * Cache results
     */
    private setCachedResults(key: string, results: RelatedNote[]): void {
        if (!this.settings.cacheResults) {
            return;
        }

        const method = results.length > 0 ? results[0].method : 'keyword';
        this.cache.set(key, {
            results,
            timestamp: Date.now(),
            method,
        });
    }

    /**
     * Clear all cached results
     */
    clearCache(): void {
        this.cache.clear();
    }

    /**
     * Refresh the Smart Connections API reference (call after plugin reload)
     */
    refreshApiReference(): void {
        this.directApi.refreshApiReference();
        this.clearCache();
    }
}

// =============================================================================
// FACTORY FUNCTION
// =============================================================================

/**
 * Create a new SmartConnectionsService instance
 */
export function createSmartConnectionsService(
    app: App,
    settings?: Partial<SmartConnectionsSettings>,
): SmartConnectionsService {
    return new SmartConnectionsService(app, settings);
}
