<script lang="ts">
    import { searchTerm } from './search-input.store';
    import { l } from '../../../../../lang/lang';
    import { X } from 'lucide-svelte';
    import { onMount } from 'svelte';

    let searchInput: HTMLInputElement;

    onMount(() => {
        setTimeout(() => searchInput?.focus(), 50);
    });

    const handleSearchInput = (e: Event) => {
        const target = e.target as HTMLInputElement;
        searchTerm.set(target.value);
    };

    const clearSearch = () => {
        searchTerm.set('');
        searchInput?.focus();
    };
</script>

<div class="annotations-search">
    <input
        bind:this={searchInput}
        type="text"
        placeholder={l.OUTLINE_SEARCH_ANNOTATIONS}
        value={$searchTerm}
        on:input={handleSearchInput}
        class="search-input"
    />
    {#if $searchTerm}
        <button class="clear-btn" on:click={clearSearch} title="Clear search">
            <X size={12} />
        </button>
    {/if}
</div>

<style>
    .annotations-search {
        display: flex;
        align-items: center;
        gap: 4px;
        width: 100%;
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
</style>
