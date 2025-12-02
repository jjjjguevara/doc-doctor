<script lang="ts">
    import { onMount } from 'svelte';
    import { X } from 'lucide-svelte';
    import { filterText, setFilterText } from '../../../../stubs/stubs-store';

    let searchInput: HTMLInputElement;

    onMount(() => {
        // Focus input when component mounts
        setTimeout(() => searchInput?.focus(), 50);
    });

    const handleSearchInput = (e: Event) => {
        const target = e.target as HTMLInputElement;
        setFilterText(target.value);
    };

    const clearSearch = () => {
        setFilterText('');
        searchInput?.focus();
    };
</script>

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

<style>
    .stubs-search {
        display: flex;
        align-items: center;
        gap: 4px;
        width: 100%;
        position: relative;
    }

    .search-input {
        flex: 1;
        padding: 6px 10px;
        border: 1px solid var(--background-modifier-border);
        border-radius: 4px;
        background: var(--background-primary);
        color: var(--text-normal);
        font-size: var(--font-ui-small);
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
        right: 6px;
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
</style>
