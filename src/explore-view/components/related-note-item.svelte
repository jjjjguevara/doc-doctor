<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { FileText, ExternalLink, Plus, Cpu, Type } from 'lucide-svelte';
    import type { RelatedNote } from '../../smart-connections/types';

    export let note: RelatedNote;

    const dispatch = createEventDispatcher<{
        click: void;
        newpane: void;
        addrelated: void;
    }>();

    $: similarityPercent = Math.round(note.similarity * 100);
    $: similarityClass = similarityPercent >= 80 ? 'high' : similarityPercent >= 60 ? 'medium' : 'low';

    function handleClick(e: MouseEvent) {
        if (e.metaKey || e.ctrlKey) {
            dispatch('newpane');
        } else {
            dispatch('click');
        }
    }

    function handleAddRelated(e: MouseEvent) {
        e.stopPropagation();
        dispatch('addrelated');
    }

    function handleNewPane(e: MouseEvent) {
        e.stopPropagation();
        dispatch('newpane');
    }
</script>

<button
    class="related-note-item"
    on:click={handleClick}
    title="{note.title}\n\n{note.excerpt}\n\nSimilarity: {similarityPercent}%\nMethod: {note.method}"
>
    <div class="note-icon">
        <FileText size={14} />
    </div>

    <div class="note-content">
        <div class="note-title">{note.title}</div>
        {#if note.excerpt}
            <div class="note-excerpt">{note.excerpt}</div>
        {/if}
        {#if note.matchedKeywords && note.matchedKeywords.length > 0}
            <div class="note-keywords">
                {#each note.matchedKeywords.slice(0, 3) as keyword}
                    <span class="keyword">{keyword}</span>
                {/each}
            </div>
        {/if}
    </div>

    <div class="note-meta">
        <div class="similarity {similarityClass}">
            {similarityPercent}%
        </div>
        <div class="method-indicator" title="{note.method === 'embedding' ? 'Semantic (embedding)' : 'Keyword-based'}">
            {#if note.method === 'embedding'}
                <Cpu size={10} />
            {:else}
                <Type size={10} />
            {/if}
        </div>
    </div>

    <div class="note-actions">
        <button
            class="action-btn"
            on:click={handleAddRelated}
            title="Add to related property"
        >
            <Plus size={12} />
        </button>
        <button
            class="action-btn"
            on:click={handleNewPane}
            title="Open in new pane"
        >
            <ExternalLink size={12} />
        </button>
    </div>
</button>

<style>
    .related-note-item {
        display: flex;
        align-items: flex-start;
        gap: 8px;
        width: 100%;
        padding: 8px;
        border: none;
        background: var(--background-primary);
        cursor: pointer;
        border-radius: 6px;
        text-align: left;
        transition: background 0.1s ease;
    }

    .related-note-item:hover {
        background: var(--background-modifier-hover);
    }

    .note-icon {
        flex-shrink: 0;
        color: var(--text-muted);
        padding-top: 2px;
    }

    .note-content {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .note-title {
        font-size: var(--font-ui-small);
        font-weight: 500;
        color: var(--text-normal);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .note-excerpt {
        font-size: var(--font-ui-smaller);
        color: var(--text-muted);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .note-keywords {
        display: flex;
        gap: 4px;
        flex-wrap: wrap;
        margin-top: 2px;
    }

    .keyword {
        font-size: 10px;
        padding: 1px 4px;
        background: var(--background-modifier-border);
        color: var(--text-faint);
        border-radius: 3px;
    }

    .note-meta {
        flex-shrink: 0;
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 4px;
    }

    .similarity {
        font-size: var(--font-ui-smaller);
        font-weight: 600;
        padding: 2px 6px;
        border-radius: 4px;
    }

    .similarity.high {
        background: rgba(46, 204, 113, 0.2);
        color: var(--color-green);
    }

    .similarity.medium {
        background: rgba(241, 196, 15, 0.2);
        color: var(--color-yellow);
    }

    .similarity.low {
        background: var(--background-modifier-border);
        color: var(--text-muted);
    }

    .method-indicator {
        color: var(--text-faint);
        opacity: 0.7;
    }

    .note-actions {
        flex-shrink: 0;
        display: flex;
        flex-direction: column;
        gap: 2px;
        opacity: 0;
        transition: opacity 0.15s ease;
    }

    .related-note-item:hover .note-actions {
        opacity: 1;
    }

    .action-btn {
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

    .action-btn:hover {
        background: var(--background-modifier-hover);
        color: var(--text-normal);
    }
</style>
