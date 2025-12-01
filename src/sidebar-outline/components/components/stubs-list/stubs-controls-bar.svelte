<script lang="ts">
    import { controls } from '../controls-bar/controls-bar.store';
    import {
        stubCount,
        orphanedStubCount,
        orphanedAnchorCount,
        filterText,
        setFilterText,
        activeTypeFilters,
        toggleTypeFilter,
        clearTypeFilters,
        stubSortOrder,
        cycleSortOrder,
        stubsConfig,
    } from '../../../../stubs/stubs-store';
    import { getSortedStubTypes } from '../../../../stubs/stubs-defaults';
    import { Search, Settings, RefreshCw, X, Filter, ArrowDown, ArrowUp, Layers } from 'lucide-svelte';
    import LabeledAnnotations from '../../../../main';

    export let plugin: LabeledAnnotations;

    let showSearch = false;
    let showTypeDropdown = false;
    let searchInput: HTMLInputElement;

    $: config = $stubsConfig;
    $: stubTypes = config ? getSortedStubTypes(config) : [];
    $: hasActiveFilters = $activeTypeFilters.size > 0;
    $: activeFilterCount = $activeTypeFilters.size;

    const toggleSettings = () => {
        controls.dispatch({ type: 'TOGGLE_STUBS_SETTINGS' });
    };

    const toggleSearch = () => {
        showSearch = !showSearch;
        showTypeDropdown = false;
        if (showSearch) {
            // Close settings view when opening search
            if ($controls.showStubsSettings) {
                controls.dispatch({ type: 'TOGGLE_STUBS_SETTINGS' });
            }
            setTimeout(() => searchInput?.focus(), 50);
        } else {
            setFilterText('');
        }
    };

    const toggleTypeDropdown = () => {
        showTypeDropdown = !showTypeDropdown;
        showSearch = false;
    };

    const handleTypeToggle = (typeKey: string) => {
        toggleTypeFilter(typeKey);
    };

    const handleClearFilters = () => {
        clearTypeFilters();
    };

    const handleSearchInput = (e: Event) => {
        const target = e.target as HTMLInputElement;
        setFilterText(target.value);
    };

    const clearSearch = () => {
        setFilterText('');
        searchInput?.focus();
    };

    const syncStubs = () => {
        plugin.syncStubsForActiveFile();
    };

    const handleSort = () => {
        cycleSortOrder();
    };

    // Close dropdown when clicking outside
    function handleClickOutside(event: MouseEvent) {
        const target = event.target as HTMLElement;
        if (showTypeDropdown && !target.closest('.filter-container')) {
            showTypeDropdown = false;
        }
    }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="stubs-controls">
    <button
        class="control-btn"
        on:click={syncStubs}
        title="Sync stubs"
    >
        <RefreshCw size={14} />
    </button>
    <button
        class="control-btn"
        class:active={showSearch}
        on:click={toggleSearch}
        title="Search stubs"
    >
        <Search size={14} />
    </button>

    <!-- Type Filter Button -->
    <div class="filter-container">
        <button
            class="control-btn"
            class:active={hasActiveFilters || showTypeDropdown}
            on:click={toggleTypeDropdown}
            title="Filter by type{hasActiveFilters ? ` (${activeFilterCount} selected)` : ''}"
        >
            <Filter size={14} />
            {#if hasActiveFilters}
                <span class="filter-badge">{activeFilterCount}</span>
            {/if}
        </button>

        {#if showTypeDropdown}
            <div class="type-dropdown">
                <button
                    class="dropdown-item clear-item"
                    class:disabled={!hasActiveFilters}
                    on:click={handleClearFilters}
                >
                    <span class="dropdown-check"></span>
                    <span>Clear filters</span>
                    {#if hasActiveFilters}
                        <X size={12} />
                    {/if}
                </button>
                <div class="dropdown-divider"></div>
                {#each stubTypes as typeDef}
                    <button
                        class="dropdown-item"
                        class:selected={$activeTypeFilters.has(typeDef.key)}
                        on:click={() => handleTypeToggle(typeDef.key)}
                    >
                        <span class="dropdown-check">{$activeTypeFilters.has(typeDef.key) ? '✓' : ''}</span>
                        <span class="type-indicator" style="background-color: {typeDef.color}"></span>
                        <span>{typeDef.displayName}</span>
                    </button>
                {/each}
            </div>
        {/if}
    </div>

    <!-- Sort Button -->
    <button
        class="control-btn"
        class:active={$stubSortOrder !== 'type'}
        on:click={handleSort}
        title="Sort: {$stubSortOrder === 'type' ? 'By type' : $stubSortOrder === 'asc' ? 'First to last' : 'Last to first'}"
    >
        {#if $stubSortOrder === 'type'}
            <Layers size={14} />
        {:else if $stubSortOrder === 'asc'}
            <ArrowDown size={14} />
        {:else}
            <ArrowUp size={14} />
        {/if}
    </button>

    <button
        class="control-btn"
        class:active={$controls.showStubsSettings}
        on:click={toggleSettings}
        title="Stub type settings"
    >
        <Settings size={14} />
    </button>

    <span class="stub-count" title="Total stubs">
        {$stubCount}
    </span>
    {#if $orphanedStubCount > 0 || $orphanedAnchorCount > 0}
        <span class="orphan-count" title="Orphaned items">
            {$orphanedStubCount + $orphanedAnchorCount}
        </span>
    {/if}
</div>

{#if showSearch}
    <div class="stubs-search">
        <input
            bind:this={searchInput}
            type="text"
            placeholder="Search stubs..."
            value={$filterText}
            on:input={handleSearchInput}
            class="search-input"
        />
        {#if $filterText}
            <button class="clear-btn" on:click={clearSearch} title="Clear search">
                <X size={12} />
            </button>
        {/if}
    </div>
{/if}

<style>
    .stubs-controls {
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

    .stub-count {
        font-size: var(--font-ui-smaller);
        color: var(--text-muted);
        background: var(--background-modifier-border);
        padding: 2px 6px;
        border-radius: 10px;
        margin-left: 4px;
    }

    .orphan-count {
        font-size: var(--font-ui-smaller);
        color: var(--text-warning);
        background: rgba(255, 165, 0, 0.2);
        padding: 2px 6px;
        border-radius: 10px;
    }

    .stubs-search {
        display: flex;
        align-items: center;
        gap: 4px;
        width: 100%;
        margin-top: 8px;
        position: relative;
    }

    .search-input {
        flex: 1;
        padding: 4px 8px;
        border: 1px solid var(--background-modifier-border);
        border-radius: 4px;
        background: var(--background-primary);
        color: var(--text-normal);
        font-size: var(--font-ui-smaller);
        outline: none;
    }

    .search-input:focus {
        border-color: var(--interactive-accent);
    }

    .search-input::placeholder {
        color: var(--text-faint);
    }

    .clear-btn {
        position: absolute;
        right: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 2px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        border-radius: 2px;
    }

    .clear-btn:hover {
        color: var(--text-normal);
        background: var(--background-modifier-hover);
    }

    /* Filter dropdown styles */
    .filter-container {
        position: relative;
    }

    .filter-badge {
        position: absolute;
        top: -2px;
        right: -2px;
        min-width: 14px;
        height: 14px;
        font-size: 9px;
        font-weight: 600;
        line-height: 14px;
        text-align: center;
        background: var(--interactive-accent);
        color: var(--text-on-accent);
        border-radius: 7px;
        padding: 0 3px;
    }

    .type-dropdown {
        position: absolute;
        top: 100%;
        left: 0;
        z-index: 100;
        min-width: 150px;
        margin-top: 4px;
        background: var(--background-primary);
        border: 1px solid var(--background-modifier-border);
        border-radius: 6px;
        box-shadow: var(--shadow-s);
        overflow: hidden;
    }

    .dropdown-item {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        padding: 6px 10px;
        border: none;
        background: transparent;
        color: var(--text-normal);
        font-size: var(--font-ui-small);
        cursor: pointer;
        text-align: left;
    }

    .dropdown-item:hover {
        background: var(--background-modifier-hover);
    }

    .dropdown-item.selected {
        background: var(--background-modifier-active-hover);
    }

    .dropdown-item.clear-item {
        color: var(--text-muted);
    }

    .dropdown-item.clear-item:not(.disabled):hover {
        color: var(--text-normal);
    }

    .dropdown-item.disabled {
        opacity: 0.5;
        cursor: default;
    }

    .dropdown-divider {
        height: 1px;
        background: var(--background-modifier-border);
        margin: 4px 0;
    }

    .dropdown-check {
        width: 14px;
        text-align: center;
        color: var(--interactive-accent);
    }

    .type-indicator {
        width: 8px;
        height: 8px;
        border-radius: 2px;
        flex-shrink: 0;
    }
</style>
