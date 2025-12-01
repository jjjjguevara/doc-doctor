<script lang="ts">
    import { ChevronDown, ChevronRight } from 'lucide-svelte';
    import StubItem from './stub-item.svelte';
    import { expandedTypes, toggleTypeExpanded } from '../stubs-store';
    import type { StubTypeDefinition, ParsedStub } from '../stubs-types';
    import type LabeledAnnotations from '../../main';

    export let plugin: LabeledAnnotations;
    export let typeDef: StubTypeDefinition;
    export let stubs: ParsedStub[];

    $: isExpanded = $expandedTypes.has(typeDef.key);

    function handleToggle() {
        toggleTypeExpanded(typeDef.key);
    }

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            handleToggle();
        }
    }
</script>

<div class="stub-type-group">
    <div
        class="type-header"
        role="button"
        tabindex="0"
        on:click={handleToggle}
        on:keydown={handleKeydown}
    >
        <span class="chevron">
            {#if isExpanded}
                <ChevronDown size={14} />
            {:else}
                <ChevronRight size={14} />
            {/if}
        </span>
        <span class="type-indicator" style="background-color: {typeDef.color}" />
        <span class="type-name">{typeDef.displayName}</span>
        <span class="type-count">{stubs.length}</span>
    </div>

    {#if isExpanded}
        <div class="stubs-list">
            {#each stubs as stub (stub.id)}
                <StubItem {plugin} {stub} {typeDef} />
            {/each}
        </div>
    {/if}
</div>

<style>
    .stub-type-group {
        display: flex;
        flex-direction: column;
        width: 100%;
    }

    .type-header {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 6px 8px;
        border-radius: var(--radius-s);
        cursor: pointer;
        user-select: none;
        transition: background-color 0.15s ease;
    }

    .type-header:hover {
        background-color: var(--background-modifier-hover);
    }

    .chevron {
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--text-muted);
    }

    .type-indicator {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        flex-shrink: 0;
    }

    .type-name {
        flex: 1;
        font-weight: var(--font-medium);
        font-size: var(--font-small);
        color: var(--text-normal);
    }

    .type-count {
        font-size: var(--font-smaller);
        color: var(--text-muted);
        background: var(--background-modifier-hover);
        padding: 2px 6px;
        border-radius: var(--radius-s);
    }

    .stubs-list {
        display: flex;
        flex-direction: column;
        padding-left: 20px;
        gap: 2px;
        margin-top: 4px;
    }
</style>
