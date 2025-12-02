/**
 * Firecrawl Service
 *
 * Provides web search and URL scraping capabilities via Firecrawl API.
 */

import { requestUrl } from 'obsidian';
import type {
    FirecrawlConfig,
    WebSearchResult,
    ScrapedContent,
    ExternalContext,
} from './llm-types';

const FIRECRAWL_API_BASE = 'https://api.firecrawl.dev/v1';

// =============================================================================
// SERVICE CLASS
// =============================================================================

export class FirecrawlService {
    private config: FirecrawlConfig;

    constructor(config: FirecrawlConfig) {
        this.config = config;
    }

    /**
     * Check if Firecrawl is properly configured
     */
    isConfigured(): boolean {
        return this.config.enabled && !!this.config.apiKey;
    }

    /**
     * Test the Firecrawl API connection
     */
    async testConnection(): Promise<{ success: boolean; message: string }> {
        if (!this.config.apiKey) {
            return { success: false, message: 'No API key configured' };
        }

        try {
            // Use a simple scrape request to test
            const response = await requestUrl({
                url: `${FIRECRAWL_API_BASE}/scrape`,
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${this.config.apiKey}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    url: 'https://example.com',
                    formats: ['markdown'],
                }),
                throw: false,
            });

            if (response.status === 200) {
                return { success: true, message: 'Connection successful' };
            } else if (response.status === 401) {
                return { success: false, message: 'Invalid API key' };
            } else {
                return { success: false, message: `API error: ${response.status}` };
            }
        } catch (error) {
            return { success: false, message: `Connection failed: ${(error as Error).message}` };
        }
    }

    /**
     * Search the web using Firecrawl
     */
    async search(query: string): Promise<WebSearchResult[]> {
        if (!this.isConfigured() || !this.config.webSearchEnabled) {
            return [];
        }

        try {
            const response = await requestUrl({
                url: `${FIRECRAWL_API_BASE}/search`,
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${this.config.apiKey}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    query,
                    limit: this.config.maxSearchResults,
                    scrapeOptions: {
                        formats: ['markdown'],
                        onlyMainContent: true,
                    },
                }),
                throw: false,
            });

            if (response.status !== 200) {
                console.error('[Doc Doctor] Firecrawl search error:', response.status, response.text);
                return [];
            }

            const data = response.json;

            // Map Firecrawl response to our WebSearchResult type
            return (data.data || []).map((result: {
                title?: string;
                url?: string;
                description?: string;
                markdown?: string;
            }) => ({
                title: result.title || 'Untitled',
                url: result.url || '',
                description: result.description || '',
                content: result.markdown,
            }));
        } catch (error) {
            console.error('[Doc Doctor] Firecrawl search error:', error);
            return [];
        }
    }

    /**
     * Scrape a single URL
     */
    async scrape(url: string): Promise<ScrapedContent | null> {
        if (!this.isConfigured() || !this.config.urlScrapingEnabled) {
            return null;
        }

        try {
            const response = await requestUrl({
                url: `${FIRECRAWL_API_BASE}/scrape`,
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${this.config.apiKey}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    url,
                    formats: ['markdown'],
                    onlyMainContent: true,
                }),
                throw: false,
            });

            if (response.status !== 200) {
                console.error('[Doc Doctor] Firecrawl scrape error:', response.status);
                return null;
            }

            const data = response.json;

            return {
                url,
                title: data.data?.metadata?.title || 'Untitled',
                markdown: data.data?.markdown || '',
                metadata: data.data?.metadata,
            };
        } catch (error) {
            console.error('[Doc Doctor] Firecrawl scrape error:', error);
            return null;
        }
    }

    /**
     * Scrape multiple URLs (with concurrency limit)
     */
    async scrapeUrls(urls: string[]): Promise<ScrapedContent[]> {
        if (!this.isConfigured() || !this.config.urlScrapingEnabled || urls.length === 0) {
            return [];
        }

        const urlsToScrape = urls.slice(0, this.config.maxUrlsToScrape);
        const results: ScrapedContent[] = [];

        // Scrape sequentially to avoid rate limits
        for (const url of urlsToScrape) {
            const content = await this.scrape(url);
            if (content) {
                results.push(content);
            }
        }

        return results;
    }

    /**
     * Extract URLs from document content
     */
    extractUrls(content: string): string[] {
        // Match URLs (http/https)
        const urlRegex = /https?:\/\/[^\s\)\]\}>"']+/g;
        const matches = content.match(urlRegex) || [];

        // Deduplicate and filter out common non-content URLs
        const excludePatterns = [
            /github\.com\/.*\/blob\//,  // GitHub blob links
            /localhost/,
            /127\.0\.0\.1/,
            /\.(jpg|jpeg|png|gif|svg|ico|webp|mp4|mp3|wav|pdf)$/i,
        ];

        const seen = new Set<string>();
        return matches.filter(url => {
            if (seen.has(url)) return false;
            seen.add(url);

            // Skip excluded patterns
            for (const pattern of excludePatterns) {
                if (pattern.test(url)) return false;
            }

            return true;
        });
    }
}

// =============================================================================
// SMART CONNECTIONS INTEGRATION
// =============================================================================

/**
 * Interface for Smart Connections plugin API
 */
interface SmartConnectionsAPI {
    find_connections: (options: {
        key: string;
        limit?: number;
    }) => Promise<Array<{
        key: string;
        score: number;
    }>>;
}

/**
 * Get related notes using Smart Connections plugin if available
 */
export async function getRelatedNotesFromSmartConnections(
    app: { plugins?: { plugins?: Record<string, { api?: SmartConnectionsAPI }> } },
    currentFilePath: string,
    maxResults: number,
): Promise<Array<{ path: string; title: string; similarity: number }>> {
    try {
        // Check if Smart Connections plugin is available
        const scPlugin = app.plugins?.plugins?.['smart-connections'];
        if (!scPlugin?.api?.find_connections) {
            console.log('[Doc Doctor] Smart Connections API not available');
            return [];
        }

        const results = await scPlugin.api.find_connections({
            key: currentFilePath,
            limit: maxResults,
        });

        return results.map(result => ({
            path: result.key,
            title: result.key.split('/').pop()?.replace('.md', '') || result.key,
            similarity: result.score,
        }));
    } catch (error) {
        console.error('[Doc Doctor] Smart Connections error:', error);
        return [];
    }
}

// =============================================================================
// CONTEXT GATHERING
// =============================================================================

/**
 * Gather external context for LLM analysis
 */
export async function gatherExternalContext(
    config: FirecrawlConfig,
    app: { plugins?: { plugins?: Record<string, { api?: SmartConnectionsAPI }> } },
    documentPath: string,
    documentContent: string,
    documentTitle: string,
    refinementScore: number,
): Promise<ExternalContext> {
    const context: ExternalContext = {
        webSearchResults: [],
        scrapedUrls: [],
        relatedNotes: [],
        errors: [],
    };

    if (!config.enabled) {
        return context;
    }

    const service = new FirecrawlService(config);

    // 1. Web search if refinement is below threshold
    if (config.webSearchEnabled && refinementScore < config.webSearchRefinementThreshold) {
        try {
            console.log(`[Doc Doctor] Performing web search (refinement ${refinementScore} < ${config.webSearchRefinementThreshold})`);
            const searchQuery = documentTitle || documentPath.split('/').pop()?.replace('.md', '') || '';
            if (searchQuery) {
                context.webSearchResults = await service.search(searchQuery);
                console.log(`[Doc Doctor] Found ${context.webSearchResults.length} web results`);
            }
        } catch (error) {
            context.errors.push(`Web search failed: ${(error as Error).message}`);
        }
    }

    // 2. Scrape URLs found in document
    if (config.urlScrapingEnabled) {
        try {
            const urls = service.extractUrls(documentContent);
            console.log(`[Doc Doctor] Found ${urls.length} URLs in document`);
            if (urls.length > 0) {
                context.scrapedUrls = await service.scrapeUrls(urls);
                console.log(`[Doc Doctor] Scraped ${context.scrapedUrls.length} URLs`);
            }
        } catch (error) {
            context.errors.push(`URL scraping failed: ${(error as Error).message}`);
        }
    }

    // 3. Get related notes from Smart Connections
    if (config.smartConnectionsEnabled) {
        try {
            context.relatedNotes = await getRelatedNotesFromSmartConnections(
                app,
                documentPath,
                config.maxRelatedNotes,
            );
            console.log(`[Doc Doctor] Found ${context.relatedNotes.length} related notes`);
        } catch (error) {
            context.errors.push(`Smart Connections failed: ${(error as Error).message}`);
        }
    }

    return context;
}

// =============================================================================
// CONTEXT FORMATTING
// =============================================================================

/**
 * Format external context for inclusion in LLM prompt
 */
export function formatExternalContext(context: ExternalContext): string {
    const sections: string[] = [];

    // Web search results
    if (context.webSearchResults.length > 0) {
        sections.push('## Web Search Results\n');
        for (const result of context.webSearchResults) {
            sections.push(`### ${result.title}`);
            sections.push(`URL: ${result.url}`);
            if (result.description) {
                sections.push(`Description: ${result.description}`);
            }
            if (result.content) {
                // Truncate content to avoid overwhelming the context
                const truncated = result.content.slice(0, 2000);
                sections.push(`Content:\n${truncated}${result.content.length > 2000 ? '\n...(truncated)' : ''}`);
            }
            sections.push('');
        }
    }

    // Scraped URLs
    if (context.scrapedUrls.length > 0) {
        sections.push('## Referenced URL Contents\n');
        for (const scraped of context.scrapedUrls) {
            sections.push(`### ${scraped.title}`);
            sections.push(`URL: ${scraped.url}`);
            // Truncate content
            const truncated = scraped.markdown.slice(0, 1500);
            sections.push(`Content:\n${truncated}${scraped.markdown.length > 1500 ? '\n...(truncated)' : ''}`);
            sections.push('');
        }
    }

    // Related notes
    if (context.relatedNotes.length > 0) {
        sections.push('## Related Notes in Vault\n');
        for (const note of context.relatedNotes) {
            sections.push(`- [[${note.path}]] (similarity: ${(note.similarity * 100).toFixed(0)}%)`);
        }
        sections.push('');
    }

    return sections.join('\n');
}

