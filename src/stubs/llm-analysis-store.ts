/**
 * LLM Analysis Store
 *
 * Manages state for LLM-powered stub suggestions.
 */

import { writable, derived, get } from 'svelte/store';
import { Notice } from 'obsidian';
import type LabeledAnnotations from '../main';
import type { SuggestedStub, FoundReference, LLMError, ExternalContext } from '../llm/llm-types';
import { syncState } from './stubs-store';
import { buildDocumentContext } from '../llm/llm-prompts';
import { getLLMService } from '../llm/llm-service';
import { gatherExternalContext, formatExternalContext } from '../llm/firecrawl-service';

// =============================================================================
// STATE TYPES
// =============================================================================

export interface LLMAnalysisState {
    /** Whether analysis is in progress */
    isAnalyzing: boolean;

    /** Current document path being analyzed */
    currentPath: string | null;

    /** Current streaming text (updates in real-time during analysis) */
    streamingText: string;

    /** LLM's thinking/reasoning process (final) */
    thinking: string | null;

    /** Analysis summary from LLM */
    summary: string | null;

    /** Suggested stubs from LLM */
    suggestions: SuggestedStub[];

    /** References found during analysis */
    references: FoundReference[];

    /** Confidence score (0-1) */
    confidence: number;

    /** Error from last analysis */
    error: LLMError | null;

    /** Timestamp of last analysis */
    lastAnalysisTime: number | null;

    /** Active tab in the panel */
    activeTab: 'suggestions' | 'references';

    /** Whether streaming text is expanded */
    streamingExpanded: boolean;
}

// =============================================================================
// INITIAL STATE
// =============================================================================

const initialState: LLMAnalysisState = {
    isAnalyzing: false,
    currentPath: null,
    streamingText: '',
    thinking: null,
    summary: null,
    suggestions: [],
    references: [],
    confidence: 0,
    error: null,
    lastAnalysisTime: null,
    activeTab: 'suggestions',
    streamingExpanded: false,
};

// =============================================================================
// STORE
// =============================================================================

export const llmAnalysisState = writable<LLMAnalysisState>(initialState);

// Derived stores for convenience
export const isAnalyzing = derived(llmAnalysisState, ($state) => $state.isAnalyzing);
export const streamingText = derived(llmAnalysisState, ($state) => $state.streamingText);
export const suggestions = derived(llmAnalysisState, ($state) => $state.suggestions);
export const suggestionCount = derived(llmAnalysisState, ($state) => $state.suggestions.length);
export const references = derived(llmAnalysisState, ($state) => $state.references);
export const referenceCount = derived(llmAnalysisState, ($state) => $state.references.length);
export const analysisError = derived(llmAnalysisState, ($state) => $state.error);
export const activeTab = derived(llmAnalysisState, ($state) => $state.activeTab);

// =============================================================================
// ACTIONS
// =============================================================================

/**
 * Start analysis
 */
function startAnalysis(path: string): void {
    llmAnalysisState.update((state) => ({
        ...state,
        isAnalyzing: true,
        currentPath: path,
        streamingText: '',
        error: null,
    }));
}

/**
 * Update streaming text (called during analysis)
 */
export function updateStreamingText(text: string): void {
    llmAnalysisState.update((state) => ({
        ...state,
        streamingText: text,
    }));
}

/**
 * Append to streaming text
 */
export function appendStreamingText(chunk: string): void {
    llmAnalysisState.update((state) => ({
        ...state,
        streamingText: state.streamingText + chunk,
    }));
}

/**
 * Toggle streaming expanded state
 */
export function toggleStreamingExpanded(): void {
    llmAnalysisState.update((state) => ({
        ...state,
        streamingExpanded: !state.streamingExpanded,
    }));
}

/**
 * Set analysis results
 */
function setResults(
    thinking: string | undefined,
    summary: string,
    suggestions: SuggestedStub[],
    references: FoundReference[],
    confidence: number,
): void {
    llmAnalysisState.update((state) => ({
        ...state,
        isAnalyzing: false,
        thinking: thinking || null,
        summary,
        suggestions,
        references,
        confidence,
        lastAnalysisTime: Date.now(),
    }));
}

/**
 * Set analysis error
 */
function setError(error: LLMError): void {
    llmAnalysisState.update((state) => ({
        ...state,
        isAnalyzing: false,
        error,
    }));
}

/**
 * Clear all results
 */
export function clearSuggestions(): void {
    llmAnalysisState.set(initialState);
}

/**
 * Remove a suggestion by index
 */
export function removeSuggestion(index: number): void {
    llmAnalysisState.update((state) => ({
        ...state,
        suggestions: state.suggestions.filter((_, i) => i !== index),
    }));
}

/**
 * Remove a reference by index
 */
export function removeReference(index: number): void {
    llmAnalysisState.update((state) => ({
        ...state,
        references: state.references.filter((_, i) => i !== index),
    }));
}

/**
 * Set active tab
 */
export function setActiveTab(tab: 'suggestions' | 'references'): void {
    llmAnalysisState.update((state) => ({
        ...state,
        activeTab: tab,
    }));
}

// =============================================================================
// MAIN TRIGGER FUNCTION
// =============================================================================

/**
 * Log available tools and integrations
 */
function logToolAvailability(plugin: LabeledAnnotations): void {
    const settings = plugin.settings.getValue();
    const firecrawlConfig = settings.llm.firecrawl;

    console.group('[Doc Doctor] Tool Availability Check');

    // Vault access
    console.log('Vault Access: ✓ Available');
    console.log(`  - Files: ${plugin.app.vault.getMarkdownFiles().length} markdown files`);

    // Metadata cache
    console.log('Metadata Cache: ✓ Available');

    // Check for Smart Connections plugin
    const smartConnections = (plugin.app as unknown as { plugins?: { plugins?: Record<string, unknown> } }).plugins?.plugins?.['smart-connections'];
    if (smartConnections) {
        console.log('Smart Connections: ✓ Available');
    } else {
        console.log('Smart Connections: ✗ Not installed');
    }

    // Check for Copilot plugin
    const copilot = (plugin.app as unknown as { plugins?: { plugins?: Record<string, unknown> } }).plugins?.plugins?.['obsidian-copilot'];
    if (copilot) {
        console.log('Obsidian Copilot: ✓ Available');
    } else {
        console.log('Obsidian Copilot: ✗ Not installed');
    }

    // Firecrawl status
    if (firecrawlConfig?.enabled && firecrawlConfig.apiKey) {
        console.log('Firecrawl: ✓ Configured');
        console.log(`  - Web Search: ${firecrawlConfig.webSearchEnabled ? '✓' : '✗'}`);
        console.log(`  - URL Scraping: ${firecrawlConfig.urlScrapingEnabled ? '✓' : '✗'}`);
        console.log(`  - Refinement Threshold: ${firecrawlConfig.webSearchRefinementThreshold}`);
    } else if (firecrawlConfig?.enabled) {
        console.log('Firecrawl: ⚠ Enabled but no API key');
    } else {
        console.log('Firecrawl: ✗ Not enabled');
    }

    // Check for linked files via metadata
    console.log('Link Resolution: ✓ Available via metadataCache');

    console.groupEnd();
}

/**
 * Trigger LLM analysis for the active file
 */
export async function triggerLLMAnalysis(plugin: LabeledAnnotations): Promise<void> {
    const activeFile = plugin.app.workspace.getActiveFile();
    if (!activeFile) {
        new Notice('No active file to analyze');
        return;
    }

    // Check if markdown
    if (activeFile.extension !== 'md') {
        new Notice('LLM analysis only works on markdown files');
        return;
    }

    // Log tool availability (debug)
    const settings = plugin.settings.getValue();
    if (settings.llm.debug.enabled) {
        logToolAvailability(plugin);
    }

    startAnalysis(activeFile.path);

    try {
        // Get file content
        const content = await plugin.app.vault.read(activeFile);

        // Parse frontmatter
        const frontmatter = plugin.app.metadataCache.getFileCache(activeFile)?.frontmatter || {};

        // Get existing stubs
        const currentSync = get(syncState);
        const existingStubs = currentSync.stubs;

        // Get document title and refinement score for context gathering
        const documentTitle = frontmatter.title || activeFile.basename;
        const refinementScore = typeof frontmatter.refinement === 'number' ? frontmatter.refinement : 0;

        // Gather external context (Firecrawl, Smart Connections)
        let externalContext: ExternalContext | null = null;
        if (settings.llm.firecrawl?.enabled) {
            updateStreamingText('Gathering external context...');
            try {
                externalContext = await gatherExternalContext(
                    settings.llm.firecrawl,
                    plugin.app as never,
                    activeFile.path,
                    content,
                    documentTitle,
                    refinementScore,
                );

                if (settings.llm.debug.enabled) {
                    console.log('[Doc Doctor] External context gathered:', {
                        webResults: externalContext.webSearchResults.length,
                        scrapedUrls: externalContext.scrapedUrls.length,
                        relatedNotes: externalContext.relatedNotes.length,
                        errors: externalContext.errors,
                    });
                }
            } catch (error) {
                console.error('[Doc Doctor] Failed to gather external context:', error);
            }
        }

        // Format external context for inclusion in prompt
        const externalContextText = externalContext ? formatExternalContext(externalContext) : '';

        // Build document context
        const context = buildDocumentContext(
            activeFile.path,
            content,
            frontmatter,
            existingStubs,
            externalContextText, // Pass external context
        );

        // Get LLM service
        const service = getLLMService(settings.llm, settings.stubs);

        // Use streaming if enabled
        let response;
        if (settings.llm.streaming) {
            response = await service.analyzeDocumentStreaming(context, (_chunk, fullText) => {
                // Update streaming text in real-time
                updateStreamingText(fullText);
            });
        } else {
            response = await service.analyzeDocument(context);
        }

        // Update state with results (response may have thinking field from validation)
        const responseWithThinking = response as typeof response & { thinking?: string };
        setResults(
            responseWithThinking.thinking,
            response.analysis_summary,
            response.suggested_stubs,
            response.references,
            response.confidence,
        );

        // Show notice
        const stubCount = response.suggested_stubs.length;
        const refCount = response.references.length;
        const parts: string[] = [];
        if (stubCount > 0) parts.push(`${stubCount} stub${stubCount !== 1 ? 's' : ''}`);
        if (refCount > 0) parts.push(`${refCount} reference${refCount !== 1 ? 's' : ''}`);
        new Notice(`Analysis complete: ${parts.join(', ') || 'no suggestions'}`);

    } catch (error) {
        const llmError = error as LLMError;
        setError(llmError);

        // Show error notice
        new Notice(`Analysis failed: ${llmError.message}`, 5000);

        console.error('[Doc Doctor] LLM analysis error:', llmError);
    }
}
