/**
 * Fallback Search Implementation
 *
 * Provides keyword-based search when Smart Connections is unavailable.
 * Uses TF-IDF-like scoring and Jaccard similarity for basic relevance.
 */

import { App, TFile, CachedMetadata } from 'obsidian';
import type { RelatedNote } from './types';

// =============================================================================
// KEYWORD EXTRACTION
// =============================================================================

/**
 * Common stop words to filter out
 */
const STOP_WORDS = new Set([
    'a', 'an', 'and', 'are', 'as', 'at', 'be', 'by', 'for', 'from',
    'has', 'he', 'in', 'is', 'it', 'its', 'of', 'on', 'or', 'that',
    'the', 'to', 'was', 'were', 'will', 'with', 'this', 'they', 'but',
    'have', 'not', 'been', 'their', 'would', 'which', 'can', 'there',
    'more', 'when', 'about', 'into', 'than', 'some', 'what', 'these',
    'could', 'other', 'also', 'just', 'very', 'only', 'then', 'such',
    'like', 'over', 'even', 'most', 'made', 'after', 'many', 'before',
]);

/**
 * Extract keywords from text
 */
function extractKeywords(text: string): string[] {
    // Normalize and tokenize
    const words = text
        .toLowerCase()
        .replace(/[^\w\s]/g, ' ')
        .split(/\s+/)
        .filter((word) => word.length > 2 && !STOP_WORDS.has(word));

    // Remove duplicates while preserving order
    return [...new Set(words)];
}

/**
 * Calculate word frequencies for TF scoring
 */
function calculateWordFrequencies(keywords: string[]): Map<string, number> {
    const freq = new Map<string, number>();
    for (const word of keywords) {
        freq.set(word, (freq.get(word) || 0) + 1);
    }
    return freq;
}

// =============================================================================
// SIMILARITY CALCULATIONS
// =============================================================================

/**
 * Calculate Jaccard similarity between two keyword sets
 */
function jaccardSimilarity(setA: Set<string>, setB: Set<string>): number {
    if (setA.size === 0 && setB.size === 0) return 0;

    const intersection = new Set([...setA].filter((x) => setB.has(x)));
    const union = new Set([...setA, ...setB]);

    return intersection.size / union.size;
}

/**
 * Calculate TF-IDF-like score between two documents
 */
function calculateRelevanceScore(
    sourceKeywords: string[],
    targetKeywords: string[],
    targetFreq: Map<string, number>,
): number {
    const sourceSet = new Set(sourceKeywords);
    const targetSet = new Set(targetKeywords);

    // Base Jaccard similarity
    const jaccard = jaccardSimilarity(sourceSet, targetSet);

    // Weighted overlap bonus (more matches = higher score)
    let overlapBonus = 0;
    let totalWeight = 0;

    for (const word of sourceKeywords) {
        if (targetSet.has(word)) {
            // Weight by frequency in target
            const freq = targetFreq.get(word) || 1;
            overlapBonus += Math.log(1 + freq);
            totalWeight += Math.log(1 + freq);
        }
    }

    const normalizedOverlap = totalWeight > 0 ? overlapBonus / (totalWeight + 1) : 0;

    // Combined score (weighted average)
    return jaccard * 0.6 + normalizedOverlap * 0.4;
}

// =============================================================================
// FALLBACK SEARCH SERVICE
// =============================================================================

/**
 * Fallback search using keyword matching
 *
 * Used when Smart Connections plugin is not available.
 * Provides basic relevance search using TF-IDF-like scoring.
 */
export class FallbackSearchService {
    private keywordCache = new Map<string, { keywords: string[]; freq: Map<string, number> }>();
    private cacheExpiry = 60000; // 1 minute
    private lastCacheClear = Date.now();

    constructor(private app: App) {}

    // =========================================================================
    // CACHE MANAGEMENT
    // =========================================================================

    /**
     * Clear stale cache entries
     */
    private clearStaleCache(): void {
        const now = Date.now();
        if (now - this.lastCacheClear > this.cacheExpiry) {
            this.keywordCache.clear();
            this.lastCacheClear = now;
        }
    }

    /**
     * Get cached keywords for a file, or extract and cache them
     */
    private async getKeywordsForFile(
        file: TFile,
    ): Promise<{ keywords: string[]; freq: Map<string, number> }> {
        this.clearStaleCache();

        // Check cache
        const cached = this.keywordCache.get(file.path);
        if (cached) {
            return cached;
        }

        // Extract keywords from file content
        let text = '';

        // Include title
        text += file.basename + ' ';

        // Include frontmatter fields
        const cache = this.app.metadataCache.getFileCache(file);
        if (cache?.frontmatter) {
            const fm = cache.frontmatter;
            if (fm.title) text += fm.title + ' ';
            if (fm.description) text += fm.description + ' ';
            if (Array.isArray(fm.tags)) text += fm.tags.join(' ') + ' ';
            if (Array.isArray(fm.aliases)) text += fm.aliases.join(' ') + ' ';
        }

        // Include headings
        if (cache?.headings) {
            for (const heading of cache.headings) {
                text += heading.heading + ' ';
            }
        }

        // Include links (link text often indicates content)
        if (cache?.links) {
            for (const link of cache.links) {
                text += link.displayText || link.link + ' ';
            }
        }

        // Try to get a content sample (first 2000 chars to avoid large files)
        try {
            const content = await this.app.vault.cachedRead(file);
            // Skip frontmatter
            const contentStart = content.indexOf('---', 4);
            const bodyContent =
                contentStart > 0 ? content.slice(contentStart + 3, contentStart + 2003) : content.slice(0, 2000);
            text += bodyContent;
        } catch {
            // Ignore read errors
        }

        const keywords = extractKeywords(text);
        const freq = calculateWordFrequencies(keywords);

        const result = { keywords, freq };
        this.keywordCache.set(file.path, result);

        return result;
    }

    // =========================================================================
    // SEARCH METHODS
    // =========================================================================

    /**
     * Find notes related to a given note using keyword matching
     *
     * @param notePath - Path to the source note
     * @param limit - Maximum number of results
     * @returns Array of related notes with similarity scores
     */
    async findRelated(notePath: string, limit = 10): Promise<RelatedNote[]> {
        const sourceFile = this.app.vault.getAbstractFileByPath(notePath);
        if (!sourceFile || !(sourceFile instanceof TFile)) {
            return [];
        }

        // Get source keywords
        const { keywords: sourceKeywords } = await this.getKeywordsForFile(sourceFile);
        if (sourceKeywords.length === 0) {
            return [];
        }

        // Score all markdown files
        const scores: Array<{ file: TFile; score: number; matchedKeywords: string[] }> = [];
        const markdownFiles = this.app.vault.getMarkdownFiles();

        for (const file of markdownFiles) {
            // Skip the source file
            if (file.path === notePath) continue;

            const { keywords: targetKeywords, freq: targetFreq } = await this.getKeywordsForFile(file);
            if (targetKeywords.length === 0) continue;

            const score = calculateRelevanceScore(sourceKeywords, targetKeywords, targetFreq);

            // Only include if there's meaningful similarity
            if (score > 0.05) {
                const sourceSet = new Set(sourceKeywords);
                const matchedKeywords = targetKeywords.filter((k) => sourceSet.has(k));
                scores.push({ file, score, matchedKeywords });
            }
        }

        // Sort by score and take top results
        scores.sort((a, b) => b.score - a.score);
        const topResults = scores.slice(0, limit);

        // Convert to RelatedNote format
        return topResults.map(({ file, score, matchedKeywords }) => ({
            path: file.path,
            title: file.basename,
            similarity: Math.min(score, 1), // Cap at 1.0
            excerpt: this.getExcerpt(file),
            matchedKeywords: matchedKeywords.slice(0, 5), // Top 5 matched keywords
            method: 'keyword' as const,
        }));
    }

    /**
     * Search vault by query string using keyword matching
     *
     * @param query - Search query
     * @param limit - Maximum number of results
     * @returns Array of matching notes with similarity scores
     */
    async search(query: string, limit = 10): Promise<RelatedNote[]> {
        const queryKeywords = extractKeywords(query);
        if (queryKeywords.length === 0) {
            return [];
        }

        const querySet = new Set(queryKeywords);

        // Score all markdown files
        const scores: Array<{ file: TFile; score: number; matchedKeywords: string[] }> = [];
        const markdownFiles = this.app.vault.getMarkdownFiles();

        for (const file of markdownFiles) {
            const { keywords: fileKeywords, freq: fileFreq } = await this.getKeywordsForFile(file);
            if (fileKeywords.length === 0) continue;

            const score = calculateRelevanceScore(queryKeywords, fileKeywords, fileFreq);

            if (score > 0.02) {
                const matchedKeywords = fileKeywords.filter((k) => querySet.has(k));
                scores.push({ file, score, matchedKeywords });
            }
        }

        // Sort by score and take top results
        scores.sort((a, b) => b.score - a.score);
        const topResults = scores.slice(0, limit);

        return topResults.map(({ file, score, matchedKeywords }) => ({
            path: file.path,
            title: file.basename,
            similarity: Math.min(score * 2, 1), // Scale up for query search
            excerpt: this.getExcerpt(file),
            matchedKeywords: matchedKeywords.slice(0, 5),
            method: 'keyword' as const,
        }));
    }

    // =========================================================================
    // HELPERS
    // =========================================================================

    /**
     * Get a brief excerpt from a file for display
     */
    private getExcerpt(file: TFile): string {
        const cache = this.app.metadataCache.getFileCache(file);

        // Try frontmatter description
        if (cache?.frontmatter?.description) {
            return String(cache.frontmatter.description).slice(0, 150);
        }

        // Try first heading
        if (cache?.headings?.[0]) {
            return cache.headings[0].heading;
        }

        // Fallback to file path
        return file.path;
    }
}
