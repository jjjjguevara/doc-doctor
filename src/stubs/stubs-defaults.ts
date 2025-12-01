/**
 * Stubs Module - Default Configuration
 *
 * Provides default stub types based on J-Editorial Tier 1 types,
 * but fully user-customizable.
 */

import {
    StubsConfiguration,
    StubTypeDefinition,
    StructuredPropertyDefinition,
} from './stubs-types';

// =============================================================================
// DEFAULT STUB TYPES (Level 1)
// =============================================================================

const DEFAULT_STUB_TYPES: Record<string, StubTypeDefinition> = {
    link: {
        id: 'link',
        key: 'link',
        displayName: 'Citation Needed',
        color: '#e67e22',
        icon: 'link',
        description: 'Content needs a citation or reference',
        defaultStubDescription: 'Citation needed',
        defaults: { stub_form: 'persistent' },
        sortOrder: 1,
    },
    clarify: {
        id: 'clarify',
        key: 'clarify',
        displayName: 'Clarify',
        color: '#3498db',
        icon: 'help-circle',
        description: 'Content is ambiguous and needs clarification',
        defaultStubDescription: 'Needs clarification',
        defaults: { stub_form: 'transient' },
        sortOrder: 2,
    },
    expand: {
        id: 'expand',
        key: 'expand',
        displayName: 'Expand',
        color: '#2ecc71',
        icon: 'plus-circle',
        description: 'Section needs more detail or content',
        defaultStubDescription: 'Expand this section',
        defaults: { stub_form: 'transient' },
        sortOrder: 3,
    },
    question: {
        id: 'question',
        key: 'question',
        displayName: 'Question',
        color: '#9b59b6',
        icon: 'message-circle',
        description: 'Open question that needs research or decision',
        defaultStubDescription: 'Open question',
        defaults: { stub_form: 'transient' },
        sortOrder: 4,
    },
    verify: {
        id: 'verify',
        key: 'verify',
        displayName: 'Verify',
        color: '#f39c12',
        icon: 'check-circle',
        description: 'Content needs fact-checking or verification',
        defaultStubDescription: 'Needs verification',
        defaults: { stub_form: 'transient' },
        sortOrder: 5,
    },
    controversy: {
        id: 'controversy',
        key: 'controversy',
        displayName: 'Controversy',
        color: '#e74c3c',
        icon: 'alert-triangle',
        description: 'Conflicting perspectives or unresolved disagreement',
        defaultStubDescription: 'Disputed content',
        defaults: { stub_form: 'blocking' },
        sortOrder: 6,
    },
    blocker: {
        id: 'blocker',
        key: 'blocker',
        displayName: 'Blocker',
        color: '#c0392b',
        icon: 'octagon',
        description: 'Cannot proceed until this is resolved',
        defaultStubDescription: 'Blocking issue',
        defaults: { stub_form: 'blocking' },
        sortOrder: 7,
    },
    todo: {
        id: 'todo',
        key: 'todo',
        displayName: 'Todo',
        color: '#7f8c8d',
        icon: 'check-square',
        description: 'Task reminder or action item',
        defaultStubDescription: 'TODO',
        defaults: { stub_form: 'transient' },
        sortOrder: 8,
    },
};

// =============================================================================
// DEFAULT STRUCTURED PROPERTIES (Level 2)
// =============================================================================

const DEFAULT_STRUCTURED_PROPERTIES: Record<string, StructuredPropertyDefinition> = {
    stub_form: {
        id: 'stub_form',
        key: 'stub_form',
        displayName: 'Form',
        type: 'enum',
        enumValues: ['transient', 'persistent', 'blocking', 'structural'],
        enumDisplayNames: ['Transient', 'Persistent', 'Blocking', 'Structural'],
        required: false,
        defaultValue: 'transient',
        description: 'Expected lifecycle and severity of the stub',
        sortOrder: 1,
    },
    priority: {
        id: 'priority',
        key: 'priority',
        displayName: 'Priority',
        type: 'enum',
        enumValues: ['low', 'medium', 'high', 'critical'],
        enumDisplayNames: ['Low', 'Medium', 'High', 'Critical'],
        required: false,
        description: 'Urgency level for resolution',
        sortOrder: 2,
    },
    assignees: {
        id: 'assignees',
        key: 'assignees',
        displayName: 'Assignees',
        type: 'array',
        required: false,
        description: 'People responsible for resolution (wikilinks)',
        sortOrder: 3,
    },
    references: {
        id: 'references',
        key: 'references',
        displayName: 'References',
        type: 'array',
        required: false,
        description: 'Related documents or external links',
        sortOrder: 4,
    },
};

// =============================================================================
// DEFAULT CONFIGURATION
// =============================================================================

export const DEFAULT_STUBS_CONFIGURATION = (): StubsConfiguration => ({
    enabled: true,
    frontmatterKey: 'stubs',
    stubTypes: { ...DEFAULT_STUB_TYPES },
    structuredProperties: { ...DEFAULT_STRUCTURED_PROPERTIES },
    anchors: {
        prefix: 'stub',
        idStyle: 'random',
        randomIdLength: 6,
    },
    descriptionFormat: {
        prefix: '',
        suffix: '',
    },
    structuredStubs: {
        includeDefaultProperties: true,
    },
    decorations: {
        enabled: true,
        style: 'background',
        opacity: 0.3,
        showTooltip: true,
    },
    sidebar: {
        expandedByDefault: true,
        showEmptyGroups: false,
        sortOrder: 'document',
        descriptionMaxLength: 50,
        showCountBadge: true,
        fontSize: 12,
        showSearchInput: false,
        showTypeFilter: false,
        hiddenTypes: [],
    },
});

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Get stub type definition by key
 */
export function getStubTypeByKey(
    config: StubsConfiguration,
    key: string
): StubTypeDefinition | undefined {
    return Object.values(config.stubTypes).find((t) => t.key === key);
}

/**
 * Get structured property definition by key
 */
export function getPropertyByKey(
    config: StubsConfiguration,
    key: string
): StructuredPropertyDefinition | undefined {
    return Object.values(config.structuredProperties).find((p) => p.key === key);
}

/**
 * Get all stub types sorted by sortOrder
 */
export function getSortedStubTypes(config: StubsConfiguration): StubTypeDefinition[] {
    return Object.values(config.stubTypes).sort((a, b) => a.sortOrder - b.sortOrder);
}

/**
 * Get all structured properties sorted by sortOrder
 */
export function getSortedProperties(config: StubsConfiguration): StructuredPropertyDefinition[] {
    return Object.values(config.structuredProperties).sort((a, b) => a.sortOrder - b.sortOrder);
}

/**
 * Check if a stub type key is valid (exists in configuration)
 */
export function isValidStubType(config: StubsConfiguration, key: string): boolean {
    return Object.values(config.stubTypes).some((t) => t.key === key);
}

/**
 * Generate a unique ID for new items
 */
export function generateId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2, 5);
}

/**
 * Get next sort order for new stub type
 */
export function getNextSortOrder(config: StubsConfiguration): number {
    const types = Object.values(config.stubTypes);
    if (types.length === 0) return 1;
    return Math.max(...types.map((t) => t.sortOrder)) + 1;
}

/**
 * Get default color for new stub type (cycle through palette)
 */
const COLOR_PALETTE = [
    '#e67e22', '#3498db', '#2ecc71', '#9b59b6', '#f39c12',
    '#e74c3c', '#1abc9c', '#34495e', '#e91e63', '#00bcd4',
];

export function getNextDefaultColor(config: StubsConfiguration): string {
    const usedColors = new Set(Object.values(config.stubTypes).map((t) => t.color.toLowerCase()));
    for (const color of COLOR_PALETTE) {
        if (!usedColors.has(color.toLowerCase())) {
            return color;
        }
    }
    // If all colors used, generate a random one
    return `#${Math.floor(Math.random() * 16777215).toString(16).padStart(6, '0')}`;
}
