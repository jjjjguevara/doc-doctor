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
    import { removeStubFromFrontmatter, removeStubFromFrontmatterByContent, performSync } from '../../../../stubs/helpers/stubs-sync';
    import { removeAnchorFromContent } from '../../../../stubs/helpers/anchor-utils';
    import LabeledAnnotations from '../../../../main';
    import type { ParsedStub, StubTypeDefinition } from '../../../../stubs/stubs-types';
    import { ChevronRight, ChevronDown, AlertTriangle } from 'lucide-svelte';
    import { MarkdownView, WorkspaceLeaf, Notice, setIcon } from 'obsidian';
    import NoStubs from './no-stubs.svelte';
    import StubItem from './stub-item.svelte';
    import { controls } from '../controls-bar/controls-bar.store';
    import { setRemediateStub } from '../../../../stubs/llm-analysis-store';

    export let plugin: LabeledAnnotations;

    $: config = $stubsConfig;
    $: sortedTypes = config ? getSortedStubTypes(config) : [];
    $: stubsByType = $visibleStubsByType;
    $: flatStubs = $sortedVisibleStubs;
    // Type view includes: 'type', 'type-asc', 'type-desc'
    $: isTypeView = $stubSortOrder === 'type' || $stubSortOrder === 'type-asc' || $stubSortOrder === 'type-desc';
    $: hasStubs = $syncState.stubs.length > 0;
    $: hasOrphans = $syncState.orphanedStubs.length > 0 || $syncState.orphanedAnchors.length > 0;

    // Drag and drop state
    let draggedTypeId: string | null = null;
    let isDraggable = false;
    let longPressTimer: ReturnType<typeof setTimeout> | null = null;
    let dragStartPos = { x: 0, y: 0 };
    let dragOverTypeId: string | null = null;
    let dropPosition: 'before' | 'after' | null = null;

    const LONG_PRESS_DELAY = 40; // ms before drag becomes active (nearly instant but allows click)
    const MOVE_THRESHOLD = 2; // px movement allowed during long-press

    // Set icon on element using Obsidian's setIcon
    function setTypeIcon(el: HTMLElement, typeDef: StubTypeDefinition) {
        if (typeDef.icon) {
            try {
                setIcon(el, typeDef.icon);
            } catch {
                el.textContent = '';
            }
        }
    }

    // Long-press to enable dragging
    function handleMouseDown(e: MouseEvent, typeId: string) {
        const target = e.currentTarget as HTMLElement;
        dragStartPos = { x: e.clientX, y: e.clientY };

        longPressTimer = setTimeout(() => {
            isDraggable = true;
            draggedTypeId = typeId;
            target.setAttribute('draggable', 'true');
            // Apply inline style for immediate visual feedback
            target.style.cursor = 'grabbing';
            target.style.background = 'var(--background-modifier-active-hover)';
        }, LONG_PRESS_DELAY);
    }

    function handleMouseMove(e: MouseEvent) {
        // Cancel long-press if mouse moves too much before drag is enabled
        if (longPressTimer && !isDraggable) {
            const dx = Math.abs(e.clientX - dragStartPos.x);
            const dy = Math.abs(e.clientY - dragStartPos.y);
            if (dx > MOVE_THRESHOLD || dy > MOVE_THRESHOLD) {
                clearTimeout(longPressTimer);
                longPressTimer = null;
            }
        }
    }

    function handleMouseUp(e: MouseEvent) {
        if (longPressTimer) {
            clearTimeout(longPressTimer);
            longPressTimer = null;
        }
        // Reset draggable state
        setTimeout(() => {
            resetDragState();
        }, 50);
    }

    function handleMouseLeave(e: MouseEvent) {
        if (longPressTimer && !isDraggable) {
            clearTimeout(longPressTimer);
            longPressTimer = null;
        }
    }

    function resetDragState() {
        isDraggable = false;
        draggedTypeId = null;
        dragOverTypeId = null;
        dropPosition = null;
        // Reset all inline styles
        document.querySelectorAll('.stub-type-header').forEach(el => {
            (el as HTMLElement).style.cursor = '';
            (el as HTMLElement).style.background = '';
            el.setAttribute('draggable', 'false');
        });
        document.querySelectorAll('.stub-type-group').forEach(el => {
            (el as HTMLElement).style.opacity = '';
            (el as HTMLElement).style.transform = '';
            (el as HTMLElement).style.borderTop = '';
            (el as HTMLElement).style.borderBottom = '';
            (el as HTMLElement).style.background = '';
        });
    }

    // Drag handlers for type groups
    function handleDragStart(e: DragEvent, typeId: string) {
        if (!isDraggable) {
            e.preventDefault();
            return;
        }
        draggedTypeId = typeId;
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = 'move';
            e.dataTransfer.setData('text/plain', typeId);
        }
        // Apply inline style to dragged element
        const group = (e.currentTarget as HTMLElement).closest('.stub-type-group') as HTMLElement;
        if (group) {
            group.style.opacity = '0.4';
            group.style.transform = 'scale(0.98)';
        }
    }

    function handleDragEnd(e: DragEvent) {
        resetDragState();
    }

    function handleDragOver(e: DragEvent, typeId: string) {
        e.preventDefault();
        if (!draggedTypeId || draggedTypeId === typeId) {
            // Clear styles if hovering over self or no drag
            if (dragOverTypeId === typeId) {
                const target = e.currentTarget as HTMLElement;
                target.style.borderTop = '';
                target.style.borderBottom = '';
                target.style.background = '';
                dragOverTypeId = null;
            }
            return;
        }

        const target = e.currentTarget as HTMLElement;
        const rect = target.getBoundingClientRect();
        const midY = rect.top + rect.height / 2;

        // Determine position
        const newDropPosition = e.clientY < midY ? 'before' : 'after';

        // Check if this would be a no-op (adjacent positions)
        const sourceIndex = sortedTypes.findIndex(t => t.key === draggedTypeId);
        const targetIndex = sortedTypes.findIndex(t => t.key === typeId);
        const isNoOp = (
            (newDropPosition === 'before' && targetIndex === sourceIndex + 1) ||
            (newDropPosition === 'after' && targetIndex === sourceIndex - 1)
        );

        // Clear previous target styles
        if (dragOverTypeId && dragOverTypeId !== typeId) {
            document.querySelectorAll('.stub-type-group').forEach(el => {
                (el as HTMLElement).style.borderTop = '';
                (el as HTMLElement).style.borderBottom = '';
                (el as HTMLElement).style.background = '';
            });
        }

        dragOverTypeId = typeId;
        dropPosition = newDropPosition;

        // Apply inline styles for insertion indicator (skip if no-op)
        if (isNoOp) {
            target.style.borderTop = '';
            target.style.borderBottom = '';
            target.style.background = '';
        } else {
            target.style.background = 'var(--background-modifier-hover)';
            if (newDropPosition === 'before') {
                target.style.borderTop = '3px solid var(--interactive-accent)';
                target.style.borderBottom = '';
            } else {
                target.style.borderTop = '';
                target.style.borderBottom = '3px solid var(--interactive-accent)';
            }
        }
    }

    function handleDragLeave(e: DragEvent) {
        const target = e.currentTarget as HTMLElement;
        // Only clear if actually leaving (not entering a child)
        const relatedTarget = e.relatedTarget as HTMLElement;
        if (!target.contains(relatedTarget)) {
            target.style.borderTop = '';
            target.style.borderBottom = '';
            target.style.background = '';
            if (dragOverTypeId === target.dataset.typeKey) {
                dragOverTypeId = null;
            }
        }
    }

    function handleDrop(e: DragEvent, targetTypeId: string) {
        e.preventDefault();

        if (!draggedTypeId || draggedTypeId === targetTypeId) {
            resetDragState();
            return;
        }

        // Find the type definitions and their indices
        const sourceIndex = sortedTypes.findIndex(t => t.key === draggedTypeId);
        const targetIndex = sortedTypes.findIndex(t => t.key === targetTypeId);
        const sourceType = sortedTypes[sourceIndex];
        const targetType = sortedTypes[targetIndex];

        if (!sourceType || !targetType || sourceIndex === -1 || targetIndex === -1) {
            resetDragState();
            return;
        }

        // Check if this would result in no change (adjacent positions)
        // - Dropping "before" target when target is immediately below source = no change
        // - Dropping "after" target when target is immediately above source = no change
        const isNoOp = (
            (dropPosition === 'before' && targetIndex === sourceIndex + 1) ||
            (dropPosition === 'after' && targetIndex === sourceIndex - 1)
        );

        if (isNoOp) {
            resetDragState();
            return;
        }

        // Dispatch reorder action
        plugin.settings.dispatch({
            type: 'STUBS_REORDER_TYPES',
            payload: { sourceId: sourceType.id, targetId: targetType.id },
        });

        resetDragState();
    }

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
            if (anchor) {
                // Stub has anchor - remove by anchor ID
                await removeStubFromFrontmatter(app, file, anchor, config);

                // Remove inline anchor from content
                const content = await app.vault.read(file);
                const newContent = removeAnchorFromContent(content, anchor);
                if (newContent !== content) {
                    await app.vault.modify(file, newContent);
                }
            } else {
                // Stub has no anchor - find by ID and remove by type+description
                const stub = $syncState.stubs.find(s => s.id === stubId);
                if (stub) {
                    await removeStubFromFrontmatterByContent(app, file, stub.type, stub.description, config);
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

    function handleStubGoto(event: CustomEvent<{ stubId: string; anchor: string | null }>) {
        const { stubId, anchor } = event.detail;
        if (anchor) {
            selectStub(stubId);
            setFocusLocation('inline');
            navigateToStub(plugin.app, stubId);
        }
    }

    function handleStubCopy(event: CustomEvent<{ stubId: string; description: string }>) {
        const { description } = event.detail;
        navigator.clipboard.writeText(description).then(() => {
            new Notice('Description copied to clipboard');
        }).catch((err) => {
            console.error('Failed to copy:', err);
            new Notice('Failed to copy to clipboard');
        });
    }

    function handleStubSelect(event: CustomEvent<{ stubId: string; anchor: string | null }>) {
        const { stubId } = event.detail;
        selectStub(stubId);
        // Navigate to frontmatter to show the stub definition
        setFocusLocation('frontmatter');
        navigateToStubFrontmatter(plugin.app, stubId);
    }

    async function handleStubUnlink(event: CustomEvent<{ stubId: string; anchor: string | null }>) {
        const { stubId, anchor } = event.detail;
        if (!anchor) return;

        const view = getMarkdownView();
        if (!view || !view.file || !config) {
            console.error('No active markdown view or file found');
            return;
        }

        const file = view.file;

        try {
            // Use MCP if connected
            const mcpTools = plugin.getMCPTools();
            let content = await plugin.app.vault.read(file);

            if (mcpTools && plugin.isMCPConnected()) {
                // Find stub index
                const stubIndex = $syncState.stubs.findIndex(s => s.id === stubId);
                if (stubIndex >= 0) {
                    try {
                        const result = await mcpTools.unlinkStubAnchor(content, stubIndex, anchor);
                        content = result.updated_content;
                        await plugin.app.vault.modify(file, content);
                        new Notice('Anchor unlinked from stub');
                    } catch (mcpError) {
                        console.warn('MCP unlink failed:', mcpError);
                        // Fall back to removing anchor only
                        const newContent = removeAnchorFromContent(content, anchor);
                        await plugin.app.vault.modify(file, newContent);
                        new Notice('Anchor removed (manual fallback)');
                    }
                }
            } else {
                // Manual: just remove the anchor from content
                const newContent = removeAnchorFromContent(content, anchor);
                await plugin.app.vault.modify(file, newContent);
                new Notice('Anchor removed');
            }

            // Resync state
            const newContent = await plugin.app.vault.read(file);
            const newState = await performSync(plugin.app, file, newContent, config);
            updateSyncState(newState);
        } catch (error) {
            console.error('Failed to unlink stub:', error);
            new Notice('Failed to unlink anchor');
        }
    }

    function handleStubRemediate(event: CustomEvent<{ stubId: string; stub: ParsedStub }>) {
        const { stub } = event.detail;

        // Store the stub info for the AIView to pick up
        setRemediateStub(stub);

        // Switch to AIView with animation
        controls.dispatch({ type: 'SET_VIEW_MODE', payload: 'ai' });

        // Save view mode to settings
        plugin.settings.dispatch({ type: 'SET_SIDEBAR_VIEW_MODE', payload: { mode: 'ai' } });
    }
</script>

<div class="stubs-list-container">
    {#if hasStubs || $filterText}
        {#if isTypeView}
            <!-- Grouped by type view -->
            {#each sortedTypes as typeDef}
                {@const stubs = stubsByType.get(typeDef.key) || []}
                {#if stubs.length > 0 || (config && config.sidebar.showEmptyGroups)}
                    <div
                        class="stub-type-group"
                        on:dragover={(e) => handleDragOver(e, typeDef.key)}
                        on:dragleave={handleDragLeave}
                        on:drop={(e) => handleDrop(e, typeDef.key)}
                    >
                        <div class="stub-type-header-row">
                            <button
                                class="stub-type-header"
                                on:click={() => handleToggleType(typeDef.key)}
                                on:mousedown={(e) => handleMouseDown(e, typeDef.key)}
                                on:mouseup={handleMouseUp}
                                on:mousemove={handleMouseMove}
                                on:mouseleave={handleMouseLeave}
                                on:dragstart={(e) => handleDragStart(e, typeDef.key)}
                                on:dragend={handleDragEnd}
                                style="--type-color: {typeDef.color}"
                                title="Click to expand/collapse. Hold to drag."
                            >
                                <span class="type-indicator" style="background-color: {typeDef.color}"></span>
                                {#if typeDef.icon}
                                    <span class="type-icon" use:setTypeIcon={typeDef}></span>
                                {/if}
                                <span class="type-name">{typeDef.displayName}</span>
                                <span class="type-chevron">
                                    {#if $expandedTypes.has(typeDef.key)}
                                        <ChevronDown size={14} />
                                    {:else}
                                        <ChevronRight size={14} />
                                    {/if}
                                </span>
                                <span class="type-count">{stubs.length}</span>
                            </button>
                        </div>

                        {#if $expandedTypes.has(typeDef.key)}
                            <div class="stub-type-items">
                                {#each stubs as stub (stub.id)}
                                    <StubItem
                                        {stub}
                                        {typeDef}
                                        isSelected={$selectedStubId === stub.id}
                                        onClick={() => handleStubClick(stub.id)}
                                        on:delete={handleStubDelete}
                                        on:goto={handleStubGoto}
                                        on:copy={handleStubCopy}
                                        on:select={handleStubSelect}
                                        on:unlink={handleStubUnlink}
                                        on:remediate={handleStubRemediate}
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
                        on:goto={handleStubGoto}
                        on:copy={handleStubCopy}
                        on:select={handleStubSelect}
                        on:unlink={handleStubUnlink}
                        on:remediate={handleStubRemediate}
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
        border-radius: 4px;
        transition: all 0.15s ease;
    }

    .stub-type-group.dragging {
        opacity: 0.4;
        transform: scale(0.98);
    }

    .stub-type-group.drag-over-above,
    .stub-type-group.drag-over-below {
        position: relative;
    }

    .stub-type-group.drag-over-above::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 2px;
        background: var(--interactive-accent);
        border-radius: 1px;
        z-index: 10;
    }

    .stub-type-group.drag-over-below::after {
        content: '';
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 2px;
        background: var(--interactive-accent);
        border-radius: 1px;
        z-index: 10;
    }

    .stub-type-group.drag-over-above .stub-type-header,
    .stub-type-group.drag-over-below .stub-type-header {
        background: var(--background-modifier-hover);
    }

    .stub-type-header-row {
        display: flex;
        align-items: center;
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
        flex: 1;
        transition: background 0.15s ease;
    }

    .stub-type-header:hover {
        background: var(--background-modifier-hover);
    }

    .stub-type-header.drag-ready,
    .stub-type-header:global(.drag-ready) {
        cursor: grabbing;
        background: var(--background-modifier-active-hover);
    }

    .type-indicator {
        width: 8px;
        height: 8px;
        border-radius: 2px;
        flex-shrink: 0;
    }

    .type-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 16px;
        height: 16px;
        color: var(--text-muted);
        flex-shrink: 0;
    }

    .type-icon :global(svg) {
        width: 14px;
        height: 14px;
    }

    .type-name {
        flex: 1;
        font-weight: 500;
    }

    .type-chevron {
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--text-muted);
        flex-shrink: 0;
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
