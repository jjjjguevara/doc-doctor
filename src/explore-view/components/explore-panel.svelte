<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { ChevronDown, ChevronRight, RefreshCw, Search, AlertTriangle, Link2, FileText, Compass } from 'lucide-svelte';
    import LabeledAnnotations from '../../main';
    import {
        activeNotePath,
        activeNoteTitle,
        searchQuery,
        results,
        isLoading,
        errorMessage,
        duplicates,
        linkSuggestions,
        stubContext,
        selectedStubForContext,
        expandedSections,
        isSearchMode,
        hasResults,
        hasDuplicates,
        hasLinkSuggestions,
        hasStubContext,
        setActiveNote,
        setSearchQuery,
        setResults,
        setLoading,
        setError,
        clearError,
        setDuplicates,
        setStubContext,
        toggleSection,
        resetExploreState,
    } from '../explore-store';
    import RelatedNoteItem from './related-note-item.svelte';
    import SemanticSearch from './semantic-search.svelte';

    export let plugin: LabeledAnnotations;

    let debounceTimer: ReturnType<typeof setTimeout> | null = null;

    // Subscribe to active file changes
    $: if (plugin.app.workspace) {
        const activeFile = plugin.app.workspace.getActiveFile();
        if (activeFile && activeFile.path !== $activeNotePath) {
            setActiveNote(activeFile.path, activeFile.basename);
            refreshRelatedNotes();
        }
    }

    onMount(() => {
        // Initial load
        const activeFile = plugin.app.workspace.getActiveFile();
        if (activeFile) {
            setActiveNote(activeFile.path, activeFile.basename);
            refreshRelatedNotes();
        }

        // Listen for active file changes
        const unregister = plugin.app.workspace.on('active-leaf-change', () => {
            const file = plugin.app.workspace.getActiveFile();
            if (file && file.path !== $activeNotePath) {
                setActiveNote(file.path, file.basename);
                if (!$isSearchMode) {
                    refreshRelatedNotes();
                }
            }
        });

        return () => {
            plugin.app.workspace.offref(unregister);
        };
    });

    onDestroy(() => {
        if (debounceTimer) {
            clearTimeout(debounceTimer);
        }
    });

    async function refreshRelatedNotes() {
        if (!$activeNotePath || !plugin.smartConnectionsService) {
            setResults([]);
            return;
        }

        setLoading(true);
        clearError();

        try {
            const related = await plugin.smartConnectionsService.findRelated($activeNotePath);
            setResults(related);

            // Also check for duplicates
            const dups = await plugin.smartConnectionsService.detectDuplicates($activeNotePath);
            setDuplicates(dups);
        } catch (error) {
            console.error('[Explore] Error fetching related notes:', error);
            setError('Failed to fetch related notes');
        } finally {
            setLoading(false);
        }
    }

    async function handleSearch(query: string) {
        if (!query.trim()) {
            // Clear search, show related notes
            setSearchQuery('');
            refreshRelatedNotes();
            return;
        }

        setSearchQuery(query);
        setLoading(true);
        clearError();

        try {
            const searchResults = await plugin.smartConnectionsService?.search(query) ?? [];
            setResults(searchResults);
        } catch (error) {
            console.error('[Explore] Search error:', error);
            setError('Search failed');
        } finally {
            setLoading(false);
        }
    }

    function handleNoteClick(path: string) {
        plugin.app.workspace.openLinkText(path, '', false);
    }

    function handleNoteNewPane(path: string) {
        plugin.app.workspace.openLinkText(path, '', 'split');
    }

    async function handleAddToRelated(path: string) {
        if (!$activeNotePath || !plugin.smartConnectionsService) return;

        try {
            const title = path.split('/').pop()?.replace('.md', '') ?? path;
            const wikilink = `[[${title}]]`;
            await plugin.smartConnectionsService.addToRelatedProperty($activeNotePath, [wikilink]);
            // Could show a notice here
        } catch (error) {
            console.error('[Explore] Error adding to related:', error);
        }
    }

    function formatSimilarity(score: number): string {
        return `${Math.round(score * 100)}%`;
    }
</script>

<div class="explore-panel">
    <!-- Search Section -->
    <div class="explore-search-section">
        <SemanticSearch
            query={$searchQuery}
            isLoading={$isLoading}
            on:search={(e) => handleSearch(e.detail)}
            on:clear={() => handleSearch('')}
        />
    </div>

    <!-- Current Note Context -->
    {#if $activeNoteTitle && !$isSearchMode}
        <div class="current-note-context">
            <div class="context-header">
                <FileText size={14} />
                <span class="context-title">{$activeNoteTitle}</span>
                <button
                    class="refresh-btn"
                    on:click={refreshRelatedNotes}
                    title="Refresh related notes"
                    disabled={$isLoading}
                >
                    <span class:spinning={$isLoading}><RefreshCw size={12} /></span>
                </button>
            </div>
        </div>
    {/if}

    <!-- Error Message -->
    {#if $errorMessage}
        <div class="error-message">
            <AlertTriangle size={14} />
            <span>{$errorMessage}</span>
        </div>
    {/if}

    <!-- Scrollable Content -->
    <div class="explore-content">
        <!-- Related Notes / Search Results Section -->
        <div class="section">
            <button
                class="section-header"
                on:click={() => toggleSection('relatedNotes')}
            >
                {#if $expandedSections.relatedNotes}
                    <ChevronDown size={14} />
                {:else}
                    <ChevronRight size={14} />
                {/if}
                <span class="section-title">
                    {$isSearchMode ? 'Search Results' : 'Related Notes'}
                </span>
                <span class="section-count">{$results.length}</span>
            </button>

            {#if $expandedSections.relatedNotes}
                <div class="section-content">
                    {#if $isLoading}
                        <div class="loading-state">
                            <span class="spinning"><RefreshCw size={16} /></span>
                            <span>Searching...</span>
                        </div>
                    {:else if !$hasResults}
                        <div class="empty-state">
                            <Compass size={24} />
                            <span>
                                {$isSearchMode
                                    ? 'No results found'
                                    : 'No related notes found'}
                            </span>
                        </div>
                    {:else}
                        <div class="results-list">
                            {#each $results as note (note.path)}
                                <RelatedNoteItem
                                    {note}
                                    on:click={() => handleNoteClick(note.path)}
                                    on:newpane={() => handleNoteNewPane(note.path)}
                                    on:addrelated={() => handleAddToRelated(note.path)}
                                />
                            {/each}
                        </div>
                    {/if}
                </div>
            {/if}
        </div>

        <!-- Stub Context Section (when a stub is selected) -->
        {#if $hasStubContext && $stubContext}
            <div class="section">
                <button
                    class="section-header"
                    on:click={() => toggleSection('stubContext')}
                >
                    {#if $expandedSections.stubContext}
                        <ChevronDown size={14} />
                    {:else}
                        <ChevronRight size={14} />
                    {/if}
                    <span class="section-title">Stub Context</span>
                </button>

                {#if $expandedSections.stubContext}
                    <div class="section-content stub-context-content">
                        <div class="stub-info">
                            <span class="stub-type">{$stubContext.stub.type}</span>
                            <span class="stub-desc">{$stubContext.stub.description}</span>
                        </div>
                        {#if $stubContext.suggestedLinks.length > 0}
                            <div class="suggested-links">
                                <span class="suggested-label">Could resolve with:</span>
                                {#each $stubContext.suggestedLinks as link}
                                    <button class="suggested-link" on:click={() => handleNoteClick(link)}>
                                        {link}
                                    </button>
                                {/each}
                            </div>
                        {/if}
                    </div>
                {/if}
            </div>
        {/if}

        <!-- Potential Duplicates Section -->
        {#if $hasDuplicates}
            <div class="section warning-section">
                <button
                    class="section-header"
                    on:click={() => toggleSection('duplicates')}
                >
                    {#if $expandedSections.duplicates}
                        <ChevronDown size={14} />
                    {:else}
                        <ChevronRight size={14} />
                    {/if}
                    <span class="warning-icon"><AlertTriangle size={14} /></span>
                    <span class="section-title">Potential Duplicates</span>
                    <span class="section-count">{$duplicates.length}</span>
                </button>

                {#if $expandedSections.duplicates}
                    <div class="section-content">
                        {#each $duplicates as dup (dup.path)}
                            <button
                                class="duplicate-item"
                                on:click={() => handleNoteClick(dup.path)}
                            >
                                <span class="dup-title">{dup.title}</span>
                                <span class="dup-score">{formatSimilarity(dup.similarity)}</span>
                            </button>
                        {/each}
                    </div>
                {/if}
            </div>
        {/if}

        <!-- Link Suggestions Section -->
        {#if $hasLinkSuggestions}
            <div class="section">
                <button
                    class="section-header"
                    on:click={() => toggleSection('linkSuggestions')}
                >
                    {#if $expandedSections.linkSuggestions}
                        <ChevronDown size={14} />
                    {:else}
                        <ChevronRight size={14} />
                    {/if}
                    <Link2 size={14} />
                    <span class="section-title">Link Suggestions</span>
                    <span class="section-count">{$linkSuggestions.length}</span>
                </button>

                {#if $expandedSections.linkSuggestions}
                    <div class="section-content">
                        {#each $linkSuggestions as suggestion (suggestion.line + suggestion.text)}
                            <div class="link-suggestion">
                                <span class="suggestion-line">Line {suggestion.line}:</span>
                                <span class="suggestion-text">"{suggestion.text}"</span>
                                <span class="suggestion-arrow">→</span>
                                <button
                                    class="suggestion-target"
                                    on:click={() => handleNoteClick(suggestion.targetPath)}
                                >
                                    [[{suggestion.targetTitle}]]
                                </button>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        {/if}
    </div>
</div>

<style>
    .explore-panel {
        display: flex;
        flex-direction: column;
        height: 100%;
        gap: 8px;
    }

    .explore-search-section {
        flex-shrink: 0;
    }

    .current-note-context {
        flex-shrink: 0;
        padding: 6px 8px;
        background: var(--background-secondary);
        border-radius: 6px;
    }

    .context-header {
        display: flex;
        align-items: center;
        gap: 6px;
        color: var(--text-muted);
        font-size: var(--font-ui-smaller);
    }

    .context-title {
        flex: 1;
        font-weight: 500;
        color: var(--text-normal);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .refresh-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 2px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        border-radius: 4px;
    }

    .refresh-btn:hover {
        background: var(--background-modifier-hover);
        color: var(--text-normal);
    }

    .refresh-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .error-message {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 8px;
        background: var(--background-modifier-error);
        color: var(--text-error);
        border-radius: 6px;
        font-size: var(--font-ui-smaller);
    }

    .explore-content {
        flex: 1;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .section {
        border-radius: 6px;
        overflow: hidden;
    }

    .section-header {
        display: flex;
        align-items: center;
        gap: 6px;
        width: 100%;
        padding: 8px;
        border: none;
        background: var(--background-secondary);
        color: var(--text-normal);
        cursor: pointer;
        text-align: left;
        font-size: var(--font-ui-small);
    }

    .section-header:hover {
        background: var(--background-modifier-hover);
    }

    .section-title {
        flex: 1;
        font-weight: 500;
    }

    .section-count {
        font-size: var(--font-ui-smaller);
        color: var(--text-muted);
        background: var(--background-modifier-border);
        padding: 1px 6px;
        border-radius: 10px;
    }

    .section-content {
        padding: 8px;
        background: var(--background-primary-alt);
    }

    .warning-section .section-header {
        background: rgba(255, 165, 0, 0.1);
    }

    .warning-icon {
        color: var(--text-warning);
    }

    .loading-state,
    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 8px;
        padding: 24px;
        color: var(--text-muted);
        font-size: var(--font-ui-smaller);
    }

    .results-list {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .stub-context-content {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .stub-info {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .stub-type {
        font-size: var(--font-ui-smaller);
        color: var(--text-accent);
        font-weight: 500;
    }

    .stub-desc {
        font-size: var(--font-ui-small);
        color: var(--text-normal);
    }

    .suggested-links {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .suggested-label {
        font-size: var(--font-ui-smaller);
        color: var(--text-muted);
    }

    .suggested-link {
        padding: 4px 8px;
        border: none;
        background: var(--background-modifier-hover);
        color: var(--text-accent);
        cursor: pointer;
        border-radius: 4px;
        text-align: left;
        font-size: var(--font-ui-smaller);
    }

    .suggested-link:hover {
        background: var(--interactive-accent);
        color: var(--text-on-accent);
    }

    .duplicate-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        padding: 6px 8px;
        border: none;
        background: transparent;
        cursor: pointer;
        border-radius: 4px;
        text-align: left;
    }

    .duplicate-item:hover {
        background: var(--background-modifier-hover);
    }

    .dup-title {
        flex: 1;
        font-size: var(--font-ui-smaller);
        color: var(--text-normal);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .dup-score {
        font-size: var(--font-ui-smaller);
        color: var(--text-warning);
        font-weight: 500;
    }

    .link-suggestion {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 4px 0;
        font-size: var(--font-ui-smaller);
        flex-wrap: wrap;
    }

    .suggestion-line {
        color: var(--text-faint);
    }

    .suggestion-text {
        color: var(--text-muted);
    }

    .suggestion-arrow {
        color: var(--text-faint);
    }

    .suggestion-target {
        padding: 2px 6px;
        border: none;
        background: var(--background-modifier-hover);
        color: var(--text-accent);
        cursor: pointer;
        border-radius: 4px;
        font-size: var(--font-ui-smaller);
    }

    .suggestion-target:hover {
        background: var(--interactive-accent);
        color: var(--text-on-accent);
    }

    :global(.spinning) {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }
</style>
