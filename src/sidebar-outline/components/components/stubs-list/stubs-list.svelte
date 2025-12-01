<script lang="ts">
    import {
        syncState,
        stubsConfig,
        visibleStubsByType,
        sortedVisibleStubs,
        stubSortOrder,
        filterText,
        expandedTypes,
        toggleTypeExpanded,
        selectStub,
        selectedStubId,
        updateSyncState,
        stubFocusLocation,
        toggleFocusLocation,
        setFocusLocation,
    } from '../../../../stubs/stubs-store';
    import { getSortedStubTypes } from '../../../../stubs/stubs-defaults';
    import { navigateToStub, navigateToOrphanedStub, navigateToStubFrontmatter } from '../../../../stubs/helpers/stubs-navigation';
    import { removeStubFromFrontmatter, performSync } from '../../../../stubs/helpers/stubs-sync';
    import { removeAnchorFromContent } from '../../../../stubs/helpers/anchor-utils';
    import LabeledAnnotations from '../../../../main';
    import { ChevronRight, ChevronDown, AlertTriangle } from 'lucide-svelte';
    import { MarkdownView, WorkspaceLeaf } from 'obsidian';
    import NoStubs from './no-stubs.svelte';
    import StubItem from './stub-item.svelte';

    export let plugin: LabeledAnnotations;

    $: config = $stubsConfig;
    $: sortedTypes = config ? getSortedStubTypes(config) : [];
    $: stubsByType = $visibleStubsByType;
    $: flatStubs = $sortedVisibleStubs;
    $: isTypeView = $stubSortOrder === 'type';
    $: hasStubs = $syncState.stubs.length > 0;
    $: hasOrphans = $syncState.orphanedStubs.length > 0 || $syncState.orphanedAnchors.length > 0;

    // Get type definition for a stub (for flat view)
    function getTypeDef(stubType: string) {
        return sortedTypes.find(t => t.key === stubType) || {
            key: stubType,
            displayName: stubType,
            color: '#888',
            sortOrder: 999,
            id: stubType
        };
    }

    /**
     * Get markdown view even when sidebar is focused
     */
    function getMarkdownView() {
        const app = plugin.app;
        const activeView = app.workspace.getActiveViewOfType(MarkdownView);
        if (activeView) return activeView;

        // If sidebar is focused, find the markdown view from the active file
        const activeFile = app.workspace.getActiveFile();
        if (!activeFile) return null;

        let markdownView: MarkdownView | null = null;
        app.workspace.iterateAllLeaves((leaf: WorkspaceLeaf) => {
            if (markdownView) return;
            const view = leaf.view;
            if (view instanceof MarkdownView && view.file?.path === activeFile.path) {
                markdownView = view;
            }
        });
        return markdownView;
    }

    function handleStubClick(stubId: string) {
        const stub = $syncState.stubs.find(s => s.id === stubId);
        const isCurrentlySelected = $selectedStubId === stubId;

        // If this is an orphaned stub (no anchor resolved), always go to frontmatter
        if (stub && !stub.anchorResolved) {
            selectStub(stubId);
            setFocusLocation('frontmatter');
            navigateToOrphanedStub(plugin.app, stub.anchor || stubId);
            return;
        }

        // For resolved stubs, implement cycling behavior
        if (isCurrentlySelected) {
            // Clicking same stub again - toggle focus location
            toggleFocusLocation();
            if ($stubFocusLocation === 'frontmatter') {
                // We just toggled TO frontmatter (it was inline before toggle, now it's frontmatter after)
                navigateToStubFrontmatter(plugin.app, stubId);
            } else {
                // We just toggled TO inline
                navigateToStub(plugin.app, stubId);
            }
        } else {
            // New stub selected - start at inline anchor
            selectStub(stubId);
            setFocusLocation('inline');
            navigateToStub(plugin.app, stubId);
        }
    }

    function handleToggleType(typeKey: string) {
        toggleTypeExpanded(typeKey);
    }

    async function handleStubDelete(event: CustomEvent<{ stubId: string; anchor: string | null; confirmed: boolean }>) {
        const { stubId, anchor, confirmed } = event.detail;

        // Only proceed if deletion is confirmed
        if (!confirmed) return;

        const app = plugin.app;

        // Get markdown view (works even when sidebar is focused)
        const view = getMarkdownView();
        if (!view || !view.file || !config) {
            console.error('No active markdown view or file found');
            return;
        }

        const file = view.file;

        try {
            // Remove from frontmatter
            if (anchor) {
                await removeStubFromFrontmatter(app, file, anchor, config);
            }

            // Remove inline anchor from content
            if (anchor) {
                const content = await app.vault.read(file);
                const newContent = removeAnchorFromContent(content, anchor);
                if (newContent !== content) {
                    await app.vault.modify(file, newContent);
                }
            }

            // Resync state
            const content = await app.vault.read(file);
            const newState = await performSync(app, file, content, config);
            updateSyncState(newState);

        } catch (error) {
            console.error('Failed to delete stub:', error);
        }
    }
</script>

<div class="stubs-list-container">
    {#if hasStubs || $filterText}
        {#if isTypeView}
            <!-- Grouped by type view -->
            {#each sortedTypes as typeDef}
                {@const stubs = stubsByType.get(typeDef.key) || []}
                {#if stubs.length > 0 || (config && config.sidebar.showEmptyGroups)}
                    <div class="stub-type-group">
                        <button
                            class="stub-type-header"
                            on:click={() => handleToggleType(typeDef.key)}
                            style="--type-color: {typeDef.color}"
                        >
                            <span class="type-indicator" style="background-color: {typeDef.color}"></span>
                            {#if $expandedTypes.has(typeDef.key)}
                                <ChevronDown size={14} />
                            {:else}
                                <ChevronRight size={14} />
                            {/if}
                            <span class="type-name">{typeDef.displayName}</span>
                            <span class="type-count">{stubs.length}</span>
                        </button>

                        {#if $expandedTypes.has(typeDef.key)}
                            <div class="stub-type-items">
                                {#each stubs as stub (stub.id)}
                                    <StubItem
                                        {stub}
                                        {typeDef}
                                        isSelected={$selectedStubId === stub.id}
                                        onClick={() => handleStubClick(stub.id)}
                                        on:delete={handleStubDelete}
                                    />
                                {/each}
                            </div>
                        {/if}
                    </div>
                {/if}
            {/each}
        {:else}
            <!-- Flat list sorted by position -->
            <div class="stub-flat-list">
                {#each flatStubs as stub (stub.id)}
                    {@const typeDef = getTypeDef(stub.type)}
                    <StubItem
                        {stub}
                        {typeDef}
                        isSelected={$selectedStubId === stub.id}
                        onClick={() => handleStubClick(stub.id)}
                        on:delete={handleStubDelete}
                        showTypeIndicator={true}
                    />
                {/each}
            </div>
        {/if}

        {#if hasOrphans}
            <div class="orphans-section">
                <div class="orphans-header">
                    <AlertTriangle size={14} />
                    <span>Orphans</span>
                </div>
                {#if $syncState.orphanedStubs.length > 0}
                    <div class="orphan-group">
                        <span class="orphan-label">Stubs without anchors: {$syncState.orphanedStubs.length}</span>
                    </div>
                {/if}
                {#if $syncState.orphanedAnchors.length > 0}
                    <div class="orphan-group">
                        <span class="orphan-label">Anchors without stubs: {$syncState.orphanedAnchors.length}</span>
                    </div>
                {/if}
            </div>
        {/if}
    {:else}
        <NoStubs />
    {/if}
</div>

<style>
    .stubs-list-container {
        display: flex;
        flex-direction: column;
        width: 100%;
        max-width: 100%;
        gap: 4px;
        overflow-y: auto;
        overflow-x: hidden;
        box-sizing: border-box;
        padding: 0 10px;
    }

    .stub-type-group {
        display: flex;
        flex-direction: column;
    }

    .stub-type-header {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 4px 8px;
        background: transparent;
        border: none;
        cursor: pointer;
        border-radius: 4px;
        color: var(--text-normal);
        font-size: var(--font-ui-small);
        text-align: left;
        width: 100%;
    }

    .stub-type-header:hover {
        background: var(--background-modifier-hover);
    }

    .type-indicator {
        width: 8px;
        height: 8px;
        border-radius: 2px;
        flex-shrink: 0;
    }

    .type-name {
        flex: 1;
        font-weight: 500;
    }

    .type-count {
        color: var(--text-muted);
        font-size: var(--font-ui-smaller);
        background: var(--background-modifier-border);
        padding: 0 6px;
        border-radius: 10px;
    }

    .stub-type-items {
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding-left: 20px;
        margin-top: 2px;
        min-width: 0;
        overflow: hidden;
    }

    .stub-flat-list {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
        overflow: hidden;
    }

    .orphans-section {
        margin-top: 12px;
        padding-top: 12px;
        border-top: 1px solid var(--background-modifier-border);
    }

    .orphans-header {
        display: flex;
        align-items: center;
        gap: 6px;
        color: var(--text-warning);
        font-size: var(--font-ui-small);
        font-weight: 500;
        margin-bottom: 8px;
    }

    .orphan-group {
        padding: 4px 8px;
        font-size: var(--font-ui-smaller);
        color: var(--text-muted);
    }

    .orphan-label {
        color: var(--text-muted);
    }
</style>
