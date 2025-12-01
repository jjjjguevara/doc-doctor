<script lang="ts">
    import StubsControlsBar from './stubs-controls-bar.svelte';
    import StubsList from './stubs-list.svelte';
    import NoStubs from './no-stubs.svelte';
    import {
        syncState,
        stubsConfig,
        filterText,
        visibleStubs,
        isLoading,
        errorMessage,
    } from '../stubs-store';
    import type LabeledAnnotations from '../../main';

    export let plugin: LabeledAnnotations;
</script>

<div class="stubs-panel">
    {#if $isLoading}
        <div class="stubs-loading">
            <span>Loading stubs...</span>
        </div>
    {:else if $errorMessage}
        <div class="stubs-error">
            <span class="error-icon">⚠</span>
            <span>{$errorMessage}</span>
        </div>
    {:else if !$stubsConfig}
        <div class="stubs-not-configured">
            <span>Stubs not configured</span>
        </div>
    {:else}
        <StubsControlsBar {plugin} />
        {#if $visibleStubs.length > 0 || $filterText}
            <StubsList {plugin} />
        {:else if $syncState.stubs.length === 0}
            <NoStubs />
        {:else}
            <StubsList {plugin} />
        {/if}
    {/if}
</div>

<style>
    .stubs-panel {
        height: 100%;
        width: 100%;
        box-sizing: border-box;
        display: flex;
        gap: 8px;
        flex-direction: column;
        align-items: start;
        justify-content: start;
    }

    .stubs-loading,
    .stubs-error,
    .stubs-not-configured {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        padding: 20px;
        color: var(--text-muted);
        font-size: var(--font-small);
    }

    .stubs-error {
        color: var(--text-error);
        gap: 8px;
    }

    .error-icon {
        font-size: 16px;
    }
</style>
