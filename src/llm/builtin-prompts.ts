/**
 * Built-in Prompts
 *
 * Default prompts shipped with the Doc Doctor plugin.
 * Taxonomy based on J-Editorial Agent Task Taxonomy.
 */

import { PromptDefinition } from './prompt-schema';

/**
 * Built-in prompt definitions
 */
export const BUILTIN_PROMPTS: Omit<PromptDefinition, 'source' | 'filePath'>[] = [
    {
        id: 'analyze-annotate',
        name: 'Analyze & Annotate',
        icon: 'search',
        description: 'Full document analysis with stub suggestions',
        category: 'analysis',
        task_family: 'combinatorial',
        vector_type: 'check',
        reliability: 'high',
        enabled: true,
        context: {
            requires_selection: false,
            requires_file: true,
            file_types: ['md'],
        },
        system_extension: `## Task: Comprehensive Document Analysis

Analyze the current document and identify areas that need work. For each issue found:
1. Determine the appropriate stub type
2. Identify the specific location (create an inline anchor)
3. Write a clear, actionable description
4. Explain why this matters for the document's goals

Focus on:
- Missing context or background
- Unsupported claims
- Unclear sections
- Missing connections to related topics
- Gaps in the logical flow

After analysis, present a summary and the proposed stubs. Wait for user approval before making changes.`,
        behavior: {
            confirm_before_apply: true,
            auto_insert_anchors: true,
            show_preview: true,
        },
    },
    {
        id: 'suggest-improvements',
        name: 'Suggest Improvements',
        icon: 'lightbulb',
        description: 'Prioritized improvement suggestions based on document state',
        category: 'analysis',
        task_family: 'synoptic',
        vector_type: 'model',
        reliability: 'medium',
        enabled: true,
        context: {
            requires_selection: false,
            requires_file: true,
            file_types: ['md'],
        },
        system_extension: `## Task: Editorial Improvement Suggestions

Based on the document's current refinement level and target audience, suggest specific improvements:

1. First, calculate the current health and usefulness margins
2. Identify the biggest gaps between current state and the next audience gate
3. Prioritize suggestions by impact on health score
4. For each suggestion, propose a stub with specific guidance

Present suggestions ranked by priority. Do not add stubs automatically - present options for the user to choose from.`,
        behavior: {
            confirm_before_apply: true,
            auto_insert_anchors: true,
            show_preview: true,
        },
    },
    {
        id: 'expand-section',
        name: 'Expand Section',
        icon: 'plus-circle',
        description: 'Expand the selected section with more detail',
        category: 'editing',
        task_family: 'generative',
        vector_type: 'draft',
        reliability: 'medium',
        enabled: true,
        context: {
            requires_selection: true,
            requires_file: true,
            selection_type: 'block',
            file_types: ['md'],
        },
        system_extension: `## Task: Section Expansion Guidance

The user has selected a section they want to expand. Your role is to:

1. Analyze the current section content
2. Identify what's missing or underdeveloped
3. Add \`expand\` stubs with specific guidance on what to add
4. Suggest related documents that might provide useful context

Create inline anchors at specific points where expansion is needed. Each stub should answer: "What specific content should be added here?"`,
        behavior: {
            confirm_before_apply: true,
            auto_insert_anchors: true,
            show_preview: true,
        },
        hotkey: 'Mod+Shift+E',
    },
    {
        id: 'find-citations',
        name: 'Find Citations',
        icon: 'link',
        description: 'Find citations for claims in selection',
        category: 'review',
        task_family: 'combinatorial',
        vector_type: 'source',
        reliability: 'high',
        enabled: true,
        context: {
            requires_selection: true,
            requires_file: true,
            file_types: ['md'],
        },
        system_extension: `## Task: Citation Finding

For the selected text, identify:
1. Claims that need citations
2. Suggest potential sources
3. Add citation stubs with specific guidance

Focus on:
- Factual claims without attribution
- Statistics or data points
- Quotes or paraphrases
- Technical specifications

Each stub should specify what type of citation would be most appropriate.`,
        behavior: {
            confirm_before_apply: false,
            auto_insert_anchors: true,
            show_preview: true,
        },
        hotkey: 'Mod+Shift+C',
    },
    {
        id: 'find-related',
        name: 'Find Related Docs',
        icon: 'file-search',
        description: 'Find related documents in the vault',
        category: 'analysis',
        task_family: 'combinatorial',
        vector_type: 'link',
        reliability: 'high',
        enabled: true,
        context: {
            requires_selection: false,
            requires_file: true,
            file_types: ['md'],
        },
        system_extension: `## Task: Document Connection Analysis

Analyze the current document and find related documents in the vault:

1. Use \`find_related_documents\` to get semantically similar docs
2. Use \`suggest_links\` to find potential wiki-link opportunities
3. For each relevant connection, propose a \`link\` stub
4. Explain how the linked document relates to the current one

Present findings as a list of potential connections with relevance explanations.`,
        behavior: {
            confirm_before_apply: false,
            auto_insert_anchors: false,
            show_preview: true,
        },
    },
    {
        id: 'verify-claims',
        name: 'Verify Claims',
        icon: 'check-circle',
        description: 'Identify claims that need verification',
        category: 'review',
        task_family: 'combinatorial',
        vector_type: 'check',
        reliability: 'high',
        enabled: true,
        context: {
            requires_selection: true,
            requires_file: true,
            file_types: ['md'],
        },
        system_extension: `## Task: Claim Verification

For the selected text, identify claims that need verification:

1. Find statements presented as facts
2. Identify technical claims that could be outdated
3. Flag numbers, statistics, or metrics
4. Note any assertions about other people or organizations

For each claim, add a \`verify\` stub with:
- The specific claim to verify
- Suggested verification method
- Priority based on impact if claim is wrong`,
        behavior: {
            confirm_before_apply: true,
            auto_insert_anchors: true,
            show_preview: true,
        },
    },
    {
        id: 'quick-review',
        name: 'Quick Review',
        icon: 'zap',
        description: 'Fast review for obvious issues',
        category: 'review',
        task_family: 'combinatorial',
        vector_type: 'check',
        reliability: 'high',
        enabled: true,
        context: {
            requires_selection: false,
            requires_file: true,
            file_types: ['md'],
        },
        system_extension: `## Task: Quick Document Review

Perform a fast review to catch obvious issues:

1. Check for incomplete sentences or thoughts
2. Identify placeholder text (TODO, TBD, ???)
3. Find broken or suspicious links
4. Note any obvious formatting issues
5. Flag content that seems out of place

Focus on low-hanging fruit - issues that are quick to identify and have clear fixes.
Limit suggestions to 5-7 most impactful issues.`,
        behavior: {
            confirm_before_apply: false,
            auto_insert_anchors: false,
            show_preview: true,
        },
        hotkey: 'Mod+Shift+R',
    },
    {
        id: 'structure-check',
        name: 'Check Structure',
        icon: 'list-tree',
        description: 'Analyze document structure and organization',
        category: 'analysis',
        task_family: 'synoptic',
        vector_type: 'model',
        reliability: 'medium',
        enabled: true,
        context: {
            requires_selection: false,
            requires_file: true,
            file_types: ['md'],
        },
        system_extension: `## Task: Structure Analysis

Analyze the document's organization and structure:

1. Check heading hierarchy (are levels used correctly?)
2. Evaluate section balance (are some too long/short?)
3. Identify missing standard sections
4. Check logical flow between sections
5. Note any orphaned or misplaced content

Suggest structural improvements as \`revise\` stubs with specific guidance.`,
        behavior: {
            confirm_before_apply: true,
            auto_insert_anchors: true,
            show_preview: true,
        },
    },
    {
        id: 'add-examples',
        name: 'Suggest Examples',
        icon: 'book-open',
        description: 'Identify where examples would help',
        category: 'editing',
        task_family: 'generative',
        vector_type: 'idea',
        reliability: 'low',
        enabled: true,
        context: {
            requires_selection: false,
            requires_file: true,
            file_types: ['md'],
        },
        system_extension: `## Task: Example Identification

Find places where examples would improve understanding:

1. Identify abstract concepts that need illustration
2. Find technical explanations that could use code samples
3. Note generalizations that would benefit from specific cases
4. Suggest what type of example would be most helpful

Add \`example\` stubs with specific guidance on what kind of example to add.`,
        behavior: {
            confirm_before_apply: true,
            auto_insert_anchors: true,
            show_preview: true,
        },
    },
    {
        id: 'define-terms',
        name: 'Define Terms',
        icon: 'book',
        description: 'Find terms that need definition',
        category: 'editing',
        task_family: 'combinatorial',
        vector_type: 'source',
        reliability: 'high',
        enabled: true,
        context: {
            requires_selection: false,
            requires_file: true,
            file_types: ['md'],
        },
        system_extension: `## Task: Term Definition

Identify terms and concepts that need definition:

1. Find jargon or technical terms used without explanation
2. Note acronyms that aren't expanded
3. Identify concepts assumed but not introduced
4. Consider the target audience's likely knowledge level

Add \`define\` stubs for terms that need clarification, with guidance on how detailed the definition should be.`,
        behavior: {
            confirm_before_apply: true,
            auto_insert_anchors: true,
            show_preview: true,
        },
    },
];
