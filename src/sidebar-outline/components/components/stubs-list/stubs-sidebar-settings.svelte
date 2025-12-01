<script lang="ts">
    import { stubsConfig } from '../../../../stubs/stubs-store';
    import { getSortedStubTypes, getNextDefaultColor } from '../../../../stubs/stubs-defaults';
    import type { StubTypeDefinition } from '../../../../stubs/stubs-types';
    import LabeledAnnotations from '../../../../main';
    import { Eye, EyeOff, Settings, Trash2, Plus, GripVertical, Undo2 } from 'lucide-svelte';

    export let plugin: LabeledAnnotations;

    $: config = $stubsConfig;
    $: sortedTypes = config ? getSortedStubTypes(config) : [];

    // Track pending deletions with undo capability
    let pendingDeletes: Map<string, { typeDef: StubTypeDefinition; timeout: ReturnType<typeof setTimeout> }> = new Map();

    // Drag state
    let draggedId: string | null = null;
    let dragOverId: string | null = null;

    function updateTypeColor(typeId: string, color: string) {
        plugin.settings.dispatch({
            type: 'STUBS_UPDATE_TYPE',
            payload: {
                id: typeId,
                updates: { color },
            },
        });
    }

    function updateTypeName(typeId: string, displayName: string) {
        plugin.settings.dispatch({
            type: 'STUBS_UPDATE_TYPE',
            payload: {
                id: typeId,
                updates: { displayName },
            },
        });
    }

    function toggleTypeVisibility(typeKey: string) {
        const currentHidden = config?.sidebar.hiddenTypes || [];
        const isHidden = currentHidden.includes(typeKey);
        const newHidden = isHidden
            ? currentHidden.filter(t => t !== typeKey)
            : [...currentHidden, typeKey];

        plugin.settings.dispatch({
            type: 'STUBS_SET_HIDDEN_TYPES',
            payload: { hiddenTypes: newHidden },
        });
    }

    function deleteType(typeId: string, typeDef: StubTypeDefinition) {
        // If already pending, confirm deletion
        if (pendingDeletes.has(typeId)) {
            const pending = pendingDeletes.get(typeId)!;
            clearTimeout(pending.timeout);
            pendingDeletes.delete(typeId);
            pendingDeletes = pendingDeletes;

            plugin.settings.dispatch({
                type: 'STUBS_DELETE_TYPE',
                payload: { id: typeId },
            });
        } else {
            // Mark as pending with auto-confirm after 5 seconds
            const timeout = setTimeout(() => {
                if (pendingDeletes.has(typeId)) {
                    pendingDeletes.delete(typeId);
                    pendingDeletes = pendingDeletes;

                    plugin.settings.dispatch({
                        type: 'STUBS_DELETE_TYPE',
                        payload: { id: typeId },
                    });
                }
            }, 5000);

            pendingDeletes.set(typeId, { typeDef, timeout });
            pendingDeletes = pendingDeletes;
        }
    }

    function undoDelete(typeId: string) {
        const pending = pendingDeletes.get(typeId);
        if (pending) {
            clearTimeout(pending.timeout);
            pendingDeletes.delete(typeId);
            pendingDeletes = pendingDeletes;
        }
    }

    function addNewType() {
        if (!config) return;

        const existingKeys = Object.values(config.stubTypes).map(t => t.key);
        let baseKey = 'custom';
        let counter = 1;
        let newKey = baseKey;

        while (existingKeys.includes(newKey)) {
            newKey = `${baseKey}-${counter}`;
            counter++;
        }

        plugin.settings.dispatch({
            type: 'STUBS_ADD_TYPE',
            payload: {
                key: newKey,
                displayName: `Custom ${counter > 1 ? counter - 1 : ''}`.trim(),
                color: getNextDefaultColor(config),
            },
        });
    }

    // Drag and drop handlers
    function handleDragStart(e: DragEvent, typeId: string) {
        draggedId = typeId;
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = 'move';
        }
    }

    function handleDragOver(e: DragEvent, typeId: string) {
        e.preventDefault();
        if (draggedId && draggedId !== typeId) {
            dragOverId = typeId;
        }
    }

    function handleDragLeave() {
        dragOverId = null;
    }

    function handleDrop(e: DragEvent, targetId: string) {
        e.preventDefault();
        if (!draggedId || draggedId === targetId) {
            draggedId = null;
            dragOverId = null;
            return;
        }

        // Get current order
        const currentOrder = sortedTypes.map(t => t.id);
        const draggedIndex = currentOrder.indexOf(draggedId);
        const targetIndex = currentOrder.indexOf(targetId);

        if (draggedIndex === -1 || targetIndex === -1) {
            draggedId = null;
            dragOverId = null;
            return;
        }

        // Reorder
        const newOrder = [...currentOrder];
        newOrder.splice(draggedIndex, 1);
        newOrder.splice(targetIndex, 0, draggedId);

        plugin.settings.dispatch({
            type: 'STUBS_REORDER_TYPES',
            payload: { orderedIds: newOrder },
        });

        draggedId = null;
        dragOverId = null;
    }

    function handleDragEnd() {
        draggedId = null;
        dragOverId = null;
    }

    $: hiddenTypes = new Set(config?.sidebar.hiddenTypes || []);
</script>

<div class="stubs-settings">
    <div class="settings-header">
        <Settings size={14} />
        <span>Stub Types</span>
        <button
            class="add-type-btn"
            on:click={addNewType}
            title="Add new stub type"
        >
            <Plus size={14} />
        </button>
    </div>

    <div class="stub-types-list">
        {#each sortedTypes as typeDef (typeDef.id)}
            {@const isPending = pendingDeletes.has(typeDef.id)}
            <div
                class="stub-type-setting"
                class:pending-delete={isPending}
                class:drag-over={dragOverId === typeDef.id}
                class:dragging={draggedId === typeDef.id}
                draggable="true"
                on:dragstart={(e) => handleDragStart(e, typeDef.id)}
                on:dragover={(e) => handleDragOver(e, typeDef.id)}
                on:dragleave={handleDragLeave}
                on:drop={(e) => handleDrop(e, typeDef.id)}
                on:dragend={handleDragEnd}
            >
                <span class="drag-handle" title="Drag to reorder">
                    <GripVertical size={12} />
                </span>
                <input
                    type="color"
                    value={typeDef.color}
                    on:change={(e) => updateTypeColor(typeDef.id, e.currentTarget.value)}
                    class="color-picker"
                    title="Change color"
                    disabled={isPending}
                />
                <input
                    type="text"
                    value={typeDef.displayName}
                    on:change={(e) => updateTypeName(typeDef.id, e.currentTarget.value)}
                    class="name-input"
                    placeholder="Type name"
                    disabled={isPending}
                />
                <button
                    class="visibility-btn"
                    class:hidden={hiddenTypes.has(typeDef.key)}
                    on:click={() => toggleTypeVisibility(typeDef.key)}
                    title={hiddenTypes.has(typeDef.key) ? 'Show type' : 'Hide type'}
                    disabled={isPending}
                >
                    {#if hiddenTypes.has(typeDef.key)}
                        <EyeOff size={14} />
                    {:else}
                        <Eye size={14} />
                    {/if}
                </button>
                {#if isPending}
                    <button
                        class="undo-btn"
                        on:click={() => undoDelete(typeDef.id)}
                        title="Undo delete"
                    >
                        <Undo2 size={14} />
                    </button>
                {/if}
                <button
                    class="delete-btn"
                    class:active={isPending}
                    on:click={() => deleteType(typeDef.id, typeDef)}
                    title={isPending ? "Confirm delete" : "Delete type definition"}
                >
                    <Trash2 size={14} />
                </button>
            </div>
        {/each}
    </div>
</div>

<style>
    .stubs-settings {
        display: flex;
        flex-direction: column;
        gap: 12px;
        width: 100%;
        padding: 8px;
        background: var(--background-secondary);
        border-radius: 6px;
    }

    .settings-header {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: var(--font-ui-small);
        font-weight: 500;
        color: var(--text-muted);
    }

    .settings-header span {
        flex: 1;
    }

    .add-type-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 2px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.15s ease;
    }

    .add-type-btn:hover {
        background: var(--background-modifier-hover);
        color: var(--text-accent);
    }

    .stub-types-list {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .stub-type-setting {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 2px;
        border-radius: 4px;
        transition: all 0.15s ease;
    }

    .stub-type-setting.pending-delete {
        opacity: 0.5;
        background: var(--background-modifier-error);
    }

    .stub-type-setting.drag-over {
        border-top: 2px solid var(--interactive-accent);
    }

    .stub-type-setting.dragging {
        opacity: 0.5;
    }

    .drag-handle {
        display: flex;
        align-items: center;
        color: var(--text-faint);
        cursor: grab;
        padding: 2px;
    }

    .drag-handle:hover {
        color: var(--text-muted);
    }

    .drag-handle:active {
        cursor: grabbing;
    }

    .color-picker {
        width: 24px;
        height: 24px;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        padding: 0;
        background: none;
    }

    .color-picker:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .color-picker::-webkit-color-swatch-wrapper {
        padding: 0;
    }

    .color-picker::-webkit-color-swatch {
        border: 1px solid var(--background-modifier-border);
        border-radius: 4px;
    }

    .name-input {
        flex: 1;
        padding: 4px 8px;
        border: 1px solid var(--background-modifier-border);
        border-radius: 4px;
        background: var(--background-primary);
        color: var(--text-normal);
        font-size: var(--font-ui-smaller);
        min-width: 0;
    }

    .name-input:focus {
        outline: none;
        border-color: var(--interactive-accent);
    }

    .name-input:disabled {
        opacity: 0.5;
        cursor: not-allowed;
        text-decoration: line-through;
    }

    .visibility-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 4px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        border-radius: 4px;
    }

    .visibility-btn:hover:not(:disabled) {
        background: var(--background-modifier-hover);
        color: var(--text-normal);
    }

    .visibility-btn.hidden {
        opacity: 0.5;
    }

    .visibility-btn:disabled {
        cursor: not-allowed;
    }

    .undo-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 4px;
        border: none;
        background: transparent;
        color: var(--text-accent);
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.15s ease;
    }

    .undo-btn:hover {
        background: var(--background-modifier-hover);
    }

    .delete-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 4px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        border-radius: 4px;
        opacity: 0.5;
        transition: all 0.15s ease;
    }

    .delete-btn:hover {
        background: var(--background-modifier-hover);
        color: var(--text-error);
        opacity: 1;
    }

    .delete-btn.active {
        color: var(--text-error);
        opacity: 1;
        background: var(--background-modifier-error);
    }
</style>
