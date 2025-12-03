<script lang="ts">
    import { controls } from '../controls-bar/controls-bar.store';
    import { RefreshCw, Settings, Wifi, WifiOff } from 'lucide-svelte';
    import LabeledAnnotations from '../../../../main';
    import { isLoading, resultsCount } from '../../../../explore-view/explore-store';

    export let plugin: LabeledAnnotations;

    $: scStatus = plugin.smartConnectionsService?.getStatus();
    $: isAvailable = scStatus?.smartConnections ?? false;

    const toggleSettings = () => {
        controls.dispatch({ type: 'TOGGLE_EXPLORE_SETTINGS' });
    };

    const refresh = () => {
        // Trigger refresh in explore panel
        // This is handled by the ExplorePanel component
        window.dispatchEvent(new CustomEvent('explore-refresh'));
    };
</script>

<div class="explore-controls">
    <button
        class="control-btn"
        on:click={refresh}
        title="Refresh"
        disabled={$isLoading}
    >
        <span class:spinning={$isLoading}>
            <RefreshCw size={14} />
        </span>
    </button>

    <div class="status-indicator" title={isAvailable ? 'Smart Connections active' : 'Using keyword fallback'}>
        {#if isAvailable}
            <span class="status-active"><Wifi size={12} /></span>
        {:else}
            <span class="status-fallback"><WifiOff size={12} /></span>
        {/if}
    </div>

    <button
        class="control-btn"
        class:active={$controls.showExploreSettings}
        on:click={toggleSettings}
        title="Explore settings"
    >
        <Settings size={14} />
    </button>

    <span class="result-count" title="Results">
        {$resultsCount}
    </span>
</div>

<style>
    .explore-controls {
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

    .control-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .status-indicator {
        display: flex;
        align-items: center;
        padding: 4px;
    }

    .status-active {
        color: var(--color-green);
    }

    .status-fallback {
        color: var(--text-muted);
    }

    .result-count {
        font-size: var(--font-ui-smaller);
        color: var(--text-muted);
        background: var(--background-modifier-border);
        padding: 2px 6px;
        border-radius: 10px;
        margin-left: 4px;
    }

    :global(.spinning) {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }
</style>
