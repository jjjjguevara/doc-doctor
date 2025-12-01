<script lang="ts">
    import { controls, type ViewMode } from '../controls-bar.store';
    import { MessageSquare, MessageSquareDashed } from 'lucide-svelte';
    import { onMount } from 'svelte';
    import type LabeledAnnotations from '../../../../../main';

    export let plugin: LabeledAnnotations;

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
    };
</script>

<div class="view-mode-switcher">
    <button
        class="view-mode-btn"
        class:active={$controls.viewMode === 'annotations'}
        on:click={() => setViewMode('annotations')}
        aria-label="Annotations"
        title="Annotations"
    >
        <MessageSquare size={16} />
    </button>
    <button
        class="view-mode-btn"
        class:active={$controls.viewMode === 'stubs'}
        on:click={() => setViewMode('stubs')}
        aria-label="Stubs"
        title="Stubs"
    >
        <MessageSquareDashed size={16} />
    </button>
</div>

<style>
    .view-mode-switcher {
        display: flex;
        gap: 2px;
        background: var(--background-modifier-border);
        border-radius: 6px;
        padding: 2px;
    }

    .view-mode-btn {
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
</style>
