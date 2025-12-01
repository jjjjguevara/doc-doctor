<script lang="ts">
    import { onDestroy } from 'svelte';
    import StubTypeGroup from './stub-type-group.svelte';
    import NoStubs from './no-stubs.svelte';
    import {
        visibleStubsByType,
        stubsConfig,
        selectedStubId,
        filterText,
    } from '../stubs-store';
    import { getSortedStubTypes } from '../stubs-defaults';
    import type LabeledAnnotations from '../../main';

    export let plugin: LabeledAnnotations;

    let listRef: HTMLDivElement;

    // Auto-scroll to selected stub
    const unsub = selectedStubId.subscribe((stubId) => {
        if (listRef && stubId) {
            const activeElement = listRef.querySelector(`[data-stub-id="${stubId}"]`);
            if (activeElement) {
                activeElement.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
            }
        }
    });

    onDestroy(unsub);

    // Get sorted type definitions
    $: sortedTypes = $stubsConfig ? getSortedStubTypes($stubsConfig) : [];

    // Check if we have any visible stubs
    $: hasVisibleStubs = [...$visibleStubsByType.values()].some((stubs) => stubs.length > 0);
</script>

<div class="stubs-list-container" bind:this={listRef}>
    {#if hasVisibleStubs}
        {#each sortedTypes as typeDef (typeDef.id)}
            {@const stubs = $visibleStubsByType.get(typeDef.key) || []}
            {#if stubs.length > 0}
                <StubTypeGroup {plugin} {typeDef} {stubs} />
            {/if}
        {/each}
    {:else if $filterText}
        <NoStubs variant="filter-applied" />
    {:else}
        <NoStubs />
    {/if}
</div>

<style>
    .stubs-list-container {
        display: flex;
        flex-direction: column;
        width: 100%;
        gap: 8px;
        overflow-y: auto;
        box-sizing: border-box;
        padding: 0 10px;
    }
</style>
