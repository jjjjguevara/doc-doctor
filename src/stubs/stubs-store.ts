/**
 * Stubs Store - Reactive State Management
 *
 * Manages stubs state for the sidebar and provides reactive updates
 * using Svelte-compatible store pattern.
 */

import { writable, derived, get, Writable } from 'svelte/store';
import {
    ParsedStub,
    InlineAnchor,
    SyncState,
    StubsState,
    StubsConfiguration,
    LinkedPair,
} from './stubs-types';
import { getSortedStubTypes } from './stubs-defaults';

// =============================================================================
// MAIN STORES
// =============================================================================

/**
 * Current sync state for the active document
 */
export const syncState = writable<SyncState>({
    stubs: [],
    anchors: [],
    linked: [],
    orphanedStubs: [],
    orphanedAnchors: [],
    lastSyncTime: 0,
    errors: [],
});

/**
 * Stubs configuration (from plugin settings)
 */
export const stubsConfig = writable<StubsConfiguration | null>(null);

/**
 * UI state stores
 */
export const expandedTypes = writable<Set<string>>(new Set());
export const selectedStubId = writable<string | null>(null);
export const filterText = writable<string>('');
export const hiddenTypes = writable<Set<string>>(new Set());

/**
 * Focus location for selected stub: 'inline' (anchor in content) or 'frontmatter'
 * Used for cycling behavior when clicking the same stub multiple times
 */
export const stubFocusLocation = writable<'inline' | 'frontmatter'>('inline');

/**
 * Type filter for sidebar - which stub types to show (empty set = show all)
 */
export const activeTypeFilters = writable<Set<string>>(new Set());

/**
 * Sort order for stubs:
 * - 'type': grouped by type (default)
 * - 'asc': first to last in document (flat list)
 * - 'desc': last to first in document (flat list)
 * - 'type-asc': grouped by type, sorted asc within type
 * - 'type-desc': grouped by type, sorted desc within type
 */
export type StubSortOrder = 'type' | 'asc' | 'desc' | 'type-asc' | 'type-desc';
export const stubSortOrder = writable<StubSortOrder>('type');

/**
 * Link status filter: 'all', 'linked', 'unlinked'
 */
export type LinkStatusFilter = 'all' | 'linked' | 'unlinked';
export const linkStatusFilter = writable<LinkStatusFilter>('all');

/**
 * Loading and error state
 */
export const isLoading = writable<boolean>(false);
export const errorMessage = writable<string | null>(null);

// =============================================================================
// DERIVED STORES
// =============================================================================

/**
 * All stubs from current sync
 */
export const allStubs = derived(syncState, ($sync) => $sync.stubs);

/**
 * All anchors from current sync
 */
export const allAnchors = derived(syncState, ($sync) => $sync.anchors);

/**
 * Stubs grouped by type
 */
export const stubsByType = derived(
    [syncState, stubsConfig],
    ([$sync, $config]) => {
        const grouped = new Map<string, ParsedStub[]>();

        if (!$config) return grouped;

        // Initialize with all configured types (even empty)
        const sortedTypes = getSortedStubTypes($config);
        for (const type of sortedTypes) {
            grouped.set(type.key, []);
        }

        // Group stubs by type
        for (const stub of $sync.stubs) {
            const existing = grouped.get(stub.type);
            if (existing) {
                existing.push(stub);
            } else {
                // Unknown type - add it
                grouped.set(stub.type, [stub]);
            }
        }

        return grouped;
    }
);

/**
 * Filtered stubs by search text, type filter, and link status
 */
export const filteredStubs = derived(
    [syncState, filterText, activeTypeFilters, linkStatusFilter],
    ([$sync, $filter, $typeFilters, $linkStatus]) => {
        let stubs = $sync.stubs;

        // Apply type filter (empty set = show all)
        if ($typeFilters.size > 0) {
            stubs = stubs.filter((stub) => $typeFilters.has(stub.type));
        }

        // Apply link status filter
        if ($linkStatus === 'linked') {
            stubs = stubs.filter((stub) => stub.anchorResolved);
        } else if ($linkStatus === 'unlinked') {
            stubs = stubs.filter((stub) => !stub.anchorResolved);
        }

        // Apply text search
        if ($filter && $filter.trim() !== '') {
            const searchLower = $filter.toLowerCase();
            stubs = stubs.filter((stub) => {
                return (
                    stub.description.toLowerCase().includes(searchLower) ||
                    stub.type.toLowerCase().includes(searchLower) ||
                    (stub.anchor && stub.anchor.toLowerCase().includes(searchLower))
                );
            });
        }

        return stubs;
    }
);

/**
 * Filtered stubs by type visibility and search
 */
export const visibleStubs = derived(
    [filteredStubs, hiddenTypes],
    ([$filtered, $hidden]) => {
        return $filtered.filter((stub) => !$hidden.has(stub.type));
    }
);

/**
 * Helper to get stub position in document (for sorting)
 * - Linked stubs: uses anchor position in document
 * - Unlinked stubs: uses frontmatter line as fallback (placed after linked stubs)
 *
 * The UNLINKED_OFFSET ensures unlinked stubs sort after all linked stubs
 * while preserving their relative frontmatter order.
 */
const UNLINKED_OFFSET = 1_000_000_000; // Large offset to place unlinked after linked

function getStubPosition(stub: ParsedStub, anchors: InlineAnchor[], stubIndex: number): number {
    // For linked stubs, try to find anchor position
    if (stub.anchor) {
        // Try exact match first
        let anchor = anchors.find((a) => a.id === stub.anchor);
        // Also try with ^ prefix if not found
        if (!anchor && !stub.anchor.startsWith('^')) {
            anchor = anchors.find((a) => a.id === `^${stub.anchor}` || a.id === stub.anchor);
        }
        if (anchor && anchor.position) {
            return anchor.position.offset;
        }
    }
    // For unlinked stubs, use stub index as fallback
    // Add UNLINKED_OFFSET so they sort after all linked stubs
    return UNLINKED_OFFSET + stubIndex;
}

/**
 * Sorted and visible stubs (flat list for position-based sorting)
 */
export const sortedVisibleStubs = derived(
    [visibleStubs, syncState, stubSortOrder],
    ([$stubs, $sync, $sortOrder]) => {
        // For type-grouped views, return unsorted (will be grouped later)
        if ($sortOrder === 'type' || $sortOrder === 'type-asc' || $sortOrder === 'type-desc') {
            return $stubs;
        }

        // Create index map for original positions
        const indexMap = new Map<string, number>();
        $stubs.forEach((stub, idx) => indexMap.set(stub.id, idx));

        // Sort by position in document
        const sorted = [...$stubs].sort((a, b) => {
            const idxA = indexMap.get(a.id) ?? 0;
            const idxB = indexMap.get(b.id) ?? 0;
            const posA = getStubPosition(a, $sync.anchors, idxA);
            const posB = getStubPosition(b, $sync.anchors, idxB);
            return $sortOrder === 'asc' ? posA - posB : posB - posA;
        });

        return sorted;
    }
);

/**
 * Visible stubs grouped by type (for type-based view)
 */
export const visibleStubsByType = derived(
    [visibleStubs, stubsConfig, syncState, stubSortOrder],
    ([$stubs, $config, $sync, $sortOrder]) => {
        const grouped = new Map<string, ParsedStub[]>();

        if (!$config) return grouped;

        // Create index map for original positions
        const indexMap = new Map<string, number>();
        $stubs.forEach((stub, idx) => indexMap.set(stub.id, idx));

        // Initialize with all configured types
        const sortedTypes = getSortedStubTypes($config);
        for (const type of sortedTypes) {
            grouped.set(type.key, []);
        }

        // Group visible stubs
        for (const stub of $stubs) {
            const existing = grouped.get(stub.type);
            if (existing) {
                existing.push(stub);
            } else {
                grouped.set(stub.type, [stub]);
            }
        }

        // Sort stubs within each group by position for type-asc/type-desc
        if ($sortOrder === 'type-asc' || $sortOrder === 'type-desc') {
            const isAsc = $sortOrder === 'type-asc';
            for (const [, stubs] of grouped) {
                stubs.sort((a, b) => {
                    const idxA = indexMap.get(a.id) ?? 0;
                    const idxB = indexMap.get(b.id) ?? 0;
                    const posA = getStubPosition(a, $sync.anchors, idxA);
                    const posB = getStubPosition(b, $sync.anchors, idxB);
                    return isAsc ? posA - posB : posB - posA;
                });
            }
        }

        return grouped;
    }
);

/**
 * Total stub count
 */
export const stubCount = derived(syncState, ($sync) => $sync.stubs.length);

/**
 * Orphan counts
 */
export const orphanedStubCount = derived(syncState, ($sync) => $sync.orphanedStubs.length);
export const orphanedAnchorCount = derived(syncState, ($sync) => $sync.orphanedAnchors.length);
export const hasOrphans = derived(
    [orphanedStubCount, orphanedAnchorCount],
    ([$stubCount, $anchorCount]) => $stubCount > 0 || $anchorCount > 0
);

/**
 * Count by type
 */
export const countByType = derived(stubsByType, ($byType) => {
    const counts = new Map<string, number>();
    for (const [type, stubs] of $byType) {
        counts.set(type, stubs.length);
    }
    return counts;
});

/**
 * Currently selected stub
 */
export const selectedStub = derived(
    [syncState, selectedStubId],
    ([$sync, $selectedId]) => {
        if (!$selectedId) return null;
        return $sync.stubs.find((s) => s.id === $selectedId) || null;
    }
);

/**
 * Anchor for selected stub
 */
export const selectedAnchor = derived(
    [syncState, selectedStub],
    ([$sync, $stub]) => {
        if (!$stub || !$stub.anchor) return null;
        return $sync.anchors.find((a) => a.id === $stub.anchor) || null;
    }
);

// =============================================================================
// STORE ACTIONS
// =============================================================================

/**
 * Update sync state
 */
export function updateSyncState(newState: SyncState): void {
    syncState.set(newState);
}

/**
 * Update stubs configuration
 */
export function updateStubsConfig(config: StubsConfiguration): void {
    stubsConfig.set(config);

    // Update hidden types from config
    hiddenTypes.set(new Set(config.sidebar.hiddenTypes));

    // Initialize expanded types based on config
    if (config.sidebar.expandedByDefault) {
        const allTypeKeys = Object.values(config.stubTypes).map((t) => t.key);
        expandedTypes.set(new Set(allTypeKeys));
    }
}

/**
 * Toggle type expansion in sidebar
 */
export function toggleTypeExpanded(typeKey: string): void {
    expandedTypes.update((current) => {
        const newSet = new Set(current);
        if (newSet.has(typeKey)) {
            newSet.delete(typeKey);
        } else {
            newSet.add(typeKey);
        }
        return newSet;
    });
}

/**
 * Expand all types
 */
export function expandAllTypes(): void {
    const config = get(stubsConfig);
    if (config) {
        const allTypeKeys = Object.values(config.stubTypes).map((t) => t.key);
        expandedTypes.set(new Set(allTypeKeys));
    }
}

/**
 * Collapse all types
 */
export function collapseAllTypes(): void {
    expandedTypes.set(new Set());
}

/**
 * Select a stub
 */
export function selectStub(stubId: string | null): void {
    selectedStubId.set(stubId);
}

/**
 * Toggle focus location between 'inline' and 'frontmatter'
 */
export function toggleFocusLocation(): void {
    stubFocusLocation.update((current) => (current === 'inline' ? 'frontmatter' : 'inline'));
}

/**
 * Set focus location explicitly
 */
export function setFocusLocation(location: 'inline' | 'frontmatter'): void {
    stubFocusLocation.set(location);
}

/**
 * Get current focus location
 */
export function getFocusLocation(): 'inline' | 'frontmatter' {
    return get(stubFocusLocation);
}

/**
 * Set filter text
 */
export function setFilterText(text: string): void {
    filterText.set(text);
}

/**
 * Toggle a type in the filter (add if not present, remove if present)
 */
export function toggleTypeFilter(typeKey: string): void {
    activeTypeFilters.update((current) => {
        const newSet = new Set(current);
        if (newSet.has(typeKey)) {
            newSet.delete(typeKey);
        } else {
            newSet.add(typeKey);
        }
        return newSet;
    });
}

/**
 * Clear all type filters (show all)
 */
export function clearTypeFilters(): void {
    activeTypeFilters.set(new Set());
}

/**
 * Set type filters from an array
 */
export function setTypeFilters(typeKeys: string[]): void {
    activeTypeFilters.set(new Set(typeKeys));
}

/**
 * Set sort order
 */
export function setSortOrder(order: StubSortOrder): void {
    stubSortOrder.set(order);
}

/**
 * Cycle through sort orders: type -> asc -> desc -> type-asc -> type-desc -> type
 */
export function cycleSortOrder(): void {
    stubSortOrder.update((current) => {
        switch (current) {
            case 'type':
                return 'asc';
            case 'asc':
                return 'desc';
            case 'desc':
                return 'type-asc';
            case 'type-asc':
                return 'type-desc';
            case 'type-desc':
                return 'type';
            default:
                return 'type';
        }
    });
}

/**
 * Set link status filter
 */
export function setLinkStatusFilter(status: LinkStatusFilter): void {
    linkStatusFilter.set(status);
}

/**
 * Toggle type visibility
 */
export function toggleTypeVisibility(typeKey: string): void {
    hiddenTypes.update((current) => {
        const newSet = new Set(current);
        if (newSet.has(typeKey)) {
            newSet.delete(typeKey);
        } else {
            newSet.add(typeKey);
        }
        return newSet;
    });
}

/**
 * Set loading state
 */
export function setLoading(loading: boolean): void {
    isLoading.set(loading);
}

/**
 * Set error message
 */
export function setError(error: string | null): void {
    errorMessage.set(error);
}

/**
 * Clear all state (on document close)
 */
export function clearState(): void {
    syncState.set({
        stubs: [],
        anchors: [],
        linked: [],
        orphanedStubs: [],
        orphanedAnchors: [],
        lastSyncTime: 0,
        errors: [],
    });
    selectedStubId.set(null);
    filterText.set('');
    errorMessage.set(null);
}

// =============================================================================
// HELPERS
// =============================================================================

/**
 * Get stub by ID
 */
export function getStubById(stubId: string): ParsedStub | undefined {
    const state = get(syncState);
    return state.stubs.find((s) => s.id === stubId);
}

/**
 * Get stub by anchor ID
 */
export function getStubByAnchorId(anchorId: string): ParsedStub | undefined {
    const state = get(syncState);
    return state.stubs.find((s) => s.anchor === anchorId);
}

/**
 * Get anchor by ID
 */
export function getAnchorById(anchorId: string): InlineAnchor | undefined {
    const state = get(syncState);
    return state.anchors.find((a) => a.id === anchorId);
}

/**
 * Get linked pair for stub
 */
export function getLinkedPair(stubId: string): LinkedPair | undefined {
    const state = get(syncState);
    return state.linked.find((p) => p.stub.id === stubId);
}

/**
 * Check if stub type is expanded
 */
export function isTypeExpanded(typeKey: string): boolean {
    return get(expandedTypes).has(typeKey);
}

/**
 * Check if stub type is visible
 */
export function isTypeVisible(typeKey: string): boolean {
    return !get(hiddenTypes).has(typeKey);
}
