<script lang="ts">
    import { ExternalLink, AlertCircle } from 'lucide-svelte';
    import { selectedStubId, selectStub } from '../stubs-store';
    import type { ParsedStub, StubTypeDefinition } from '../stubs-types';
    import type LabeledAnnotations from '../../main';
    import { navigateToAnchor } from '../helpers/stubs-navigation';

    export let plugin: LabeledAnnotations;
    export let stub: ParsedStub;
    export let typeDef: StubTypeDefinition;

    $: isSelected = $selectedStubId === stub.id;
    $: hasAnchor = stub.anchor !== null && stub.anchorResolved;
    $: hasWarnings = stub.warnings && stub.warnings.length > 0;

    function handleClick() {
        selectStub(stub.id);

        // Navigate to anchor if it exists
        if (stub.anchor && stub.anchorResolved) {
            navigateToAnchor(plugin.app, stub.anchor);
        }
    }

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            handleClick();
        }
    }
</script>

<div
    class="stub-item"
    class:is-selected={isSelected}
    class:is-orphaned={!stub.anchorResolved && stub.anchor}
    class:has-warnings={hasWarnings}
    data-stub-id={stub.id}
    role="button"
    tabindex="0"
    on:click={handleClick}
    on:keydown={handleKeydown}
>
    <span class="stub-description">{stub.description}</span>

    <span class="stub-badges">
        {#if hasWarnings}
            <span class="warning-badge" title={stub.warnings.join('\n')}>
                <AlertCircle size={12} />
            </span>
        {/if}

        {#if hasAnchor}
            <span class="anchor-badge" title={stub.anchor}>
                <ExternalLink size={12} />
            </span>
        {:else if stub.anchor}
            <span class="orphan-badge" title="Anchor not found in document">
                ?
            </span>
        {/if}

        {#if stub.syntax === 'structured'}
            <span class="syntax-badge" title="Structured syntax">S</span>
        {/if}
    </span>
</div>

<style>
    .stub-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        padding: 6px 8px;
        border-radius: var(--radius-s);
        cursor: pointer;
        transition: background-color 0.15s ease;
        font-size: var(--font-small);
        color: var(--text-normal);
        line-height: var(--line-height-tight);
    }

    .stub-item:hover {
        background-color: var(--background-modifier-hover);
    }

    .stub-item.is-selected {
        background-color: var(--background-modifier-active-hover);
        color: var(--text-normal);
    }

    .stub-item.is-orphaned {
        opacity: 0.7;
        border-left: 2px solid var(--text-warning);
        padding-left: 6px;
    }

    .stub-item.has-warnings {
        border-left: 2px solid var(--text-warning);
        padding-left: 6px;
    }

    .stub-description {
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .stub-badges {
        display: flex;
        align-items: center;
        gap: 4px;
        flex-shrink: 0;
    }

    .anchor-badge {
        color: var(--text-muted);
        display: flex;
        align-items: center;
    }

    .orphan-badge {
        color: var(--text-warning);
        font-weight: bold;
        font-size: 10px;
    }

    .warning-badge {
        color: var(--text-warning);
        display: flex;
        align-items: center;
    }

    .syntax-badge {
        font-size: 9px;
        font-weight: bold;
        color: var(--text-muted);
        background: var(--background-modifier-hover);
        padding: 1px 4px;
        border-radius: 2px;
    }
</style>
