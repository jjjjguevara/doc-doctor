/**
 * Direct Smart Connections API
 *
 * Wrapper around the Smart Connections Obsidian plugin API.
 * Provides fast, synchronous-style access to embedding-based search.
 */

import { App, TFile } from 'obsidian';
import type {
    RelatedNote,
    SmartConnectionsPluginAPI,
    SmartConnectionsResult,
    SmartConnectionsStatus,
} from './types';

// =============================================================================
// SMART CONNECTIONS PLUGIN API WRAPPER
// =============================================================================

/**
 * Direct wrapper for Smart Connections plugin API
 *
 * Uses the plugin's native API for fast, embedding-based search.
 * Falls back gracefully when the plugin is unavailable.
 */
export class DirectSmartConnectionsAPI {
    private api: SmartConnectionsPluginAPI | null = null;
    private lastStatusCheck = 0;
    private statusCacheDuration = 30000; // 30 seconds
    private cachedStatus: SmartConnectionsStatus | null = null;

    constructor(private app: App) {
        this.refreshApiReference();
    }

    // =========================================================================
    // API ACCESS
    // =========================================================================

    /**
     * Refresh the API reference (call after plugin reload)
     */
    refreshApiReference(): void {
        this.api = this.getSmartConnectionsAPI();
        this.cachedStatus = null;
    }

    /**
     * Get the Smart Connections plugin API if available
     *
     * Smart Connections v2 exposes API through plugin.env.smart_sources
     * We need to access the environment to use semantic search
     */
    private getSmartConnectionsAPI(): SmartConnectionsPluginAPI | null {
        try {
            // Access the Smart Connections plugin
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const plugins = (this.app as any).plugins?.plugins;
            if (!plugins) {
                console.log('[SmartConnections] No plugins object found');
                return null;
            }

            const scPlugin = plugins['smart-connections'];
            if (!scPlugin) {
                console.log('[SmartConnections] Plugin not installed');
                return null;
            }

            if (!scPlugin._loaded) {
                console.log('[SmartConnections] Plugin not loaded/enabled');
                return null;
            }

            // Smart Connections v2 uses env.smart_sources for embeddings
            // Check for the environment object
            const env = scPlugin.env;
            if (!env) {
                console.log('[SmartConnections] No env object - plugin may still be initializing');
                return null;
            }

            // Check for smart_sources which contains the embeddings
            const smartSources = env.smart_sources;
            if (!smartSources) {
                console.log('[SmartConnections] No smart_sources - embeddings may not be ready');
                return null;
            }

            // Helper to strip block anchors from paths (e.g., "file.md#Heading" -> "file.md")
            const stripBlockAnchor = (path: string): string => {
                // Smart Connections returns block-level paths like "file.md#Heading#Subheading"
                // We need just the file path for file operations
                const hashIndex = path.indexOf('#');
                return hashIndex > 0 ? path.substring(0, hashIndex) : path;
            };

            // Helper to safely extract path from connection object
            const getConnectionPath = (conn: any): string | null => {
                // Smart Connections can return connections in various formats
                // Try different property names
                let rawPath: string | null = null;

                if (typeof conn === 'string') rawPath = conn;
                else if (conn?.key) rawPath = conn.key;
                else if (conn?.path) rawPath = conn.path;
                else if (conn?.data?.path) rawPath = conn.data.path;
                else if (conn?.item?.key) rawPath = conn.item.key;
                else if (conn?.item?.path) rawPath = conn.item.path;
                // Some versions use the connection itself as the key holder
                else if (conn?.source?.key) rawPath = conn.source.key;

                // Strip block anchors from the path
                return rawPath ? stripBlockAnchor(rawPath) : null;
            };

            const getConnectionScore = (conn: any): number => {
                if (typeof conn?.score === 'number') return conn.score;
                if (typeof conn?.sim === 'number') return conn.sim;
                if (typeof conn?.similarity === 'number') return conn.similarity;
                return 0;
            };

            // Create a wrapper API that uses the env
            // Smart Connections v2 API structure
            const api: SmartConnectionsPluginAPI = {
                find_connections: async (source: TFile | { key: string }) => {
                    const key = 'key' in source ? source.key : source.path;
                    console.log('[SmartConnections] Looking for source:', key);

                    const sourceItem = smartSources.get(key);
                    if (!sourceItem) {
                        console.log('[SmartConnections] Source NOT found in smart_sources:', key);
                        // Log available keys sample for debugging
                        const allKeys = smartSources.keys ? smartSources.keys.slice(0, 5) : [];
                        console.log('[SmartConnections] Sample of available keys:', allKeys);
                        return [];
                    }

                    console.log('[SmartConnections] Source found, has vec:', !!sourceItem.vec);

                    // Use the find_connections method if available on the source item
                    if (typeof sourceItem.find_connections === 'function') {
                        const connections = await sourceItem.find_connections();
                        console.log('[SmartConnections] Found', connections.length, 'connections');

                        // Map and deduplicate results (multiple blocks from same file -> one result)
                        const seenPaths = new Set<string>();
                        const results = connections
                            .map((conn: any) => {
                                const connPath = getConnectionPath(conn);
                                if (!connPath) {
                                    console.warn('[SmartConnections] Could not extract path from:', conn);
                                    return null;
                                }
                                // Skip duplicates (keep first/highest scored)
                                if (seenPaths.has(connPath)) {
                                    return null;
                                }
                                seenPaths.add(connPath);

                                return {
                                    key: connPath,
                                    path: connPath,
                                    basename: connPath.split('/').pop()?.replace('.md', '') || connPath,
                                    score: getConnectionScore(conn),
                                    excerpt: conn.excerpt || conn.item?.data?.excerpt || conn.data?.excerpt || '',
                                };
                            })
                            .filter((conn: any): conn is NonNullable<typeof conn> => conn !== null);

                        console.log('[SmartConnections] After dedup:', results.length, 'unique files');
                        if (results.length > 0) {
                            console.log('[SmartConnections] First result:', results[0].path, 'score:', results[0].score);
                        }
                        return results;
                    }

                    // Alternative: use env.smart_connections if available
                    if (env.smart_connections?.find_connections) {
                        return env.smart_connections.find_connections(key);
                    }

                    return [];
                },
                search: async (query: string) => {
                    // Try to use lookup or search method
                    if (typeof smartSources.lookup === 'function') {
                        const results = await smartSources.lookup({ hypotheticals: [query] });

                        // Debug: log first result structure (safe for circular refs)
                        if (results.length > 0) {
                            const sample = results[0];
                            console.log('[SmartConnections] Search result sample - available keys:', Object.keys(sample || {}));
                        }

                        // Map and deduplicate results
                        const seenSearchPaths = new Set<string>();
                        return results
                            .map((r: any) => {
                                const rPath = getConnectionPath(r);
                                if (!rPath) {
                                    console.warn('[SmartConnections] Could not extract path from search result:', r);
                                    return null;
                                }
                                // Skip duplicates
                                if (seenSearchPaths.has(rPath)) {
                                    return null;
                                }
                                seenSearchPaths.add(rPath);

                                return {
                                    key: rPath,
                                    path: rPath,
                                    basename: rPath.split('/').pop()?.replace('.md', '') || rPath,
                                    score: getConnectionScore(r),
                                    excerpt: r.excerpt || r.item?.data?.excerpt || r.data?.excerpt || '',
                                };
                            })
                            .filter((r: any): r is NonNullable<typeof r> => r !== null);
                    }

                    // Fallback: env.smart_connections.search
                    if (env.smart_connections?.search) {
                        return env.smart_connections.search(query);
                    }

                    console.log('[SmartConnections] No search method available');
                    return [];
                },
            };

            // Store reference to the plugin for status checks
            this.scPlugin = scPlugin;

            console.log('[SmartConnections] API wrapper created successfully');
            return api;
        } catch (error) {
            console.error('[SmartConnections] Error accessing plugin API:', error);
            return null;
        }
    }

    // Reference to the Smart Connections plugin for status checks
    private scPlugin: any = null;

    // =========================================================================
    // STATUS
    // =========================================================================

    /**
     * Check if Smart Connections is available
     */
    isAvailable(): boolean {
        if (!this.api) {
            this.refreshApiReference();
        }
        return this.api !== null;
    }

    /**
     * Check if Smart Connections is fully loaded and ready for queries
     * Smart Connections v3 takes 10-25 seconds to load all embeddings
     */
    isReady(): boolean {
        if (!this.scPlugin?.env?.smart_sources) {
            return false;
        }

        // Check if smart_sources has finished loading
        // The load_queue processing indicates loading is in progress
        const smartSources = this.scPlugin.env.smart_sources;

        // Check for loading state indicators
        if (smartSources._loading || smartSources.load_queue?.length > 0) {
            return false;
        }

        // Check if items are actually loaded with embeddings
        const keys = smartSources.keys || [];
        if (keys.length === 0) {
            return false;
        }

        // Sample check: verify at least one item has embeddings
        const sampleItem = smartSources.get(keys[0]);
        if (!sampleItem?.vec && !sampleItem?.data?.embeddings) {
            return false;
        }

        return true;
    }

    /**
     * Get detailed status information
     */
    getStatus(): SmartConnectionsStatus {
        const now = Date.now();

        // Use cached status if recent
        if (this.cachedStatus && now - this.lastStatusCheck < this.statusCacheDuration) {
            return this.cachedStatus;
        }

        // Refresh API reference
        this.refreshApiReference();

        const status: SmartConnectionsStatus = {
            smartConnections: this.api !== null,
            embeddingsCount: 0,
            fallbackMode: this.api === null,
            lastChecked: now,
        };

        // Try to get embeddings count if available
        if (this.scPlugin?.env?.smart_sources) {
            try {
                const smartSources = this.scPlugin.env.smart_sources;
                // Try different ways to get the count
                if (smartSources.items) {
                    status.embeddingsCount = Object.keys(smartSources.items).length;
                } else if (smartSources.keys) {
                    status.embeddingsCount = smartSources.keys.length;
                } else if (typeof smartSources.length === 'number') {
                    status.embeddingsCount = smartSources.length;
                }
            } catch {
                // Ignore - embeddings count is optional
            }
        }

        // Add diagnostic info
        if (!this.api) {
            const plugins = (this.app as any).plugins?.plugins;
            const scPlugin = plugins?.['smart-connections'];
            if (!scPlugin) {
                status.error = 'Smart Connections plugin not installed';
            } else if (!scPlugin._loaded) {
                status.error = 'Smart Connections plugin not enabled';
            } else if (!scPlugin.env) {
                status.error = 'Smart Connections still initializing (no env)';
            } else if (!scPlugin.env.smart_sources) {
                status.error = 'Smart Connections embeddings not ready';
            } else {
                status.error = 'Smart Connections API unavailable';
            }
        } else if (!this.isReady()) {
            // API exists but embeddings are still loading
            status.error = 'Smart Connections loading embeddings...';
        }

        this.cachedStatus = status;
        this.lastStatusCheck = now;

        return status;
    }

    // =========================================================================
    // CORE SEARCH
    // =========================================================================

    /**
     * Find notes semantically related to a given note
     *
     * @param notePath - Path to the source note
     * @param limit - Maximum number of results (default: 10)
     * @returns Array of related notes with similarity scores
     */
    async findRelated(notePath: string, limit = 10): Promise<RelatedNote[]> {
        // Auto-refresh API if not available or not ready
        // This handles the case where Smart Connections was still loading when we first checked
        if (!this.isAvailable() || !this.isReady()) {
            this.refreshApiReference();
            // Check again after refresh
            if (!this.isAvailable()) {
                console.log('[SmartConnections] Not available, skipping findRelated');
                return [];
            }
            if (!this.isReady()) {
                console.log('[SmartConnections] Still loading embeddings, results may be incomplete');
            }
        }

        try {
            // Get the file reference
            const file = this.app.vault.getAbstractFileByPath(notePath);
            if (!file || !(file instanceof TFile)) {
                console.warn(`[SmartConnections] File not found: ${notePath}`);
                return [];
            }

            // Call the Smart Connections API
            // The API expects the file key (path) as input
            const results = await this.api!.find_connections({
                key: notePath,
                limit,
            } as unknown as TFile);

            // Map results to our RelatedNote format
            return this.mapResults(results, limit);
        } catch (error) {
            console.error('[SmartConnections] Error finding related notes:', error);
            return [];
        }
    }

    /**
     * Search vault semantically by query string
     *
     * @param query - Search query
     * @param limit - Maximum number of results (default: 10)
     * @returns Array of matching notes with similarity scores
     */
    async search(query: string, limit = 10): Promise<RelatedNote[]> {
        // Auto-refresh API if not available or not ready
        if (!this.isAvailable() || !this.isReady()) {
            this.refreshApiReference();
            if (!this.isAvailable()) {
                console.log('[SmartConnections] Not available, skipping search');
                return [];
            }
        }

        try {
            // Check if the API has a search method
            if (this.api!.search) {
                const results = await this.api!.search(query);
                return this.mapResults(results, limit);
            }

            // Fallback: Use find_connections with query as a virtual file
            // This may not work depending on the Smart Connections version
            console.warn('[SmartConnections] Direct search not available, falling back');
            return [];
        } catch (error) {
            console.error('[SmartConnections] Error searching:', error);
            return [];
        }
    }

    // =========================================================================
    // HELPERS
    // =========================================================================

    /**
     * Map Smart Connections results to RelatedNote format
     */
    private mapResults(
        results: SmartConnectionsResult[] | Array<{ key: string; score: number }>,
        limit: number,
    ): RelatedNote[] {
        if (!Array.isArray(results)) {
            return [];
        }

        return results.slice(0, limit).map((result) => {
            // Handle both API formats
            const path = 'path' in result ? result.path : result.key;
            const basename =
                'basename' in result
                    ? result.basename
                    : path.split('/').pop()?.replace('.md', '') || path;
            const score = 'score' in result ? result.score : 0;
            const excerpt = 'excerpt' in result ? result.excerpt : undefined;

            return {
                path,
                title: basename,
                similarity: score,
                excerpt: excerpt || this.getExcerptFromPath(path),
                method: 'embedding' as const,
            };
        });
    }

    /**
     * Get a brief excerpt from a file (for display when API doesn't provide one)
     */
    private getExcerptFromPath(path: string): string {
        try {
            const file = this.app.vault.getAbstractFileByPath(path);
            if (file instanceof TFile) {
                // Get cached content if available
                const cache = this.app.metadataCache.getFileCache(file);
                if (cache?.frontmatter?.description) {
                    return cache.frontmatter.description as string;
                }
                if (cache?.frontmatter?.title) {
                    return `Note: ${cache.frontmatter.title}`;
                }
            }
        } catch {
            // Ignore errors
        }
        return '';
    }

    /**
     * Format a note path as a wikilink
     */
    formatAsWikilink(path: string): string {
        // Remove .md extension and format as wikilink
        const basename = path.replace(/\.md$/, '').split('/').pop() || path;
        return `[[${basename}]]`;
    }

    /**
     * Get suggested wikilinks for the `related:` frontmatter property
     *
     * @param notePath - Path to the source note
     * @param limit - Maximum number of suggestions (default: 5)
     * @returns Array of wikilinks in [[link]] format
     */
    async getSuggestedRelated(notePath: string, limit = 5): Promise<string[]> {
        const related = await this.findRelated(notePath, limit);
        return related.map((note) => this.formatAsWikilink(note.path));
    }
}
