<script lang="ts">
    import { Notice } from 'obsidian';
    import { onMount, onDestroy, tick } from 'svelte';
    import { writable } from 'svelte/store';
    import { Check, X, AlertCircle, ChevronDown, ChevronRight, Sparkles, Link, ExternalLink, BookOpen, Brain, Loader2, GripVertical } from 'lucide-svelte';
    import { llmAnalysisState, clearSuggestions, removeSuggestion, removeReference, setActiveTab, toggleStreamingExpanded } from '../../../../stubs/llm-analysis-store';
    import { stubsConfig } from '../../../../stubs/stubs-store';
    import { generateAnchorId, insertAnchorAtLine, getValidAnchors } from '../../../../stubs/helpers/anchor-utils';
    import type LabeledAnnotations from '../../../../main';
    import type { SuggestedStub, FoundReference, LLMConfiguration } from '../../../../llm/llm-types';

    export let plugin: LabeledAnnotations;

    // Local state
    let expandedSuggestions: Set<number> = new Set();
    let expandedReferences: Set<number> = new Set();
    let panelHeight = 350; // Default height
    let isResizing = false;
    let streamingScrollRef: HTMLDivElement | null = null;

    // Settings subscription
    const llmSettingsStore = writable<LLMConfiguration | null>(null);
    let unsubscribe: (() => void) | null = null;

    onMount(() => {
        unsubscribe = plugin.settings.subscribe((settings) => {
            llmSettingsStore.set(settings.llm);
        });
    });

    onDestroy(() => {
        if (unsubscribe) unsubscribe();
    });

    $: llmSettings = $llmSettingsStore;
    $: insertionOrder = llmSettings?.insertionOrder || 'bottom';
    $: separateReferenceProperties = llmSettings?.separateReferenceProperties || false;
    $: vaultReferenceProperty = llmSettings?.vaultReferenceProperty || 'references';
    $: webReferenceProperty = llmSettings?.webReferenceProperty || 'references';
    $: config = $stubsConfig;
    $: state = $llmAnalysisState;
    $: isAnalyzing = state.isAnalyzing;
    $: streamingText = state.streamingText;
    $: streamingExpanded = state.streamingExpanded;
    $: suggestions = state.suggestions;
    $: references = state.references;
    $: activeTab = state.activeTab;
    $: hasSuggestions = suggestions.length > 0;
    $: hasReferences = references.length > 0;
    $: hasContent = hasSuggestions || hasReferences;
    $: hasError = state.error !== null;
    $: hasThinking = state.thinking !== null && state.thinking.length > 0;
    $: showPanel = isAnalyzing || hasContent || hasError;

    // Auto-scroll streaming text only during active analysis
    $: if (isAnalyzing && streamingText && streamingScrollRef) {
        // Use requestAnimationFrame to avoid blocking
        requestAnimationFrame(() => {
            if (streamingScrollRef) {
                streamingScrollRef.scrollTop = streamingScrollRef.scrollHeight;
            }
        });
    }

    // Clear expanded states when arrays shrink to prevent stale indices
    let prevSuggestionsLength = 0;
    let prevReferencesLength = 0;
    $: {
        if (suggestions.length < prevSuggestionsLength) {
            expandedSuggestions = new Set();
        }
        prevSuggestionsLength = suggestions.length;
    }
    $: {
        if (references.length < prevReferencesLength) {
            expandedReferences = new Set();
        }
        prevReferencesLength = references.length;
    }

    // Get stub type info
    function getStubTypeInfo(typeKey: string) {
        if (!config) return { displayName: typeKey, color: '#888' };
        const type = Object.values(config.stubTypes).find(t => t.key === typeKey);
        return type || { displayName: typeKey, color: '#888' };
    }

    // Get reference icon
    function getReferenceIcon(type: FoundReference['type']) {
        switch (type) {
            case 'vault': return Link;
            case 'web': return ExternalLink;
            case 'citation': return BookOpen;
            default: return Link;
        }
    }

    // Get the right frontmatter property for a reference based on settings
    function getReferenceProperty(refType: FoundReference['type']): string {
        if (!separateReferenceProperties) {
            return vaultReferenceProperty;
        }
        // Separate properties: vault/citation go to vault property, web goes to web property
        if (refType === 'web') {
            return webReferenceProperty;
        }
        return vaultReferenceProperty;
    }

    // Toggle expansion
    function toggleSuggestionExpanded(index: number) {
        if (expandedSuggestions.has(index)) {
            expandedSuggestions.delete(index);
        } else {
            expandedSuggestions.add(index);
        }
        expandedSuggestions = expandedSuggestions;
    }

    function toggleReferenceExpanded(index: number) {
        if (expandedReferences.has(index)) {
            expandedReferences.delete(index);
        } else {
            expandedReferences.add(index);
        }
        expandedReferences = expandedReferences;
    }

    // Resize handling - drag from bottom edge to expand panel downward
    function startResize(e: MouseEvent) {
        e.preventDefault();
        e.stopPropagation();
        isResizing = true;
        const startY = e.clientY;
        const startHeight = panelHeight;

        function onMouseMove(e: MouseEvent) {
            e.preventDefault();
            // Moving mouse down (positive delta) expands the panel
            const deltaY = e.clientY - startY;
            panelHeight = Math.max(150, Math.min(600, startHeight + deltaY));
        }

        function onMouseUp(e: MouseEvent) {
            e.preventDefault();
            isResizing = false;
            document.removeEventListener('mousemove', onMouseMove);
            document.removeEventListener('mouseup', onMouseUp);
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
        }

        // Use document instead of window and set cursor/select styles
        document.body.style.cursor = 'ns-resize';
        document.body.style.userSelect = 'none';
        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
    }

    // Track if an operation is in progress to prevent cascading updates
    let isProcessing = false;

    // Accept a suggestion (add to document)
    async function acceptSuggestion(suggestion: SuggestedStub, index: number) {
        if (isProcessing) return;
        isProcessing = true;

        const activeFile = plugin.app.workspace.getActiveFile();
        if (!activeFile || !config) {
            isProcessing = false;
            return;
        }

        try {
            let content = await plugin.app.vault.read(activeFile);

            // Get existing anchors to ensure uniqueness
            const existingAnchors = getValidAnchors(content, config.anchors);
            const existingAnchorIds = new Set(existingAnchors.map(a => a.id));

            // Generate a unique anchor ID
            const anchorId = generateAnchorId(config.anchors, suggestion.type, existingAnchorIds);

            // Insert anchor at the specified line in content (if lineNumber provided)
            // Note: lineNumber from LLM is 1-indexed, insertAnchorAtLine expects 0-indexed
            if (suggestion.location.lineNumber > 0) {
                const targetLine = suggestion.location.lineNumber - 1; // Convert to 0-indexed
                content = insertAnchorAtLine(content, targetLine, anchorId);
            }

            // Insert stub into frontmatter (with anchor reference)
            const stubEntry = buildStubYamlEntry(suggestion, anchorId);
            const newContent = insertYamlArrayItem(content, 'stubs', stubEntry);

            await plugin.app.vault.modify(activeFile, newContent);

            // Use tick() and requestAnimationFrame to ensure clean state transition
            await tick();
            requestAnimationFrame(() => {
                removeSuggestion(index);
                new Notice(`Stub added: ${suggestion.description.slice(0, 50)}...`);
                // Sync after a delay to let UI settle
                setTimeout(() => {
                    plugin.debouncedSyncStubsForActiveFile();
                    isProcessing = false;
                }, 150);
            });
        } catch (error) {
            console.error('[Doc Doctor] Failed to accept suggestion:', error);
            new Notice('Failed to add stub');
            isProcessing = false;
        }
    }

    // Accept a reference (add to frontmatter under configurable property)
    async function acceptReference(reference: FoundReference, index: number) {
        if (isProcessing) return;
        isProcessing = true;

        const activeFile = plugin.app.workspace.getActiveFile();
        if (!activeFile) {
            isProcessing = false;
            return;
        }

        const propertyName = getReferenceProperty(reference.type);

        try {
            const content = await plugin.app.vault.read(activeFile);
            const refEntry = buildReferenceYamlEntry(reference);
            const newContent = insertYamlArrayItem(content, propertyName, refEntry);
            await plugin.app.vault.modify(activeFile, newContent);

            // Use tick() and requestAnimationFrame to ensure clean state transition
            await tick();
            requestAnimationFrame(() => {
                removeReference(index);
                new Notice(`Reference added to ${propertyName}`);
                isProcessing = false;
            });
        } catch (error) {
            console.error('[Doc Doctor] Failed to accept reference:', error);
            new Notice('Failed to add reference');
            isProcessing = false;
        }
    }

    // Generic function to insert an item into a YAML array in frontmatter
    function insertYamlArrayItem(content: string, propertyName: string, entry: string): string {
        const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);

        if (frontmatterMatch) {
            const frontmatterContent = frontmatterMatch[1];
            const propRegex = new RegExp(`^${propertyName}:`, 'm');

            if (propRegex.test(frontmatterContent)) {
                // Property exists - check different formats

                // Format 1: Empty property (just "property:" with optional whitespace)
                const emptyPropertyRegex = new RegExp(`^(${propertyName}:)[ \\t]*$`, 'm');

                // Format 2: Inline empty array "property: []"
                const inlineEmptyArrayRegex = new RegExp(`^(${propertyName}:)[ \\t]*\\[\\][ \\t]*$`, 'm');

                // Format 3: Array with items
                const hasArrayRegex = new RegExp(`^${propertyName}:\\n([ \\t]+- )`, 'm');

                if (inlineEmptyArrayRegex.test(frontmatterContent)) {
                    // Empty inline array [] - replace with first entry
                    return content.replace(
                        new RegExp(`^${propertyName}:[ \\t]*\\[\\][ \\t]*$`, 'm'),
                        `${propertyName}:\n${entry}`
                    );
                } else if (emptyPropertyRegex.test(frontmatterContent)) {
                    // Empty property - add first entry
                    return content.replace(emptyPropertyRegex, `$1\n${entry}`);
                } else if (hasArrayRegex.test(frontmatterContent)) {
                    // Has entries - insert based on insertion order
                    // Match the property and all its array items
                    const arrayRegex = new RegExp(
                        `(^${propertyName}:\\n(?:[ \\t]+- [^\\n]*\\n?|[ \\t]+[^-][^\\n]*\\n?)*)`,
                        'm'
                    );
                    const match = frontmatterContent.match(arrayRegex);
                    if (match) {
                        const existingBlock = match[1];
                        let updatedBlock: string;
                        if (insertionOrder === 'top') {
                            // Insert after property declaration, before existing items
                            updatedBlock = `${propertyName}:\n${entry}\n` +
                                existingBlock.replace(new RegExp(`^${propertyName}:\\n`), '').trimEnd();
                        } else {
                            // Append at bottom (default)
                            updatedBlock = existingBlock.trimEnd() + '\n' + entry;
                        }
                        return content.replace(existingBlock, updatedBlock);
                    }
                }
                // Fallback - try to append to property
                return content.replace(
                    new RegExp(`^(${propertyName}:[^\\n]*)`, 'm'),
                    `$1\n${entry}`
                );
            } else {
                // Property doesn't exist - add before closing ---
                return content.replace(
                    /\n---/,
                    `\n${propertyName}:\n${entry}\n---`
                );
            }
        } else {
            // No frontmatter - create it
            return `---\n${propertyName}:\n${entry}\n---\n\n${content}`;
        }
    }

    // Build YAML entry for a stub (with anchor if provided)
    function buildStubYamlEntry(suggestion: SuggestedStub, anchorId?: string): string {
        // Always use structured format when we have an anchor or extra properties
        const hasExtraProperties = suggestion.stub_form !== 'transient' || suggestion.priority;
        const useStructuredFormat = anchorId || hasExtraProperties;

        if (useStructuredFormat) {
            const lines: string[] = [];
            lines.push(`  - ${suggestion.type}:`);
            lines.push(`      description: "${escapeYamlString(suggestion.description)}"`);
            if (anchorId) {
                lines.push(`      anchor: ${anchorId}`);
            }
            if (suggestion.stub_form && suggestion.stub_form !== 'transient') {
                lines.push(`      stub_form: ${suggestion.stub_form}`);
            }
            if (suggestion.priority) {
                lines.push(`      priority: ${suggestion.priority}`);
            }
            return lines.join('\n');
        } else {
            return `  - ${suggestion.type}: "${escapeYamlString(suggestion.description)}"`;
        }
    }

    // Build YAML entry for a reference
    function buildReferenceYamlEntry(reference: FoundReference): string {
        return `  - "${escapeYamlString(reference.target)}"`;
    }

    function escapeYamlString(str: string): string {
        return str.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, ' ');
    }

    // Reject handlers
    function rejectSuggestion(index: number) {
        removeSuggestion(index);
    }

    function rejectReference(index: number) {
        removeReference(index);
    }

    // Clear all
    function handleClearAll() {
        clearSuggestions();
    }

    // Accept all suggestions
    async function handleAcceptAllSuggestions() {
        for (let i = suggestions.length - 1; i >= 0; i--) {
            await acceptSuggestion(suggestions[i], i);
        }
    }

    // Accept all references
    async function handleAcceptAllReferences() {
        for (let i = references.length - 1; i >= 0; i--) {
            await acceptReference(references[i], i);
        }
    }

    // Tab handlers - use tick() to ensure clean state transitions
    async function switchToSuggestions() {
        await tick();
        requestAnimationFrame(() => {
            setActiveTab('suggestions');
        });
    }

    async function switchToReferences() {
        await tick();
        requestAnimationFrame(() => {
            setActiveTab('references');
        });
    }
</script>

{#if showPanel}
    <div class="suggestions-panel" style="--panel-height: {panelHeight}px">
        <!-- Header with streaming indicator -->
        <div class="suggestions-header">
            <div class="header-left">
                {#if isAnalyzing}
                    <Loader2 size={14} class="spin" />
                {:else}
                    <Sparkles size={14} />
                {/if}
                <span class="header-title">Analysis</span>
            </div>

            <!-- Streaming text display in header - show during and after analysis -->
            {#if streamingText}
                <button
                    class="streaming-band"
                    class:expanded={streamingExpanded}
                    class:live={isAnalyzing}
                    on:click={toggleStreamingExpanded}
                >
                    {#if isAnalyzing}
                        <Loader2 size={12} class="spin" />
                    {:else}
                        <Brain size={12} />
                    {/if}
                    <span class="streaming-preview">
                        {#if streamingExpanded}
                            <ChevronDown size={10} />
                        {:else}
                            {streamingText.slice(-80)}
                            <ChevronRight size={10} />
                        {/if}
                    </span>
                </button>
            {/if}

            <div class="header-actions">
                <button
                    class="action-btn"
                    on:click={handleClearAll}
                    title="Clear all"
                >
                    <X size={12} />
                </button>
            </div>
        </div>

        <!-- Expanded streaming view - show when expanded, during or after analysis -->
        {#if streamingExpanded && streamingText}
            <div class="streaming-expanded" bind:this={streamingScrollRef}>
                <div class="streaming-header">
                    <Brain size={12} />
                    <span>LLM Thought Stream</span>
                    {#if isAnalyzing}
                        <span class="streaming-live-indicator">Live</span>
                    {/if}
                </div>
                <pre>{streamingText}</pre>
            </div>
        {/if}

        {#if state.summary}
            <div class="analysis-summary">
                {state.summary}
            </div>
        {/if}

        {#if hasError}
            <div class="error-message">
                <AlertCircle size={14} />
                <span>{state.error?.message}</span>
                {#if state.error?.suggestedAction}
                    <div class="error-action">{state.error.suggestedAction}</div>
                {/if}
            </div>
        {/if}

        <!-- Tabs -->
        {#if hasContent}
            <div class="tabs">
                <button
                    class="tab"
                    class:active={activeTab === 'suggestions'}
                    on:click={switchToSuggestions}
                >
                    Stubs
                    {#if hasSuggestions}
                        <span class="tab-badge">{suggestions.length}</span>
                    {/if}
                </button>
                <button
                    class="tab"
                    class:active={activeTab === 'references'}
                    on:click={switchToReferences}
                >
                    References
                    {#if hasReferences}
                        <span class="tab-badge">{references.length}</span>
                    {/if}
                </button>
            </div>

            <!-- Suggestions tab -->
            {#if activeTab === 'suggestions' && hasSuggestions}
                <div class="tab-actions">
                    <button
                        class="action-btn accept-all"
                        on:click={handleAcceptAllSuggestions}
                        title="Accept all stubs"
                    >
                        <Check size={12} />
                        Accept All
                    </button>
                </div>
                <div class="items-list">
                    {#each suggestions as suggestion, index}
                        {@const typeInfo = getStubTypeInfo(suggestion.type)}
                        {@const isExpanded = expandedSuggestions.has(index)}
                        <div class="item">
                            <div class="item-main">
                                <button
                                    class="expand-btn"
                                    on:click={() => toggleSuggestionExpanded(index)}
                                >
                                    {#if isExpanded}
                                        <ChevronDown size={12} />
                                    {:else}
                                        <ChevronRight size={12} />
                                    {/if}
                                </button>
                                <span
                                    class="type-indicator"
                                    style="background-color: {typeInfo.color}"
                                    title={typeInfo.displayName}
                                ></span>
                                <span class="item-description">
                                    {suggestion.description}
                                </span>
                                <div class="item-actions">
                                    <button
                                        class="action-btn accept"
                                        on:click={() => acceptSuggestion(suggestion, index)}
                                        title="Accept"
                                    >
                                        <Check size={12} />
                                    </button>
                                    <button
                                        class="action-btn reject"
                                        on:click={() => rejectSuggestion(index)}
                                        title="Reject"
                                    >
                                        <X size={12} />
                                    </button>
                                </div>
                            </div>

                            {#if isExpanded}
                                <div class="item-details">
                                    <div class="detail-row">
                                        <span class="detail-label">Type:</span>
                                        <span class="detail-value">{typeInfo.displayName}</span>
                                    </div>
                                    {#if suggestion.stub_form}
                                        <div class="detail-row">
                                            <span class="detail-label">Form:</span>
                                            <span class="detail-value">{suggestion.stub_form}</span>
                                        </div>
                                    {/if}
                                    {#if suggestion.priority}
                                        <div class="detail-row">
                                            <span class="detail-label">Priority:</span>
                                            <span class="detail-value">{suggestion.priority}</span>
                                        </div>
                                    {/if}
                                    {#if suggestion.location?.section}
                                        <div class="detail-row">
                                            <span class="detail-label">Section:</span>
                                            <span class="detail-value">{suggestion.location.section}</span>
                                        </div>
                                    {/if}
                                    {#if suggestion.location?.lineNumber}
                                        <div class="detail-row">
                                            <span class="detail-label">Line:</span>
                                            <span class="detail-value line-number">{suggestion.location.lineNumber}</span>
                                        </div>
                                    {/if}
                                    {#if suggestion.rationale}
                                        <div class="detail-row rationale">
                                            <span class="detail-label">Rationale:</span>
                                            <span class="detail-value">{suggestion.rationale}</span>
                                        </div>
                                    {/if}
                                </div>
                            {/if}
                        </div>
                    {/each}
                </div>
            {/if}

            <!-- References tab -->
            {#if activeTab === 'references' && hasReferences}
                <div class="tab-actions">
                    {#if separateReferenceProperties}
                        <span class="property-hint">vault → {vaultReferenceProperty}: | web → {webReferenceProperty}:</span>
                    {:else}
                        <span class="property-hint">→ {vaultReferenceProperty}:</span>
                    {/if}
                    <button
                        class="action-btn accept-all"
                        on:click={handleAcceptAllReferences}
                        title="Accept all references"
                    >
                        <Check size={12} />
                        Accept All
                    </button>
                </div>
                <div class="items-list">
                    {#each references as reference, index}
                        {@const RefIcon = getReferenceIcon(reference.type)}
                        {@const isExpanded = expandedReferences.has(index)}
                        <div class="item">
                            <div class="item-main">
                                <button
                                    class="expand-btn"
                                    on:click={() => toggleReferenceExpanded(index)}
                                >
                                    {#if isExpanded}
                                        <ChevronDown size={12} />
                                    {:else}
                                        <ChevronRight size={12} />
                                    {/if}
                                </button>
                                <span class="ref-icon" title={reference.type}>
                                    <svelte:component this={RefIcon} size={12} />
                                </span>
                                <span class="item-description">
                                    {reference.title}
                                </span>
                                <div class="item-actions">
                                    <button
                                        class="action-btn accept"
                                        on:click={() => acceptReference(reference, index)}
                                        title="Accept"
                                    >
                                        <Check size={12} />
                                    </button>
                                    <button
                                        class="action-btn reject"
                                        on:click={() => rejectReference(index)}
                                        title="Reject"
                                    >
                                        <X size={12} />
                                    </button>
                                </div>
                            </div>

                            {#if isExpanded}
                                <div class="item-details">
                                    <div class="detail-row">
                                        <span class="detail-label">Type:</span>
                                        <span class="detail-value">{reference.type}</span>
                                    </div>
                                    <div class="detail-row">
                                        <span class="detail-label">Target:</span>
                                        <span class="detail-value target">{reference.target}</span>
                                    </div>
                                    {#if reference.section}
                                        <div class="detail-row">
                                            <span class="detail-label">Section:</span>
                                            <span class="detail-value">{reference.section}</span>
                                        </div>
                                    {/if}
                                    {#if reference.context}
                                        <div class="detail-row rationale">
                                            <span class="detail-label">Context:</span>
                                            <span class="detail-value">{reference.context}</span>
                                        </div>
                                    {/if}
                                </div>
                            {/if}
                        </div>
                    {/each}
                </div>
            {/if}

            <!-- Empty state for tabs -->
            {#if activeTab === 'suggestions' && !hasSuggestions}
                <div class="empty-state">No stub suggestions</div>
            {/if}
            {#if activeTab === 'references' && !hasReferences}
                <div class="empty-state">No references found</div>
            {/if}
        {/if}

        {#if state.confidence > 0}
            <div class="confidence-bar">
                <span class="confidence-label">Confidence:</span>
                <div class="confidence-track">
                    <div
                        class="confidence-fill"
                        style="width: {state.confidence * 100}%"
                    ></div>
                </div>
                <span class="confidence-value">{Math.round(state.confidence * 100)}%</span>
            </div>
        {/if}

        <!-- Resize handle -->
        <div
            class="resize-handle"
            on:mousedown={startResize}
            class:resizing={isResizing}
        >
            <GripVertical size={12} />
        </div>
    </div>
{/if}

<style>
    .suggestions-panel {
        background: var(--background-secondary);
        border: 1px solid var(--background-modifier-border);
        border-radius: 6px;
        margin-bottom: 8px;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        height: var(--panel-height);
        min-height: 150px;
        max-height: 600px;
        position: relative;
    }

    .suggestions-header {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 10px;
        background: var(--background-modifier-hover);
        border-bottom: 1px solid var(--background-modifier-border);
        flex-shrink: 0;
    }

    .header-left {
        display: flex;
        align-items: center;
        gap: 6px;
        flex-shrink: 0;
    }

    .header-title {
        font-size: var(--font-ui-small);
        font-weight: 500;
        color: var(--text-normal);
    }

    .header-actions {
        display: flex;
        gap: 4px;
        flex-shrink: 0;
        margin-left: auto;
    }

    /* Streaming band in header */
    .streaming-band {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 2px 8px;
        border: none;
        background: var(--background-primary);
        border-radius: 4px;
        color: var(--text-muted);
        font-size: 11px;
        font-family: var(--font-monospace);
        cursor: pointer;
        overflow: hidden;
        min-width: 0;
    }

    .streaming-band:hover {
        background: var(--background-modifier-hover);
    }

    .streaming-band.live {
        border: 1px solid var(--color-purple);
    }

    .streaming-preview {
        display: flex;
        align-items: center;
        gap: 4px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .streaming-expanded {
        max-height: 120px;
        overflow-y: auto;
        padding: 8px 10px;
        background: var(--background-primary);
        border-bottom: 1px solid var(--background-modifier-border);
        font-family: var(--font-monospace);
        font-size: 11px;
        color: var(--text-muted);
        flex-shrink: 0;
    }

    .streaming-expanded pre {
        margin: 0;
        white-space: pre-wrap;
        word-break: break-word;
    }

    .streaming-header {
        display: flex;
        align-items: center;
        gap: 6px;
        padding-bottom: 6px;
        margin-bottom: 6px;
        border-bottom: 1px solid var(--background-modifier-border);
        color: var(--text-muted);
        font-size: var(--font-ui-smaller);
    }

    .streaming-live-indicator {
        margin-left: auto;
        padding: 1px 6px;
        background: var(--color-purple);
        color: white;
        border-radius: 8px;
        font-size: 10px;
        animation: pulse 1.5s ease-in-out infinite;
    }

    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.6; }
    }

    /* Spin animation */
    :global(.spin) {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }

    /* Tabs */
    .tabs {
        display: flex;
        border-bottom: 1px solid var(--background-modifier-border);
        flex-shrink: 0;
    }

    .tab {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        padding: 8px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        font-size: var(--font-ui-smaller);
        cursor: pointer;
        border-bottom: 2px solid transparent;
    }

    .tab:hover {
        background: var(--background-modifier-hover);
    }

    .tab.active {
        color: var(--text-normal);
        border-bottom-color: var(--interactive-accent);
    }

    .tab-badge {
        background: var(--color-purple);
        color: white;
        padding: 0 5px;
        border-radius: 8px;
        font-size: 10px;
    }

    .tab-actions {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 8px;
        padding: 6px 10px;
        border-bottom: 1px solid var(--background-modifier-border);
        flex-shrink: 0;
    }

    .property-hint {
        font-size: var(--font-ui-smaller);
        color: var(--text-faint);
        font-style: italic;
    }

    /* Action buttons */
    .action-btn {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 3px 6px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        font-size: var(--font-ui-smaller);
        cursor: pointer;
        border-radius: 4px;
    }

    .action-btn:hover {
        background: var(--background-modifier-hover);
        color: var(--text-normal);
    }

    .action-btn.accept-all {
        background: var(--interactive-success);
        color: white;
    }

    .action-btn.accept-all:hover {
        filter: brightness(1.1);
    }

    .action-btn.accept {
        color: var(--color-green);
    }

    .action-btn.accept:hover {
        background: rgba(0, 200, 0, 0.15);
    }

    .action-btn.reject {
        color: var(--color-red);
    }

    .action-btn.reject:hover {
        background: rgba(200, 0, 0, 0.15);
    }

    /* Content sections */
    .analysis-summary {
        padding: 8px 10px;
        font-size: var(--font-ui-smaller);
        color: var(--text-muted);
        border-bottom: 1px solid var(--background-modifier-border);
        flex-shrink: 0;
    }

    .error-message {
        display: flex;
        flex-direction: column;
        gap: 4px;
        padding: 10px;
        background: rgba(200, 0, 0, 0.1);
        color: var(--text-error);
        font-size: var(--font-ui-smaller);
        flex-shrink: 0;
    }

    .error-action {
        color: var(--text-muted);
        font-style: italic;
    }

    .empty-state {
        padding: 16px;
        text-align: center;
        color: var(--text-faint);
        font-size: var(--font-ui-smaller);
    }

    /* Items list - scrollable */
    .items-list {
        flex: 1;
        overflow-y: auto;
        min-height: 0;
    }

    .item {
        border-bottom: 1px solid var(--background-modifier-border);
    }

    .item:last-child {
        border-bottom: none;
    }

    .item-main {
        display: flex;
        align-items: flex-start;
        gap: 6px;
        padding: 8px 10px;
    }

    .expand-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 2px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        flex-shrink: 0;
    }

    .expand-btn:hover {
        color: var(--text-normal);
    }

    .type-indicator {
        width: 8px;
        height: 8px;
        border-radius: 2px;
        flex-shrink: 0;
        margin-top: 4px;
    }

    .ref-icon {
        display: flex;
        align-items: center;
        color: var(--text-muted);
        margin-top: 2px;
    }

    .item-description {
        flex: 1;
        font-size: var(--font-ui-smaller);
        color: var(--text-normal);
        word-break: break-word;
    }

    .item-actions {
        display: flex;
        gap: 2px;
        flex-shrink: 0;
    }

    .item-details {
        padding: 6px 10px 10px 28px;
        background: var(--background-primary);
    }

    .detail-row {
        display: flex;
        gap: 8px;
        font-size: var(--font-ui-smaller);
        margin-bottom: 4px;
    }

    .detail-row.rationale {
        flex-direction: column;
        gap: 2px;
    }

    .detail-label {
        color: var(--text-muted);
        flex-shrink: 0;
    }

    .detail-value {
        color: var(--text-normal);
    }

    .detail-value.target {
        font-family: var(--font-monospace);
        font-size: 11px;
        word-break: break-all;
    }

    .detail-value.line-number {
        font-family: var(--font-monospace);
        background: var(--background-modifier-hover);
        padding: 1px 6px;
        border-radius: 3px;
    }

    /* Confidence bar */
    .confidence-bar {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 10px;
        font-size: var(--font-ui-smaller);
        color: var(--text-muted);
        border-top: 1px solid var(--background-modifier-border);
        flex-shrink: 0;
    }

    .confidence-track {
        flex: 1;
        height: 4px;
        background: var(--background-modifier-border);
        border-radius: 2px;
        overflow: hidden;
    }

    .confidence-fill {
        height: 100%;
        background: var(--interactive-accent);
        border-radius: 2px;
    }

    .confidence-value {
        min-width: 36px;
        text-align: right;
    }

    /* Resize handle */
    .resize-handle {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 16px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: ns-resize;
        color: var(--text-faint);
        background: linear-gradient(to bottom, transparent 0%, var(--background-secondary) 50%);
        z-index: 10;
        transition: color 0.1s, background 0.1s;
    }

    .resize-handle:hover,
    .resize-handle.resizing {
        color: var(--text-normal);
        background: var(--background-modifier-hover);
    }

    .resize-handle.resizing {
        background: var(--interactive-accent);
        color: white;
    }
</style>
