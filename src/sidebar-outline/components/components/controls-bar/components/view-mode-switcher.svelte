<script lang="ts">
    import { controls, type ViewMode } from '../controls-bar.store';
    import { MessageSquare, MessageSquareDashed, Sparkles, Compass } from 'lucide-svelte';
    import { onMount } from 'svelte';
    import type LabeledAnnotations from '../../../../../main';
    import { llmAnalysisState } from '../../../../../stubs/llm-analysis-store';

    export let plugin: LabeledAnnotations;

    $: suggestionCount = $llmAnalysisState.suggestions.length;

    // Hover state for expansion
    let isExpanded = false;
    let hoverTimeout: ReturnType<typeof setTimeout> | null = null;
    let collapseTimeout: ReturnType<typeof setTimeout> | null = null;

    // View mode icons and labels
    const viewModes: { mode: ViewMode; label: string; icon: typeof MessageSquare }[] = [
        { mode: 'annotations', label: 'Annotations', icon: MessageSquare },
        { mode: 'stubs', label: 'Stubs', icon: MessageSquareDashed },
        { mode: 'ai', label: 'AI Analysis', icon: Sparkles },
        { mode: 'explore', label: 'Explore', icon: Compass },
    ];

    // Load saved view mode on mount
    onMount(() => {
        const savedMode = plugin.settings.getValue().outline.sidebarViewMode;
        if (savedMode && savedMode !== $controls.viewMode) {
            controls.dispatch({ type: 'SET_VIEW_MODE', payload: savedMode });
        }
    });

    const setViewMode = (mode: ViewMode) => {
        controls.dispatch({ type: 'SET_VIEW_MODE', payload: mode });
        // Save to settings
        plugin.settings.dispatch({ type: 'SET_SIDEBAR_VIEW_MODE', payload: { mode } });
        plugin.saveSettings();
        // Collapse after selection
        isExpanded = false;
    };

    const handleMouseEnter = () => {
        if (collapseTimeout) {
            clearTimeout(collapseTimeout);
            collapseTimeout = null;
        }
        // Small delay before expanding to avoid accidental triggers
        hoverTimeout = setTimeout(() => {
            isExpanded = true;
        }, 150);
    };

    const handleMouseLeave = () => {
        if (hoverTimeout) {
            clearTimeout(hoverTimeout);
            hoverTimeout = null;
        }
        // Delay collapse to allow moving between buttons
        collapseTimeout = setTimeout(() => {
            isExpanded = false;
        }, 300);
    };

    // Get current mode info
    $: currentMode = viewModes.find(v => v.mode === $controls.viewMode) || viewModes[0];
    $: inactiveVodes = viewModes.filter(v => v.mode !== $controls.viewMode);
</script>

<div
    class="view-mode-switcher"
    class:expanded={isExpanded}
    on:mouseenter={handleMouseEnter}
    on:mouseleave={handleMouseLeave}
    role="group"
    aria-label="View mode selector"
>
    <!-- Active/Current Mode Button (always visible) -->
    <button
        class="view-mode-btn active"
        aria-label={currentMode.label}
        title={currentMode.label}
    >
        <svelte:component this={currentMode.icon} size={16} />
        {#if currentMode.mode === 'ai' && suggestionCount > 0}
            <span class="suggestion-badge">{suggestionCount}</span>
        {/if}
    </button>

    <!-- Expandable section with other modes -->
    <div class="expandable-section">
        {#each inactiveVodes as viewMode (viewMode.mode)}
            <button
                class="view-mode-btn"
                on:click={() => setViewMode(viewMode.mode)}
                aria-label={viewMode.label}
                title={viewMode.label}
            >
                <svelte:component this={viewMode.icon} size={16} />
                {#if viewMode.mode === 'ai' && suggestionCount > 0}
                    <span class="suggestion-badge">{suggestionCount}</span>
                {/if}
            </button>
        {/each}
    </div>
</div>

<style>
    .view-mode-switcher {
        display: flex;
        align-items: center;
        background: var(--background-modifier-border);
        border-radius: 6px;
        padding: 2px;
        overflow: hidden;
        transition: all 0.2s ease;
    }

    .expandable-section {
        display: flex;
        gap: 2px;
        max-width: 0;
        opacity: 0;
        overflow: hidden;
        transition: max-width 0.25s ease, opacity 0.2s ease;
    }

    .view-mode-switcher.expanded .expandable-section {
        max-width: 150px;
        opacity: 1;
        margin-left: 2px;
    }

    .view-mode-btn {
        position: relative;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 4px 8px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.15s ease;
        flex-shrink: 0;
    }

    .view-mode-btn:hover {
        color: var(--text-normal);
        background: var(--background-modifier-hover);
    }

    .view-mode-btn.active {
        background: var(--background-primary);
        color: var(--text-normal);
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    }

    .suggestion-badge {
        position: absolute;
        top: -4px;
        right: -4px;
        min-width: 14px;
        height: 14px;
        font-size: 9px;
        font-weight: 600;
        line-height: 14px;
        text-align: center;
        background: var(--color-purple);
        color: white;
        border-radius: 7px;
        padding: 0 3px;
    }
</style>
