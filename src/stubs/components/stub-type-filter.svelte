<script lang="ts">
    import {
        stubsConfig,
        hiddenTypes,
        countByType,
        toggleTypeVisibility,
    } from '../stubs-store';
    import { getSortedStubTypes } from '../stubs-defaults';

    $: sortedTypes = $stubsConfig ? getSortedStubTypes($stubsConfig) : [];
</script>

<div class="type-filter">
    <div class="filter-header">
        <span>Filter by type</span>
    </div>
    <div class="filter-list">
        {#each sortedTypes as typeDef (typeDef.id)}
            {@const count = $countByType.get(typeDef.key) || 0}
            {@const isHidden = $hiddenTypes.has(typeDef.key)}
            <button
                class="filter-item"
                class:is-hidden={isHidden}
                on:click={() => toggleTypeVisibility(typeDef.key)}
            >
                <span class="type-indicator" style="background-color: {typeDef.color}" />
                <span class="type-name">{typeDef.displayName}</span>
                <span class="type-count">{count}</span>
            </button>
        {/each}
    </div>
</div>

<style>
    .type-filter {
        display: flex;
        flex-direction: column;
        gap: 8px;
        width: 100%;
    }

    .filter-header {
        font-size: var(--font-smaller);
        color: var(--text-muted);
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .filter-list {
        display: flex;
        flex-wrap: wrap;
        gap: 4px;
    }

    .filter-item {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 4px 8px;
        border: 1px solid var(--background-modifier-border);
        border-radius: var(--radius-s);
        background: var(--background-primary);
        color: var(--text-normal);
        font-size: var(--font-smaller);
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .filter-item:hover {
        background: var(--background-modifier-hover);
    }

    .filter-item.is-hidden {
        opacity: 0.5;
        background: var(--background-secondary);
    }

    .type-indicator {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        flex-shrink: 0;
    }

    .type-name {
        flex: 1;
    }

    .type-count {
        color: var(--text-muted);
        font-size: var(--font-smaller);
    }
</style>
