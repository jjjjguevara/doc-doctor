<script lang="ts">
    import { l } from '../../../../../lang/lang';
    import { controls, isReading } from '../controls-bar.store';
    import { tts } from '../helpers/tts';
    import { Paintbrush, Settings, ClipboardCopy, FileAudio, StopCircle } from 'lucide-svelte';
    import LabeledAnnotations from '../../../../../main';
    import { get } from 'svelte/store';
    import { filteredBySearchAndCategory } from '../../annotations-list/annotations-list.store';
    import { annotationsToText } from '../../../../../clipboard/helpers/annotations-to-text';
    import { clipboard } from 'electron';
    import { Notice } from 'obsidian';
    import { onDestroy } from 'svelte';

    export let plugin: LabeledAnnotations;

    export const copyAnnotationsToClipboard = () => {
        const annotations = Object.values(
            get(filteredBySearchAndCategory).labels,
        )
            .flat()
            .sort((a, b) => a.position.from - b.position.from);
        const outline = plugin.outline;
        const f = outline.getValue().view?.file;
        if (f) {
            const folder = f.parent?.path as string;
            const basename = f.basename;

            const text = annotationsToText(
                [{ folder, basename, annotations }],
                plugin.settings.getValue().clipboard.templates,
                folder,
            );
            clipboard.writeText(text);
            new Notice(l.OUTLINE_NOTICE_COPIED_TO_CLIPBOARD);
        } else {
            new Notice(l.OUTLINE_NOTICE_COULD_NOT_COPY);
        }
    };

    const read = () => {
        if (!tts.isReading) {
            tts.read();
        } else {
            tts.stop();
        }
    };

    const toggleShowStylesSettings = () => {
        controls.dispatch({ type: 'TOGGLE_STYLES_SETTINGS' });
    };

    const unsub = tts.subscribe((value) => isReading.set(value));
    onDestroy(unsub);
</script>

<div class="extra-controls">
    <button
        class="control-btn"
        class:active={$isReading}
        on:click={read}
        title={l.OUTLINE_READ_ANNOTATIONS}
    >
        {#if $isReading}
            <StopCircle size={14} />
        {:else}
            <FileAudio size={14} />
        {/if}
    </button>
    <button
        class="control-btn"
        on:click={copyAnnotationsToClipboard}
        title={l.OUTLINE_COPY_ANNOTATIONS_TO_CLIPBOARD}
    >
        <ClipboardCopy size={14} />
    </button>
    <button
        class="control-btn"
        class:active={$controls.showStylesSettings}
        on:click={toggleShowStylesSettings}
        title={l.OUTLINE_TOGGLE_STYLES_SETTINGS}
    >
        <Paintbrush size={14} />
    </button>
    <button
        class="control-btn"
        class:active={$controls.showOutlineSettings}
        on:click={() => controls.dispatch({ type: 'TOGGLE_OUTLINE_SETTINGS' })}
        title={l.OUTLINE_SETTINGS}
    >
        <Settings size={14} />
    </button>
</div>

<style>
    .extra-controls {
        display: flex;
        align-items: center;
        gap: 4px;
    }

    .control-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 4px;
        border: none;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.15s ease;
    }

    .control-btn:hover {
        background: var(--background-modifier-hover);
        color: var(--text-normal);
    }

    .control-btn.active {
        background: var(--interactive-accent);
        color: var(--text-on-accent);
    }
</style>
