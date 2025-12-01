<script lang="ts">
    import { Bookmark } from 'lucide-svelte';
</script>

<div class="no-stubs">
    <div class="no-stubs-icon">
        <Bookmark size={24} />
    </div>
    <p class="no-stubs-text">No stubs in this document</p>
    <p class="no-stubs-hint">
        Type <code>^^</code> to insert a stub
    </p>
</div>

<style>
    .no-stubs {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 40px 20px;
        text-align: center;
        color: var(--text-muted);
        width: 100%;
    }

    .no-stubs-icon {
        opacity: 0.5;
        margin-bottom: 12px;
    }

    .no-stubs-text {
        margin: 0;
        font-size: var(--font-ui-small);
    }

    .no-stubs-hint {
        margin: 8px 0 0;
        font-size: var(--font-ui-smaller);
        opacity: 0.7;
    }

    .no-stubs-hint code {
        background: var(--background-modifier-border);
        padding: 2px 6px;
        border-radius: 3px;
        font-family: var(--font-monospace);
    }
</style>
