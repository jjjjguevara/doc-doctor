<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { writable } from 'svelte/store';
    import { Wand2, Loader2, Settings, Trash2, ChevronDown } from 'lucide-svelte';
    import LabeledAnnotations from '../../../../main';
    import { llmAnalysisState, triggerLLMAnalysis, clearSuggestions } from '../../../../stubs/llm-analysis-store';
    import { BUILT_IN_TEMPLATES, type LLMConfiguration, type PromptTemplate } from '../../../../llm/llm-types';

    export let plugin: LabeledAnnotations;

    // Create a local store for LLM settings that we can subscribe to
    const llmSettingsStore = writable<LLMConfiguration | null>(null);
    let unsubscribe: (() => void) | null = null;
    let showTemplateDropdown = false;

    onMount(() => {
        // Subscribe to plugin settings and update our local store
        unsubscribe = plugin.settings.subscribe((settings) => {
            llmSettingsStore.set(settings.llm);
        });
    });

    onDestroy(() => {
        if (unsubscribe) unsubscribe();
    });

    // LLM analysis state - reactive
    $: llmSettings = $llmSettingsStore;
    $: isLLMConfigured = llmSettings?.enabled && llmSettings?.apiKey;
    $: isAnalyzing = $llmAnalysisState.isAnalyzing;
    $: suggestionCount = $llmAnalysisState.suggestions.length;

    // Template state
    $: selectedTemplateId = llmSettings?.selectedTemplateId || 'standard';
    $: allTemplates = BUILT_IN_TEMPLATES;
    $: selectedTemplate = allTemplates.find(t => t.id === selectedTemplateId) || allTemplates[0];

    const handleMagicWand = () => {
        if (!isLLMConfigured) {
            // Show settings
            plugin.app.setting.open();
            plugin.app.setting.openTabById('doc-doctor');
            return;
        }
        triggerLLMAnalysis(plugin);
    };

    const handleOpenSettings = () => {
        plugin.app.setting.open();
        plugin.app.setting.openTabById('doc-doctor');
    };

    const handleClearSuggestions = () => {
        clearSuggestions();
    };

    const toggleTemplateDropdown = () => {
        showTemplateDropdown = !showTemplateDropdown;
    };

    const selectTemplate = (template: PromptTemplate) => {
        plugin.settings.dispatch({
            type: 'LLM_SET_SELECTED_TEMPLATE',
            payload: { templateId: template.id },
        });
        plugin.saveSettings();
        showTemplateDropdown = false;
    };

    // Close dropdown when clicking outside
    function handleClickOutside(event: MouseEvent) {
        const target = event.target as HTMLElement;
        if (showTemplateDropdown && !target.closest('.template-selector')) {
            showTemplateDropdown = false;
        }
    }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="ai-controls">
    <!-- Magic Wand - LLM Analysis -->
    <button
        class="control-btn magic-wand"
        class:active={isAnalyzing}
        class:disabled={!isLLMConfigured && !isAnalyzing}
        on:click={handleMagicWand}
        disabled={isAnalyzing}
        title={isLLMConfigured
            ? (isAnalyzing ? 'Analyzing...' : 'Analyze document for stub suggestions')
            : 'Configure LLM in settings to enable'}
    >
        {#if isAnalyzing}
            <Loader2 size={14} class="spin" />
        {:else}
            <Wand2 size={14} />
        {/if}
    </button>

    <!-- Template selector -->
    <div class="template-selector">
        <button
            class="template-btn"
            on:click={toggleTemplateDropdown}
            title={selectedTemplate.description}
        >
            <span class="template-name">{selectedTemplate.name}</span>
            <ChevronDown size={12} />
        </button>

        {#if showTemplateDropdown}
            <div class="template-dropdown">
                {#each allTemplates as template}
                    <button
                        class="template-option"
                        class:selected={template.id === selectedTemplateId}
                        on:click={() => selectTemplate(template)}
                    >
                        <span class="template-option-name">{template.name}</span>
                        <span class="template-option-desc">{template.description}</span>
                    </button>
                {/each}
            </div>
        {/if}
    </div>

    {#if suggestionCount > 0}
        <button
            class="control-btn"
            on:click={handleClearSuggestions}
            title="Clear suggestions"
        >
            <Trash2 size={14} />
        </button>
    {/if}

    <button
        class="control-btn"
        on:click={handleOpenSettings}
        title="AI Settings"
    >
        <Settings size={14} />
    </button>

    {#if suggestionCount > 0}
        <span class="suggestion-count" title="Suggestions">
            {suggestionCount}
        </span>
    {/if}
</div>

<style>
    .ai-controls {
        display: flex;
        align-items: center;
        gap: 4px;
    }

    .control-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 4px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.15s ease;
    }

    .control-btn:hover {
        background: var(--background-modifier-hover);
        color: var(--text-normal);
    }

    .control-btn.active {
        background: var(--interactive-accent);
        color: var(--text-on-accent);
    }

    .control-btn.disabled {
        opacity: 0.5;
    }

    .control-btn.magic-wand.active {
        background: var(--color-purple);
        color: white;
    }

    /* Spin animation for loading state */
    :global(.spin) {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    .suggestion-count {
        font-size: var(--font-ui-smaller);
        color: var(--text-muted);
        background: var(--background-modifier-border);
        padding: 2px 6px;
        border-radius: 10px;
        margin-left: 4px;
    }

    /* Template selector */
    .template-selector {
        position: relative;
    }

    .template-btn {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 3px 8px;
        border: 1px solid var(--background-modifier-border);
        border-radius: 4px;
        background: var(--background-primary);
        color: var(--text-muted);
        font-size: var(--font-ui-smaller);
        cursor: pointer;
        max-width: 120px;
    }

    .template-btn:hover {
        background: var(--background-modifier-hover);
        color: var(--text-normal);
    }

    .template-name {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .template-dropdown {
        position: absolute;
        top: 100%;
        right: 0;
        z-index: 100;
        min-width: 180px;
        max-width: calc(100vw - 32px);
        width: max-content;
        margin-top: 4px;
        background: var(--background-primary);
        border: 1px solid var(--background-modifier-border);
        border-radius: 6px;
        box-shadow: var(--shadow-s);
        overflow: hidden;
    }

    .template-option {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 2px;
        width: 100%;
        padding: 8px 10px;
        border: none;
        background: transparent;
        cursor: pointer;
        text-align: left;
    }

    .template-option:hover {
        background: var(--background-modifier-hover);
    }

    .template-option.selected {
        background: var(--background-modifier-active-hover);
    }

    .template-option-name {
        font-size: var(--font-ui-small);
        color: var(--text-normal);
        font-weight: 500;
    }

    .template-option-desc {
        font-size: var(--font-ui-smaller);
        color: var(--text-muted);
        word-wrap: break-word;
        overflow-wrap: break-word;
    }
</style>
