<script lang="ts">
    export let variant: 'empty' | 'filter-applied' = 'empty';
</script>

<div class="no-stubs">
    {#if variant === 'filter-applied'}
        <span class="message">No stubs match your filter</span>
        <span class="hint">Try adjusting your search or filter settings</span>
    {:else}
        <span class="message">No stubs in this document</span>
        <span class="hint">Add stubs to your frontmatter or use the + button</span>
    {/if}
</div>

<style>
    .no-stubs {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        width: 100%;
        padding: 40px 20px;
        text-align: center;
        gap: 8px;
    }

    .message {
        font-size: var(--font-small);
        color: var(--text-muted);
    }

    .hint {
        font-size: var(--font-smaller);
        color: var(--text-faint);
    }
</style>
