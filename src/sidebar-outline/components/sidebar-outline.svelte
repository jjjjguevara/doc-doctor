<script lang="ts">
	import ControlsBar from './components/controls-bar/controls-bar.svelte';
	import FlatOutline from './components/annotations-list/annotations-list.svelte';
	import NoAnnotations from './components/no-annotations.svelte';
	import { searchTerm } from './components/controls-bar/components/search-input.store';
	import {
		filteredBySearch,
		filteredHiddenCategories,
		filteredHiddenLabels
	} from './components/annotations-list/annotations-list.store';
	import LabeledAnnotations from '../../main';
	import { controls, pluginIdle } from './components/controls-bar/controls-bar.store';
	import PluginIdle from './components/plugin-idle.svelte';
	import StylesList from './components/controls-bar/components/styles/styles-list.svelte';
	import ViewModeSwitcher from './components/controls-bar/components/view-mode-switcher.svelte';
	import StubsList from './components/stubs-list/stubs-list.svelte';
	import StubsSidebarSettings from './components/stubs-list/stubs-sidebar-settings.svelte';
	import StubsControlsBar from './components/stubs-list/stubs-controls-bar.svelte';

	export let plugin: LabeledAnnotations;
</script>

<div class="outline">
    {#if $pluginIdle}
        <PluginIdle {plugin} />
    {:else}
        <!-- Combined header with toggle and view-specific controls -->
        <div class="outline-header">
            <ViewModeSwitcher {plugin} />
            {#if $controls.viewMode === 'annotations'}
                <ControlsBar {plugin} />
            {:else if $controls.viewMode === 'stubs'}
                <StubsControlsBar {plugin} />
            {/if}
        </div>

        <!-- Content area -->
        {#if $controls.viewMode === 'annotations'}
            {#if $controls.showStylesSettings}
                <StylesList {plugin} />
            {:else if Object.values($filteredBySearch.labels).flat().length || $searchTerm.length || $filteredHiddenLabels.size || $filteredHiddenCategories.size}
                <FlatOutline {plugin} />
            {:else}
                <NoAnnotations />
            {/if}
        {:else if $controls.viewMode === 'stubs'}
            {#if $controls.showStubsSettings}
                <StubsSidebarSettings {plugin} />
            {/if}
            <StubsList {plugin} />
        {/if}
    {/if}
</div>

<style>
    .outline {
        height: 100%;
        width: 100%;
        box-sizing: border-box;
        display: flex;
        gap: 8px;
        flex-direction: column;
        align-items: start;
        justify-content: start;
    }

    .outline-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        padding: 0 8px;
        gap: 8px;
        flex-wrap: wrap;
    }
</style>
