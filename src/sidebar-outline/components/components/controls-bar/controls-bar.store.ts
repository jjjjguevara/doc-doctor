import { writable } from 'svelte/store';
import { Store } from '../../../../helpers/store';

export const POSSIBLE_FONT_SIZES = [10, 12, 14, 16, 18, 20, 22, 24] as const;
export type FontSize = (typeof POSSIBLE_FONT_SIZES)[number];
export const fontSize = writable<FontSize>(12);

export const isReading = writable<boolean>(false);

export const pluginIdle = writable(false);

export type ViewMode = 'annotations' | 'stubs' | 'ai';

type Controls = {
    showSearchInput: boolean;
    showLabelsFilter: boolean;
    showExtraButtons: boolean;
    showStylesSettings: boolean;
    showOutlineSettings: boolean;
    viewMode: ViewMode;
    showStubsSettings: boolean;
    showStubsSearch: boolean;
};

type ControlsAction =
    | { type: 'TOGGLE_EXTRA_BUTTONS' }
    | { type: 'TOGGLE_OUTLINE_SETTINGS' }
    | { type: 'TOGGLE_STYLES_SETTINGS' }
    | { type: 'TOGGLE_SEARCH_INPUT' }
    | { type: 'TOGGLE_LABELS_FILTERS' }
    | { type: 'SET_VIEW_MODE'; payload: ViewMode }
    | { type: 'TOGGLE_STUBS_SETTINGS' }
    | { type: 'TOGGLE_STUBS_SEARCH' };

const updateState = (store: Controls, action: ControlsAction) => {
    if (action.type === 'TOGGLE_SEARCH_INPUT') {
        store.showSearchInput = !store.showSearchInput;
        if (store.showSearchInput) store.showStylesSettings = false;
    } else if (action.type === 'TOGGLE_LABELS_FILTERS') {
        store.showLabelsFilter = !store.showLabelsFilter;
        if (store.showLabelsFilter) store.showStylesSettings = false;
    } else if (action.type === 'TOGGLE_EXTRA_BUTTONS') {
        store.showExtraButtons = !store.showExtraButtons;
        if (!store.showExtraButtons) {
            store.showStylesSettings = false;
            store.showStubsSettings = false;
        }
    } else if (action.type === 'TOGGLE_STYLES_SETTINGS') {
        store.showStylesSettings = !store.showStylesSettings;
        if (store.showStylesSettings) {
            store.showSearchInput = false;
            store.showLabelsFilter = false;
            store.showOutlineSettings = false;
            store.showStubsSettings = false;
        }
    } else if (action.type === 'TOGGLE_OUTLINE_SETTINGS') {
        store.showOutlineSettings = !store.showOutlineSettings;
        if (store.showOutlineSettings) {
            store.showStylesSettings = false;
            store.showStubsSettings = false;
        }
    } else if (action.type === 'SET_VIEW_MODE') {
        store.viewMode = action.payload;
        // Reset view-specific settings when switching
        store.showStylesSettings = false;
        store.showStubsSettings = false;
        store.showStubsSearch = false;
    } else if (action.type === 'TOGGLE_STUBS_SETTINGS') {
        store.showStubsSettings = !store.showStubsSettings;
        if (store.showStubsSettings) {
            store.showStylesSettings = false;
            store.showOutlineSettings = false;
            store.showStubsSearch = false;
        }
    } else if (action.type === 'TOGGLE_STUBS_SEARCH') {
        store.showStubsSearch = !store.showStubsSearch;
        if (store.showStubsSearch) {
            store.showStubsSettings = false;
        }
    }
};
export const reducer = (store: Controls, action: ControlsAction): Controls => {
    updateState(store, action);
    return store;
};

export const controls = new Store<Controls, ControlsAction>(
    {
        showLabelsFilter: false,
        showSearchInput: false,
        showExtraButtons: false,
        showStylesSettings: false,
        showOutlineSettings: false,
        viewMode: 'annotations',
        showStubsSettings: false,
        showStubsSearch: false,
    },
    reducer,
);
