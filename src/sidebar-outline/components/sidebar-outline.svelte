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
	import SecondaryControlsBar from './components/controls-bar/components/extra-buttons.svelte';
	import TabsFilter from './components/controls-bar/components/tabs-filter.svelte';
	import OutlineSettings from './components/controls-bar/components/outline-settings.svelte';
	import AnnotationsSearch from './components/controls-bar/components/annotations-search.svelte';
	import AIControlsBar from './components/ai-view/ai-controls-bar.svelte';
	import AIView from './components/ai-view/ai-view.svelte';
	import StubsSearch from './components/stubs-list/stubs-search.svelte';

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
                <ControlsBar />
            {:else if $controls.viewMode === 'stubs'}
                <StubsControlsBar {plugin} />
            {:else if $controls.viewMode === 'ai'}
                <AIControlsBar {plugin} />
            {/if}
        </div>

        <!-- Annotations expanded panels -->
        {#if $controls.viewMode === 'annotations'}
            {#if $controls.showExtraButtons}
                <div class="expanded-panel">
                    <SecondaryControlsBar {plugin} />
                </div>
            {/if}
            {#if $controls.showSearchInput}
                <div class="expanded-panel">
                    <AnnotationsSearch />
                </div>
            {/if}
            {#if $controls.showLabelsFilter}
                <div class="expanded-panel">
                    <TabsFilter />
                </div>
            {/if}
            {#if $controls.showOutlineSettings}
                <div class="expanded-panel">
                    <OutlineSettings />
                </div>
            {/if}
        {/if}

        <!-- Stubs expanded panels -->
        {#if $controls.viewMode === 'stubs'}
            {#if $controls.showStubsSearch}
                <div class="expanded-panel">
                    <StubsSearch />
                </div>
            {/if}
        {/if}

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
            <div class="stubs-content">
                <StubsList {plugin} />
            </div>
        {:else if $controls.viewMode === 'ai'}
            <div class="ai-content">
                <AIView {plugin} />
            </div>
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
        align-items: stretch;
        justify-content: start;
    }

    .outline-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        padding: 0 8px;
        gap: 8px;
        flex-shrink: 0;
        position: relative;
        z-index: 10;
    }

    .expanded-panel {
        width: 100%;
        padding: 0 8px;
        box-sizing: border-box;
    }

    .stubs-content {
        width: 100%;
        padding: 0 8px;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        gap: 8px;
        flex: 1;
        overflow-y: auto;
    }

    .ai-content {
        width: 100%;
        padding: 0 8px;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        flex: 1;
        overflow: hidden;
    }
</style>
