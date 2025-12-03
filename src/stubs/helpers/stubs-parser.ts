/**
 * Stubs YAML Frontmatter Parser
 *
 * Parses the stubs array from YAML frontmatter, supporting both
 * compact and structured syntax.
 *
 * Compact syntax:
 *   - link: "Add citation for OAuth spec"
 *     anchor: "^stub-abc123"
 *
 * Structured syntax:
 *   - controversy:
 *       description: "Pricing model disagreement"
 *       stub_form: blocking
 *       anchor: "^stub-pricing"
 */

import {
    ParsedStub,
    StubParseResult,
    StubParseError,
    StubParseWarning,
    StubsConfiguration,
    StructuredPropertyDefinition,
} from '../stubs-types';
import { getStubTypeByKey, getPropertyByKey } from '../stubs-defaults';

// =============================================================================
// MAIN PARSER
// =============================================================================

/**
 * Parse stubs from frontmatter object
 */
export function parseStubsFrontmatter(
    frontmatter: Record<string, unknown>,
    config: StubsConfiguration
): StubParseResult {
    const result: StubParseResult = {
        stubs: [],
        errors: [],
        warnings: [],
    };

    const stubsKey = config.frontmatterKey;
    const stubsArray = frontmatter[stubsKey];

    // Case 1: Key not present
    if (stubsArray === undefined) {
        return result;
    }

    // Case 2: Key present but null/empty
    if (stubsArray === null || stubsArray === '') {
        return result;
    }

    // Case 3: Not an array
    if (!Array.isArray(stubsArray)) {
        result.errors.push({
            type: 'invalid_format',
            message: `"${stubsKey}" must be an array, got ${typeof stubsArray}`,
        });
        return result;
    }

    // Case 4: Parse each entry
    for (let i = 0; i < stubsArray.length; i++) {
        const entry = stubsArray[i];
        try {
            const parsed = parseStubEntry(entry, i, config);

            if (parsed.stub) {
                result.stubs.push(parsed.stub);
            }
            if (parsed.error) {
                result.errors.push(parsed.error);
            }
            if (parsed.warnings) {
                result.warnings.push(...parsed.warnings);
            }
        } catch (error) {
            result.errors.push({
                type: 'invalid_entry',
                index: i,
                message: `Failed to parse stub at index ${i}: ${error}`,
            });
        }
    }

    return result;
}

// =============================================================================
// ENTRY PARSER
// =============================================================================

interface EntryParseResult {
    stub?: ParsedStub;
    error?: StubParseError;
    warnings?: StubParseWarning[];
}

/**
 * Parse a single stub entry from the array
 */
function parseStubEntry(
    entry: unknown,
    index: number,
    config: StubsConfiguration
): EntryParseResult {
    const warnings: StubParseWarning[] = [];

    // Must be an object
    if (typeof entry !== 'object' || entry === null || Array.isArray(entry)) {
        return {
            error: {
                type: 'invalid_entry',
                index,
                message: `Entry at index ${index} must be an object`,
            },
        };
    }

    const entryObj = entry as Record<string, unknown>;
    const keys = Object.keys(entryObj);

    // Check for EXPLICIT format first: { type: "link", description: "..." }
    // This is the alternative format where 'type' is an explicit key
    if ('type' in entryObj && typeof entryObj.type === 'string' && 'description' in entryObj) {
        const explicitType = entryObj.type as string;
        const description = entryObj.description;

        if (typeof description !== 'string') {
            return {
                error: {
                    type: 'invalid_value',
                    index,
                    message: `Entry at index ${index} has non-string description`,
                },
            };
        }

        const anchor = extractAnchor(entryObj);
        const typeConfig = getStubTypeByKey(config, explicitType);

        if (!typeConfig) {
            warnings.push({
                type: 'unknown_type',
                index,
                stubType: explicitType,
                message: `Unknown stub type "${explicitType}" at index ${index}`,
            });
        }

        // Extract all other properties (exclude type, description, anchor)
        const props: Record<string, unknown> = {};
        for (const [key, value] of Object.entries(entryObj)) {
            if (key !== 'type' && key !== 'description' && key !== 'anchor') {
                props[key] = value;
            }
        }

        const validatedProps = validateProperties(props, config.structuredProperties, warnings, index);

        return {
            stub: {
                id: generateStubId(explicitType, description, index),
                type: explicitType,
                description: description,
                anchor,
                anchorResolved: false,
                properties: { ...(typeConfig?.defaults ?? {}), ...validatedProps },
                syntax: 'explicit', // New syntax type for explicit format
                frontmatterLine: -1,
                warnings: warnings.map((w) => w.message),
            },
            warnings,
        };
    }

    // Find the stub type key (should be one of the configured types, or 'anchor')
    const typeKey = findStubTypeKey(keys, config);

    if (!typeKey) {
        // No recognized stub type key found
        return {
            error: {
                type: 'invalid_entry',
                index,
                message: `Entry at index ${index} has no recognized stub type key. Keys found: ${keys.join(', ')}`,
            },
        };
    }

    const value = entryObj[typeKey];
    const anchor = extractAnchor(entryObj);

    // Check if type is known
    const typeConfig = getStubTypeByKey(config, typeKey);
    if (!typeConfig) {
        warnings.push({
            type: 'unknown_type',
            index,
            stubType: typeKey,
            message: `Unknown stub type "${typeKey}" at index ${index}`,
        });
    }

    // Determine syntax and parse
    if (typeof value === 'string') {
        // Compact syntax: { link: "description", anchor: "^stub-xxx" }
        return {
            stub: {
                id: generateStubId(typeKey, value, index),
                type: typeKey,
                description: value,
                anchor,
                anchorResolved: false,
                properties: typeConfig?.defaults ?? {},
                syntax: 'compact',
                frontmatterLine: -1, // Will be populated later if needed
                warnings: warnings.map((w) => w.message),
            },
            warnings,
        };
    }

    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
        // Structured syntax: { link: { description: "...", stub_form: "...", anchor: "..." } }
        const props = value as Record<string, unknown>;

        // Description is required
        if (!props.description || typeof props.description !== 'string') {
            return {
                error: {
                    type: 'missing_description',
                    index,
                    message: `Structured stub at index ${index} missing required "description" property`,
                },
            };
        }

        // Extract anchor from inside structured value if present, else use top-level
        const structuredAnchor = extractAnchorFromValue(props) || anchor;

        // Validate properties against configuration
        const validatedProps = validateProperties(props, config.structuredProperties, warnings, index);

        return {
            stub: {
                id: generateStubId(typeKey, props.description as string, index),
                type: typeKey,
                description: props.description as string,
                anchor: structuredAnchor,
                anchorResolved: false,
                properties: { ...(typeConfig?.defaults ?? {}), ...validatedProps },
                syntax: 'structured',
                frontmatterLine: -1,
                warnings: warnings.map((w) => w.message),
            },
            warnings,
        };
    }

    // Invalid value type
    return {
        error: {
            type: 'invalid_value',
            index,
            message: `Value for "${typeKey}" at index ${index} must be a string or object`,
        },
    };
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Find the stub type key from object keys
 */
function findStubTypeKey(keys: string[], config: StubsConfiguration): string | null {
    // Get all configured type keys
    const typeKeys = Object.values(config.stubTypes).map((t) => t.key);

    // Find first key that matches a type
    for (const key of keys) {
        if (typeKeys.includes(key)) {
            return key;
        }
    }

    // If no configured type found, check if any key looks like a type
    // (not 'anchor', 'description', or known property keys)
    const propertyKeys = Object.values(config.structuredProperties).map((p) => p.key);
    const reservedKeys = ['anchor', 'description', ...propertyKeys];

    for (const key of keys) {
        if (!reservedKeys.includes(key)) {
            return key;
        }
    }

    return null;
}

/**
 * Extract anchor from entry object (top-level 'anchor' property)
 */
function extractAnchor(entry: Record<string, unknown>): string | null {
    const anchor = entry.anchor;
    if (typeof anchor === 'string' && anchor.startsWith('^')) {
        return anchor;
    }
    return null;
}

/**
 * Extract anchor from structured value object
 */
function extractAnchorFromValue(value: Record<string, unknown>): string | null {
    const anchor = value.anchor;
    if (typeof anchor === 'string' && anchor.startsWith('^')) {
        return anchor;
    }
    return null;
}

/**
 * Validate structured properties against configuration
 */
function validateProperties(
    props: Record<string, unknown>,
    definitions: Record<string, StructuredPropertyDefinition>,
    warnings: StubParseWarning[],
    index: number
): Record<string, unknown> {
    const validated: Record<string, unknown> = {};
    const knownKeys = new Set(Object.values(definitions).map((d) => d.key));
    knownKeys.add('description'); // Always valid
    knownKeys.add('anchor'); // Always valid

    for (const [key, value] of Object.entries(props)) {
        if (key === 'description' || key === 'anchor') {
            continue; // Handled separately
        }

        const def = getPropertyDefinitionByKey(definitions, key);

        if (!def) {
            warnings.push({
                type: 'unknown_property',
                index,
                property: key,
                message: `Unknown property "${key}" at index ${index}`,
            });
            // Preserve unknown properties
            validated[key] = value;
            continue;
        }

        const validationResult = validatePropertyValue(value, def);

        if (validationResult.valid) {
            validated[key] = validationResult.value;
        } else {
            warnings.push({
                type: 'invalid_property_value',
                index,
                property: key,
                message: `${validationResult.message} at index ${index}`,
            });
        }
    }

    return validated;
}

/**
 * Get property definition by key
 */
function getPropertyDefinitionByKey(
    definitions: Record<string, StructuredPropertyDefinition>,
    key: string
): StructuredPropertyDefinition | undefined {
    return Object.values(definitions).find((d) => d.key === key);
}

/**
 * Validate a property value against its definition
 */
function validatePropertyValue(
    value: unknown,
    def: StructuredPropertyDefinition
): { valid: boolean; value?: unknown; message?: string } {
    switch (def.type) {
        case 'string':
            if (typeof value !== 'string') {
                return { valid: false, message: `"${def.key}" must be a string` };
            }
            return { valid: true, value };

        case 'enum':
            if (typeof value !== 'string') {
                return { valid: false, message: `"${def.key}" must be a string` };
            }
            if (def.enumValues && !def.enumValues.includes(value)) {
                return {
                    valid: false,
                    message: `"${def.key}" must be one of: ${def.enumValues.join(', ')}`,
                };
            }
            return { valid: true, value };

        case 'array':
            if (!Array.isArray(value)) {
                return { valid: false, message: `"${def.key}" must be an array` };
            }
            return { valid: true, value };

        case 'boolean':
            if (typeof value !== 'boolean') {
                return { valid: false, message: `"${def.key}" must be a boolean` };
            }
            return { valid: true, value };

        case 'number':
            if (typeof value !== 'number') {
                return { valid: false, message: `"${def.key}" must be a number` };
            }
            return { valid: true, value };

        default:
            return { valid: true, value };
    }
}

/**
 * Generate a deterministic stub ID from content
 */
function generateStubId(type: string, description: string, index: number): string {
    const content = `${type}:${description}:${index}`;
    const hash = simpleHash(content);
    return `stub-${type}-${hash}`;
}

/**
 * Simple hash function for ID generation
 */
function simpleHash(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i);
        hash = ((hash << 5) - hash) + char;
        hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(16).padStart(8, '0').slice(0, 8);
}

// =============================================================================
// WIKILINK HANDLING
// =============================================================================

/**
 * Remove wikilink brackets and quotes: "[[Page]]" -> "Page"
 */
export function unquoteWikilink(link: string): string {
    return link
        .trim()
        .replace(/^"(.*)"$/, '$1') // Remove outer quotes
        .replace(/^\[\[|\]\]$/g, ''); // Remove wikilink brackets
}

/**
 * Process an array of wikilinks
 */
export function processWikilinks(value: unknown): string[] {
    if (!Array.isArray(value)) return [];
    return value
        .filter((v): v is string => typeof v === 'string')
        .map((v) => v.trim().replace(/^"(.*)"$/, '$1')); // Keep wikilink syntax, remove quotes
}
