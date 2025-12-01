<script lang="ts">
    import { AlertTriangle, Link, Unlink } from 'lucide-svelte';
    import {
        syncState,
        orphanedStubCount,
        orphanedAnchorCount,
        hasOrphans,
    } from '../stubs-store';
    import type LabeledAnnotations from '../../main';
    import { navigateToLine } from '../helpers/stubs-navigation';

    export let plugin: LabeledAnnotations;

    function handleResolveOrphanedStub(stubId: string) {
        // Dispatch command to resolve orphaned stub
        plugin.app.commands.executeCommandById('m-stubs:resolve-orphaned-stub');
    }

    function handleResolveOrphanedAnchor(anchorId: string) {
        // Navigate to anchor location
        const anchor = $syncState.orphanedAnchors.find((a) => a.id === anchorId);
        if (anchor) {
            navigateToLine(plugin.app, anchor.position.line);
        }
    }

    function handleRemoveOrphanedAnchor(anchorId: string) {
        // Dispatch command to remove orphaned anchor
        plugin.app.commands.executeCommandById('m-stubs:remove-orphaned-anchor');
    }
</script>

{#if $hasOrphans}
    <div class="orphans-panel">
        <div class="orphans-header">
            <AlertTriangle size={14} />
            <span>Sync Issues</span>
        </div>

        {#if $orphanedStubCount > 0}
            <div class="orphan-section">
                <div class="section-header">
                    <Unlink size={12} />
                    <span>Stubs without anchors ({$orphanedStubCount})</span>
                </div>
                <div class="orphan-list">
                    {#each $syncState.orphanedStubs as stub (stub.id)}
                        <div class="orphan-item">
                            <span class="orphan-text">{stub.description}</span>
                            <button
                                class="resolve-btn"
                                on:click={() => handleResolveOrphanedStub(stub.id)}
                                title="Create anchor for this stub"
                            >
                                <Link size={12} />
                            </button>
                        </div>
                    {/each}
                </div>
            </div>
        {/if}

        {#if $orphanedAnchorCount > 0}
            <div class="orphan-section">
                <div class="section-header">
                    <Link size={12} />
                    <span>Anchors without stubs ({$orphanedAnchorCount})</span>
                </div>
                <div class="orphan-list">
                    {#each $syncState.orphanedAnchors as anchor (anchor.id)}
                        <div class="orphan-item">
                            <button
                                class="orphan-text clickable"
                                on:click={() => handleResolveOrphanedAnchor(anchor.id)}
                            >
                                {anchor.id}
                            </button>
                            <span class="line-number">Line {anchor.position.line + 1}</span>
                        </div>
                    {/each}
                </div>
            </div>
        {/if}
    </div>
{/if}

<style>
    .orphans-panel {
        display: flex;
        flex-direction: column;
        width: 100%;
        padding: 10px;
        box-sizing: border-box;
        background: var(--background-secondary);
        border-radius: var(--radius-s);
        gap: 12px;
    }

    .orphans-header {
        display: flex;
        align-items: center;
        gap: 6px;
        color: var(--text-warning);
        font-weight: var(--font-medium);
        font-size: var(--font-small);
    }

    .orphan-section {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .section-header {
        display: flex;
        align-items: center;
        gap: 4px;
        font-size: var(--font-smaller);
        color: var(--text-muted);
    }

    .orphan-list {
        display: flex;
        flex-direction: column;
        gap: 4px;
        padding-left: 16px;
    }

    .orphan-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        padding: 4px 8px;
        background: var(--background-primary);
        border-radius: var(--radius-s);
        font-size: var(--font-smaller);
    }

    .orphan-text {
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        color: var(--text-normal);
    }

    .orphan-text.clickable {
        cursor: pointer;
        background: none;
        border: none;
        text-align: left;
        padding: 0;
    }

    .orphan-text.clickable:hover {
        color: var(--text-accent);
        text-decoration: underline;
    }

    .line-number {
        color: var(--text-muted);
        font-size: var(--font-smaller);
    }

    .resolve-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 20px;
        height: 20px;
        border: none;
        border-radius: var(--radius-s);
        background: var(--interactive-accent);
        color: var(--text-on-accent);
        cursor: pointer;
        transition: opacity 0.15s ease;
    }

    .resolve-btn:hover {
        opacity: 0.8;
    }
</style>
