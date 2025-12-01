<script lang="ts">
    import { Search, ListFilter, Plus, RefreshCw } from 'lucide-svelte';
    import {
        filterText,
        hiddenTypes,
        hasOrphans,
        setFilterText,
    } from '../stubs-store';
    import type LabeledAnnotations from '../../main';
    import { writable } from 'svelte/store';

    export let plugin: LabeledAnnotations;

    // Local UI state
    const showSearchInput = writable(false);
    const showTypeFilter = writable(false);

    function toggleSearchInput() {
        showSearchInput.update((v) => !v);
    }

    function toggleTypeFilter() {
        showTypeFilter.update((v) => !v);
    }

    function handleSearchInput(event: Event) {
        const target = event.target as HTMLInputElement;
        setFilterText(target.value);
    }

    function handleInsertStub() {
        // Dispatch command to insert stub at cursor
        plugin.app.commands.executeCommandById('m-stubs:insert-stub');
    }

    function handleSync() {
        // Dispatch command to sync stubs
        plugin.app.commands.executeCommandById('m-stubs:sync-stubs');
    }
</script>

<div class="stubs-controls">
    <div class="nav-buttons-container">
        <button
            class="nav-button"
            class:is-active={$showSearchInput}
            class:has-value={!!$filterText}
            on:click={toggleSearchInput}
            aria-label="Search stubs"
            title="Search stubs"
        >
            <Search size={16} />
        </button>

        <button
            class="nav-button"
            class:is-active={$showTypeFilter}
            class:has-value={$hiddenTypes.size > 0}
            on:click={toggleTypeFilter}
            aria-label="Filter by type"
            title="Filter by type"
        >
            <ListFilter size={16} />
        </button>

        <button
            class="nav-button"
            on:click={handleInsertStub}
            aria-label="Insert new stub"
            title="Insert new stub"
        >
            <Plus size={16} />
        </button>

        <button
            class="nav-button"
            class:has-orphans={$hasOrphans}
            on:click={handleSync}
            aria-label="Sync stubs"
            title="Sync stubs"
        >
            <RefreshCw size={16} />
        </button>
    </div>

    {#if $showSearchInput}
        <div class="search-container">
            <input
                type="text"
                class="search-input"
                placeholder="Search stubs..."
                value={$filterText}
                on:input={handleSearchInput}
            />
        </div>
    {/if}

    {#if $showTypeFilter}
        <div class="type-filter-container">
            <slot name="type-filter" />
        </div>
    {/if}
</div>

<style>
    .stubs-controls {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        width: 100%;
        gap: 10px;
        box-sizing: border-box;
    }

    .nav-buttons-container {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 4px;
    }

    .nav-button {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        border: none;
        border-radius: var(--radius-s);
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .nav-button:hover {
        background: var(--background-modifier-hover);
        color: var(--text-normal);
    }

    .nav-button.is-active {
        background: var(--interactive-accent);
        color: var(--text-on-accent);
    }

    .nav-button.has-value {
        color: var(--interactive-accent);
    }

    .nav-button.has-orphans {
        color: var(--text-warning);
    }

    .search-container {
        width: 100%;
        padding: 0 10px;
        box-sizing: border-box;
    }

    .search-input {
        width: 100%;
        padding: 6px 10px;
        border: 1px solid var(--background-modifier-border);
        border-radius: var(--radius-s);
        background: var(--background-primary);
        color: var(--text-normal);
        font-size: var(--font-small);
    }

    .search-input:focus {
        outline: none;
        border-color: var(--interactive-accent);
    }

    .type-filter-container {
        width: 100%;
        padding: 0 10px;
        box-sizing: border-box;
    }
</style>
