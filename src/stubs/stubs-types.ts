/**
 * Stubs Module - Type Definitions
 *
 * Configurable stubs system for tracking document gaps with bidirectional
 * sync between frontmatter and inline ^stub-* anchors.
 */

// =============================================================================
// CONFIGURATION TYPES (stored in plugin settings)
// =============================================================================

/**
 * Vector family for stub types (J-Editorial Framework)
 */
export type VectorFamily = 'Retrieval' | 'Computation' | 'Synthesis' | 'Creation' | 'Structural';

/**
 * Ontological dimension for stub types (J-Editorial Framework)
 */
export type OntologicalDimension =
    | 'Epistemic Status'      // Truth/evidence concerns
    | 'Content Completeness'  // Coverage concerns
    | 'Structural Quality'    // Organization concerns
    | 'Perspective'           // Bias/neutrality concerns
    | 'Dependencies'          // External constraints
    | 'Workflow';             // Process/action concerns

/**
 * Level 1: Stub type definition (user-configurable vocabulary)
 */
export interface StubTypeDefinition {
    /** Unique identifier for this stub type */
    id: string;
    /** The YAML key used in frontmatter (e.g., "link", "expand", "question") */
    key: string;
    /** Display name in UI (e.g., "Citation Needed", "Expand Section") */
    displayName: string;
    /** Color for sidebar and inline highlighting (hex color) */
    color: string;
    /** Lucide icon name (optional) */
    icon?: string;
    /** Description shown in tooltips */
    description?: string;
    /** Default stub description when using ^^ (compact syntax) */
    defaultStubDescription?: string;
    /** Default values for Level 2 properties when creating this type */
    defaults?: Record<string, unknown>;
    /** Sort order in UI (lower = first) */
    sortOrder: number;

    // ==========================================================================
    // LLM SEMANTIC FIELDS (for AI-powered stub suggestions)
    // ==========================================================================

    /**
     * Detailed explanation of when to use this stub type.
     * Used by LLM to understand the semantic purpose of the stub.
     * @example "Use when a factual claim lacks supporting evidence or citation."
     */
    semanticPurpose?: string;

    /**
     * The vector family this type belongs to (J-Editorial Framework).
     * Enables intelligent routing to appropriate resolution strategies.
     */
    vectorFamily?: VectorFamily;

    /**
     * The ontological dimension this type addresses (J-Editorial Framework).
     * Provides categorical context for gap classification.
     */
    ontologicalDimension?: OntologicalDimension;

    /**
     * Example indicators the LLM should look for in document content.
     * Concrete patterns that suggest this stub type is needed.
     */
    indicators?: string[];

    /**
     * Anti-patterns: situations where this type should NOT be used.
     */
    antiPatterns?: string[];
}

/**
 * Level 2: Structured property definition (user-configurable sub-properties)
 */
export interface StructuredPropertyDefinition {
    /** Unique identifier */
    id: string;
    /** Property key in YAML (e.g., "stub_form", "priority") */
    key: string;
    /** Display name in UI */
    displayName: string;
    /** Property value type */
    type: 'string' | 'enum' | 'array' | 'boolean' | 'number';
    /** For enum type: allowed values (user-configurable) */
    enumValues?: string[];
    /** For enum type: display names (parallel array) */
    enumDisplayNames?: string[];
    /** Whether property is required in structured syntax */
    required?: boolean;
    /** Default value when not specified */
    defaultValue?: unknown;
    /** Description shown in UI */
    description?: string;
    /** Sort order */
    sortOrder: number;
    /** Whether to include this property when inserting structured stubs (^^^) */
    includeInStructured?: boolean;
}

/**
 * Anchor generation settings
 */
export interface AnchorSettings {
    /** Prefix for stub anchors (default: "stub") */
    prefix: string;
    /** ID generation style */
    idStyle: 'random' | 'sequential' | 'type-prefixed';
    /** Length of random IDs */
    randomIdLength: number;
}

/**
 * Description formatting settings for stub insertion
 */
export interface DescriptionFormatSettings {
    /** Prefix to add before description (e.g., '"' for quotation marks) */
    prefix: string;
    /** Suffix to add after description (e.g., '"' for quotation marks) */
    suffix: string;
}

/**
 * Structured stub insertion settings
 */
export interface StructuredStubSettings {
    /** Whether to include default properties when inserting structured stubs (^^^) */
    includeDefaultProperties: boolean;
}

/**
 * Decoration settings for inline highlighting
 */
export interface StubDecorationSettings {
    /** Whether to show inline highlighting for ^stub-* anchors */
    enabled: boolean;
    /** Highlight style */
    style: 'background' | 'underline' | 'gutter' | 'badge';
    /** Opacity for highlights (0-1) */
    opacity: number;
    /** Show tooltip on hover */
    showTooltip: boolean;
}

/**
 * Sidebar display settings
 */
export interface StubSidebarSettings {
    /** Default expanded state for type groups */
    expandedByDefault: boolean;
    /** Whether to show empty type groups */
    showEmptyGroups: boolean;
    /** Sort stubs within groups */
    sortOrder: 'document' | 'alphabetical' | 'priority';
    /** Truncate description at this length (0 = no truncation) */
    descriptionMaxLength: number;
    /** Show stub count badge on sidebar icon */
    showCountBadge: boolean;
    /** Font size for stub items */
    fontSize: number;
    /** Show search input */
    showSearchInput: boolean;
    /** Show type filter */
    showTypeFilter: boolean;
    /** Hidden stub type IDs */
    hiddenTypes: string[];
}

/**
 * Root stubs configuration (stored in plugin settings)
 */
export interface StubsConfiguration {
    /** Whether stubs feature is enabled */
    enabled: boolean;
    /** The frontmatter key used for stubs array (default: "stubs") */
    frontmatterKey: string;
    /** Level 1: Defined stub types */
    stubTypes: Record<string, StubTypeDefinition>;
    /** Level 2: Structured property definitions */
    structuredProperties: Record<string, StructuredPropertyDefinition>;
    /** Anchor generation settings */
    anchors: AnchorSettings;
    /** Description formatting settings */
    descriptionFormat: DescriptionFormatSettings;
    /** Structured stub insertion settings */
    structuredStubs: StructuredStubSettings;
    /** Editor decoration settings */
    decorations: StubDecorationSettings;
    /** Sidebar display settings */
    sidebar: StubSidebarSettings;
}

// =============================================================================
// PARSED STUB TYPES (runtime representation)
// =============================================================================

/**
 * Parsed stub from frontmatter
 */
export interface ParsedStub {
    /** Generated unique ID for this stub instance */
    id: string;
    /** The stub type key (matches StubTypeDefinition.key) */
    type: string;
    /** The stub description text */
    description: string;
    /** Anchor reference (e.g., "^stub-abc123") */
    anchor: string | null;
    /** Whether the anchor was found in document content */
    anchorResolved: boolean;
    /** Level 2 properties (if structured syntax) */
    properties: Record<string, unknown>;
    /** Parsing syntax used */
    syntax: 'compact' | 'structured';
    /** Line number in frontmatter where stub starts */
    frontmatterLine: number;
    /** Any parsing warnings for this stub */
    warnings: string[];
}

/**
 * Inline anchor found in document content
 */
export interface InlineAnchor {
    /** The full anchor ID (e.g., "^stub-abc123") */
    id: string;
    /** Position in document content */
    position: {
        /** 0-indexed line number in content (after frontmatter) */
        line: number;
        /** Character offset in line */
        ch: number;
        /** Absolute offset from content start */
        offset: number;
    };
    /** The line content where anchor appears */
    lineContent: string;
    /** Whether anchor is at end of line (standard position) */
    isEndOfLine: boolean;
    /** Whether this anchor has a corresponding frontmatter stub */
    hasStub: boolean;
    /** The stub type key (if linked to a stub) */
    stubType?: string;
    /** The stub description (if linked to a stub) */
    stubDescription?: string;
}

/**
 * Linked pair of stub and anchor
 */
export interface LinkedPair {
    stub: ParsedStub;
    anchor: InlineAnchor;
}

// =============================================================================
// SYNC STATE TYPES
// =============================================================================

/**
 * Complete sync state for a document
 */
export interface SyncState {
    /** All stubs parsed from frontmatter */
    stubs: ParsedStub[];
    /** All ^stub-* anchors found in content */
    anchors: InlineAnchor[];
    /** Successfully linked stub-anchor pairs */
    linked: LinkedPair[];
    /** Stubs without corresponding anchors in content */
    orphanedStubs: ParsedStub[];
    /** Anchors without corresponding frontmatter stubs */
    orphanedAnchors: InlineAnchor[];
    /** Last sync timestamp */
    lastSyncTime: number;
    /** Any sync errors encountered */
    errors: SyncError[];
}

/**
 * Sync error
 */
export interface SyncError {
    type: 'parse_error' | 'anchor_collision' | 'invalid_anchor_format' | 'yaml_error';
    message: string;
    location?: { line: number; ch: number };
}

/**
 * Parse result from frontmatter parser
 */
export interface StubParseResult {
    stubs: ParsedStub[];
    errors: StubParseError[];
    warnings: StubParseWarning[];
}

export interface StubParseError {
    type: 'invalid_format' | 'invalid_entry' | 'missing_description' | 'invalid_value' | 'yaml_error';
    index?: number;
    message: string;
    line?: number;
}

export interface StubParseWarning {
    type: 'unknown_type' | 'unknown_property' | 'invalid_property_value';
    index?: number;
    stubType?: string;
    property?: string;
    message: string;
}

// =============================================================================
// UI STATE TYPES
// =============================================================================

/**
 * Stubs store state for sidebar
 */
export interface StubsState {
    /** Current sync state */
    sync: SyncState;
    /** Stubs grouped by type for sidebar display */
    byType: Map<string, ParsedStub[]>;
    /** Loading state */
    loading: boolean;
    /** Error message if any */
    error: string | null;
    /** Expanded type groups in sidebar */
    expandedTypes: Set<string>;
    /** Currently selected stub ID */
    selectedStubId: string | null;
    /** Search/filter text */
    filterText: string;
}

/**
 * Orphan resolution action
 */
export interface OrphanResolution {
    /** The orphan item ID (stub ID or anchor ID) */
    itemId: string;
    /** Resolution strategy */
    strategy: 'delete' | 'reinsert' | 'create_stub' | 'ignore' | 'convert';
    /** For create_stub: the stub type to use */
    stubType?: string;
    /** For create_stub: the stub description */
    description?: string;
}

// =============================================================================
// ACTION TYPES (for reducer pattern)
// =============================================================================

export type StubsSettingsActions =
    // Stub type management (Level 1)
    | { type: 'STUBS_ADD_TYPE'; payload: { key: string; displayName: string; color: string; icon?: string } }
    | { type: 'STUBS_UPDATE_TYPE'; payload: { id: string; updates: Partial<StubTypeDefinition> } }
    | { type: 'STUBS_DELETE_TYPE'; payload: { id: string } }
    | { type: 'STUBS_REORDER_TYPES'; payload: { orderedIds: string[] } }
    // Structured property management (Level 2)
    | { type: 'STUBS_ADD_PROPERTY'; payload: { key: string; displayName: string; type: StructuredPropertyDefinition['type'] } }
    | { type: 'STUBS_UPDATE_PROPERTY'; payload: { id: string; updates: Partial<StructuredPropertyDefinition> } }
    | { type: 'STUBS_TOGGLE_PROPERTY_INCLUDE'; payload: { id: string } }
    | { type: 'STUBS_DELETE_PROPERTY'; payload: { id: string } }
    | { type: 'STUBS_ADD_ENUM_VALUE'; payload: { propertyId: string; value: string; displayName?: string } }
    | { type: 'STUBS_REMOVE_ENUM_VALUE'; payload: { propertyId: string; value: string } }
    // Anchor settings
    | { type: 'STUBS_SET_ANCHOR_PREFIX'; payload: { prefix: string } }
    | { type: 'STUBS_SET_ANCHOR_ID_STYLE'; payload: { style: AnchorSettings['idStyle'] } }
    | { type: 'STUBS_SET_ANCHOR_ID_LENGTH'; payload: { length: number } }
    // Decoration settings
    | { type: 'STUBS_SET_DECORATIONS_ENABLED'; payload: { enabled: boolean } }
    | { type: 'STUBS_SET_DECORATION_STYLE'; payload: { style: StubDecorationSettings['style'] } }
    | { type: 'STUBS_SET_DECORATION_OPACITY'; payload: { opacity: number } }
    // Sidebar settings
    | { type: 'STUBS_SET_SIDEBAR_FONT_SIZE'; payload: { fontSize: number } }
    | { type: 'STUBS_SET_SIDEBAR_EXPANDED_DEFAULT'; payload: { expanded: boolean } }
    | { type: 'STUBS_TOGGLE_TYPE_VISIBILITY'; payload: { typeId: string } }
    | { type: 'STUBS_SET_HIDDEN_TYPES'; payload: { hiddenTypes: string[] } }
    | { type: 'STUBS_SET_SHOW_SEARCH'; payload: { show: boolean } }
    | { type: 'STUBS_SET_SHOW_TYPE_FILTER'; payload: { show: boolean } }
    // General
    | { type: 'STUBS_SET_ENABLED'; payload: { enabled: boolean } }
    | { type: 'STUBS_SET_FRONTMATTER_KEY'; payload: { key: string } }
    // Structured stubs settings
    | { type: 'STUBS_SET_INCLUDE_DEFAULT_PROPERTIES'; payload: { include: boolean } };
