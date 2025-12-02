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
        // LLM semantic fields
        semanticPurpose: `Use when a factual claim, statistic, quote, or technical specification lacks supporting evidence or citation. This stub signals that readers cannot verify the information without an authoritative source. Common triggers include: numerical data, historical facts, API specifications, protocol references, quoted statements, and any claim that makes a testable assertion about the world.`,
        vectorFamily: 'Retrieval',
        ontologicalDimension: 'Epistemic Status',
        indicators: [
            'Statistics or numerical claims without source',
            'Quoted text without attribution',
            'Technical specifications without reference',
            '"According to..." without link',
            'Historical facts or dates',
            'Claims about external systems or standards',
        ],
        antiPatterns: [
            'Self-evident truths that need no citation',
            "Author's own opinions clearly marked as such",
            'Content already marked with placeholder citations',
        ],
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
        // LLM semantic fields
        semanticPurpose: `Use when content is ambiguous, vague, or could be interpreted multiple ways. This stub signals that the meaning is unclear and requires additional precision or context. Unlike "Question" (unknown answer) or "Expand" (insufficient depth), this addresses content that exists but lacks clarity.`,
        vectorFamily: 'Computation',
        ontologicalDimension: 'Content Completeness',
        indicators: [
            'Ambiguous pronouns or references',
            'Unclear scope or boundaries',
            'Jargon without definition',
            'Vague quantifiers (some, many, few)',
            'Implicit assumptions not stated',
            'Terms used inconsistently',
        ],
        antiPatterns: [
            'Content that is simply incomplete (use Expand)',
            'Content with unknown answers (use Question)',
            'Intentionally general overview content',
        ],
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
        // LLM semantic fields
        semanticPurpose: `Use when a topic is introduced but not sufficiently developed for the document's intended audience. This stub signals that the current content provides a foundation but lacks depth, examples, or elaboration needed for comprehension. The gap is about insufficient coverage, not missing evidence.`,
        vectorFamily: 'Creation',
        ontologicalDimension: 'Content Completeness',
        indicators: [
            'One-sentence explanations of complex topics',
            'Bullet lists that could be paragraphs',
            '"This section covers X" without covering X',
            'Concepts introduced but not explained',
            'Procedures described without steps',
            'Empty or near-empty sections',
        ],
        antiPatterns: [
            'Intentionally brief overview sections',
            'Summaries meant to be concise',
            'Content that links to detailed docs elsewhere',
        ],
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
        // LLM semantic fields
        semanticPurpose: `Use when the author has an unresolved question that affects the content direction or accuracy. This stub surfaces uncertainty that requires research, stakeholder input, or decision-making before the content can be finalized. It represents the author's acknowledged knowledge boundary.`,
        vectorFamily: 'Computation',
        ontologicalDimension: 'Workflow',
        indicators: [
            'Explicit question marks in content',
            '"TODO: decide..." or similar markers',
            'Multiple alternatives presented without choice',
            'Uncertainty language: "might", "could", "possibly"',
            'Placeholders like TBD, TBA, XXX',
        ],
        antiPatterns: [
            'Rhetorical questions for effect',
            'Questions answered in following text',
            'FAQ-style content structure',
        ],
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
        // LLM semantic fields
        semanticPurpose: `Use when content exists but its accuracy is uncertain and needs fact-checking. Unlike "Citation Needed" (missing source), this indicates the content may already have sources but the facts themselves need verification. This is common for outdated information, secondary sources, or rapidly changing domains.`,
        vectorFamily: 'Retrieval',
        ontologicalDimension: 'Epistemic Status',
        indicators: [
            'Dates or versions that may be outdated',
            'Information from secondary sources',
            'Content copied from other documents',
            'Technical details that may have changed',
            'Third-party features or integrations',
        ],
        antiPatterns: [
            'Content that simply needs a citation (use Citation Needed)',
            'Content known to be wrong (use Blocker)',
            'Theoretical or opinion content',
        ],
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
        // LLM semantic fields
        semanticPurpose: `Use when there are conflicting perspectives, interpretations, or approaches that need resolution or balanced presentation. This stub signals that reasonable people disagree and the content should either acknowledge multiple viewpoints or a decision needs to be made.`,
        vectorFamily: 'Synthesis',
        ontologicalDimension: 'Perspective',
        indicators: [
            'Contradicting sources or recommendations',
            'Industry debates without consensus',
            'Multiple valid approaches described',
            'Content that sparked disagreement in review',
            'One-sided presentation of contested topic',
        ],
        antiPatterns: [
            'Simple factual errors (use Blocker)',
            'Personal preference differences',
            'Already balanced multi-perspective content',
        ],
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
        // LLM semantic fields
        semanticPurpose: `Use when there is a known error, critical issue, or missing prerequisite that must be resolved before the document can be considered usable. This stub differs from others by indicating urgency and blocking status—the document should not be promoted or published with this stub unresolved.`,
        vectorFamily: 'Creation',
        ontologicalDimension: 'Epistemic Status',
        indicators: [
            'Contradictions within the document',
            'Outdated information (dates, versions)',
            'Broken links or references',
            'Code examples that do not compile',
            'Incorrect technical specifications',
            'Logical inconsistencies',
            'Missing critical prerequisites',
        ],
        antiPatterns: [
            'Content that is merely incomplete',
            'Opinions that someone disagrees with',
            'Alternative approaches (not errors)',
            'Nice-to-have improvements',
        ],
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
        // LLM semantic fields
        semanticPurpose: `Use for general task reminders or action items that don't fit other categories. This is a catch-all for workflow items like "review later", "add screenshots", or "check with team". For more specific gaps, prefer the specialized stub types.`,
        vectorFamily: 'Creation',
        ontologicalDimension: 'Workflow',
        indicators: [
            'Explicit TODO or FIXME markers',
            'Inline comments about future work',
            'Placeholder content like [TBD]',
            'Notes to self about improvements',
        ],
        antiPatterns: [
            'Gaps better served by specific types',
            'Long-term aspirational improvements',
            'Stylistic preferences',
        ],
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
