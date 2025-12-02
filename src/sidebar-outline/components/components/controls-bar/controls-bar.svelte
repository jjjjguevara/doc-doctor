<script lang="ts">
	import { controls } from './controls-bar.store';
	import { l } from '../../../../lang/lang';
	import { searchTerm } from './components/search-input.store';
	import { ListFilter, MoreHorizontal, Search } from 'lucide-svelte';
	import { filteredHiddenCategories, filteredHiddenLabels } from '../annotations-list/annotations-list.store';

	$: hasActiveFilters = $filteredHiddenLabels.size > 0 || $filteredHiddenCategories.size > 0;

	const toggleLabelsFilter = () => {
		controls.dispatch({type:"TOGGLE_LABELS_FILTERS"});
		if ($controls.showSearchInput) {
			controls.dispatch({type:"TOGGLE_SEARCH_INPUT"});
		}
	};

	const toggleSearch = () => {
		controls.dispatch({type:"TOGGLE_SEARCH_INPUT"});
		if ($controls.showLabelsFilter) {
			controls.dispatch({type:"TOGGLE_LABELS_FILTERS"});
		}
	};

	const toggleSecondaryControlsBar = () => {
		controls.dispatch({type:"TOGGLE_EXTRA_BUTTONS"});
	};
</script>

<div class="annotations-controls">
	<button
		class="control-btn"
		class:active={$controls.showSearchInput || !!$searchTerm}
		on:click={toggleSearch}
		title={l.OUTLINE_SEARCH_ANNOTATIONS}
	>
		<Search size={14} />
	</button>
	<button
		class="control-btn"
		class:active={$controls.showLabelsFilter || hasActiveFilters}
		on:click={toggleLabelsFilter}
		title={l.OUTLINE_FILTER_ANNOTATIONS}
	>
		<ListFilter size={14} />
		{#if hasActiveFilters}
			<span class="filter-badge">!</span>
		{/if}
	</button>
	<button
		class="control-btn"
		class:active={$controls.showExtraButtons}
		on:click={toggleSecondaryControlsBar}
		title={l.OUTLINE_SHOW_ALL_CONTROLS}
	>
		<MoreHorizontal size={14} />
	</button>
</div>
<style>
	.annotations-controls {
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
		position: relative;
	}

	.control-btn:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}

	.control-btn.active {
		background: var(--interactive-accent);
		color: var(--text-on-accent);
	}

	.filter-badge {
		position: absolute;
		top: -2px;
		right: -2px;
		min-width: 14px;
		height: 14px;
		font-size: 9px;
		font-weight: 600;
		line-height: 14px;
		text-align: center;
		background: var(--interactive-accent);
		color: var(--text-on-accent);
		border-radius: 7px;
		padding: 0 3px;
	}
</style>
