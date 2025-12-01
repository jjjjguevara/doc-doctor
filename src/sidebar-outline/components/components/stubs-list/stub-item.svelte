<script lang="ts">
    import type { ParsedStub, StubTypeDefinition } from '../../../../stubs/stubs-types';
    import { Link, Unlink, X, Undo2 } from 'lucide-svelte';
    import { createEventDispatcher, onDestroy, onMount } from 'svelte';

    export let stub: ParsedStub;
    export let typeDef: StubTypeDefinition;
    export let isSelected: boolean = false;
    export let onClick: () => void;
    export let showTypeIndicator: boolean = false;

    const dispatch = createEventDispatcher<{
        delete: { stubId: string; anchor: string | null; confirmed: boolean };
    }>();

    let isPendingDelete = false;
    let isHovered = false;
    let confirmTimeout: ReturnType<typeof setTimeout> | null = null;
    let wrapperEl: HTMLDivElement;

    $: isLinked = stub.anchorResolved;

    // Reset pending delete when stub changes (component reused for different item)
    $: stub.id, resetPendingState();

    function resetPendingState() {
        if (isPendingDelete) {
            isPendingDelete = false;
            if (confirmTimeout) {
                clearTimeout(confirmTimeout);
                confirmTimeout = null;
            }
        }
    }

    function handleDelete(e: MouseEvent) {
        e.stopPropagation();
        if (isPendingDelete) {
            // Second click - confirm deletion
            if (confirmTimeout) {
                clearTimeout(confirmTimeout);
                confirmTimeout = null;
            }
            isPendingDelete = false;
            dispatch('delete', { stubId: stub.id, anchor: stub.anchor, confirmed: true });
        } else {
            // First click - mark as pending deletion
            isPendingDelete = true;
            // Auto-confirm after 5 seconds if not undone
            confirmTimeout = setTimeout(() => {
                if (isPendingDelete) {
                    isPendingDelete = false;
                    dispatch('delete', { stubId: stub.id, anchor: stub.anchor, confirmed: true });
                }
            }, 5000);
        }
    }

    function handleUndo(e: MouseEvent) {
        e.stopPropagation();
        isPendingDelete = false;
        if (confirmTimeout) {
            clearTimeout(confirmTimeout);
            confirmTimeout = null;
        }
    }

    function handleClickOutside(e: MouseEvent) {
        if (isPendingDelete && wrapperEl && !wrapperEl.contains(e.target as Node)) {
            // Clicked outside - cancel pending delete
            isPendingDelete = false;
            if (confirmTimeout) {
                clearTimeout(confirmTimeout);
                confirmTimeout = null;
            }
        }
    }

    onMount(() => {
        document.addEventListener('click', handleClickOutside, true);
        return () => {
            document.removeEventListener('click', handleClickOutside, true);
        };
    });

    onDestroy(() => {
        if (confirmTimeout) {
            clearTimeout(confirmTimeout);
        }
    });
</script>

<div
    bind:this={wrapperEl}
    class="stub-item-wrapper"
    class:pending-delete={isPendingDelete}
    on:mouseenter={() => isHovered = true}
    on:mouseleave={() => isHovered = false}
>
    <button
        class="stub-item"
        class:selected={isSelected}
        class:unlinked={!isLinked}
        class:pending-delete={isPendingDelete}
        on:click={onClick}
        style="--type-color: {typeDef.color}"
    >
        {#if showTypeIndicator}
            <span class="type-dot" style="background-color: {typeDef.color}" title={typeDef.displayName}></span>
        {/if}
        <span class="stub-icon">
            {#if isLinked}
                <Link size={12} />
            {:else}
                <Unlink size={12} />
            {/if}
        </span>
        <span class="stub-description" title={stub.description}>
            {stub.description}
        </span>
    </button>

    <div class="stub-actions" class:visible={isHovered || isPendingDelete}>
        {#if isPendingDelete}
            <button
                class="action-btn undo-btn"
                on:click={handleUndo}
                title="Undo delete"
            >
                <Undo2 size={12} />
            </button>
        {/if}
        <button
            class="action-btn delete-btn"
            class:active={isPendingDelete}
            on:click={handleDelete}
            title={isPendingDelete ? "Confirm delete" : "Delete stub"}
        >
            <X size={12} />
        </button>
    </div>
</div>

<style>
    .stub-item-wrapper {
        display: flex;
        align-items: center;
        width: 100%;
        max-width: 100%;
        min-width: 0;
        position: relative;
        box-sizing: border-box;
    }

    .stub-item-wrapper.pending-delete {
        opacity: 0.5;
    }

    .stub-item {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 4px 8px;
        background: transparent;
        border: none;
        border-left: 2px solid var(--type-color);
        cursor: pointer;
        border-radius: 0 4px 4px 0;
        color: var(--text-normal);
        font-size: var(--font-ui-smaller);
        text-align: left;
        flex: 1;
        min-width: 0;
        overflow: hidden;
        transition: background 0.1s ease;
    }

    .stub-item:hover {
        background: var(--background-modifier-hover);
    }

    .stub-item.selected {
        background: var(--background-modifier-active-hover);
    }

    .stub-item.unlinked {
        opacity: 0.7;
        border-left-style: dashed;
    }

    .stub-item.pending-delete {
        text-decoration: line-through;
    }

    .type-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        flex-shrink: 0;
    }

    .stub-icon {
        display: flex;
        align-items: center;
        color: var(--text-muted);
        flex-shrink: 0;
    }

    .stub-description {
        flex: 1;
        min-width: 0;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .stub-actions {
        display: flex;
        align-items: center;
        gap: 2px;
        flex-shrink: 0;
        opacity: 0;
        transition: opacity 0.15s ease;
        padding-left: 4px;
    }

    .stub-actions.visible {
        opacity: 1;
    }

    .action-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 2px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        border-radius: 3px;
        transition: all 0.1s ease;
    }

    .action-btn:hover {
        background: var(--background-modifier-hover);
        color: var(--text-normal);
    }

    .delete-btn:hover {
        color: var(--text-error);
    }

    .delete-btn.active {
        color: var(--text-error);
        background: var(--background-modifier-error);
    }

    .undo-btn:hover {
        color: var(--text-accent);
    }
</style>
