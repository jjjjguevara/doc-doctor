# Specification: YAML Frontmatter Parsing

**Version**: 0.1.0
**Date**: 2025-12-01
**Status**: Draft

---

## 1. Overview

This document specifies how the stubs feature parses YAML frontmatter from Obsidian documents. The parser must handle:

1. **Configurable top-level key** (default: `stubs`)
2. **Compact syntax**: `- type: "description"`
3. **Structured syntax**: `- type: { description: "...", ... }`
4. **Mixed syntax**: Both formats in same document
5. **Unknown types**: Graceful handling with warnings
6. **Malformed entries**: Skip invalid entries without breaking

---

## 2. Syntax Specification

### 2.1 Compact Syntax

The simplest form uses the stub type as the YAML key with a string value:

```yaml
stubs:
  - link: "Add citation for OAuth 2.0 specification"
  - expand: "Add deployment architecture diagram"
  - question: "Should we support legacy authentication?"
```

**Grammar**:
```
stub_entry_compact := stub_type_key ':' WHITESPACE? description_string
stub_type_key := IDENTIFIER  # lowercase, alphanumeric, hyphen, underscore
description_string := QUOTED_STRING | UNQUOTED_STRING
```

**Parsing rules**:
- The single key in the object is the stub type
- The string value is the description
- All other properties use defaults from configuration

### 2.2 Structured Syntax

For stubs requiring additional properties:

```yaml
stubs:
  - controversy:
      description: "Pricing model disagreement between teams"
      stub_form: blocking
      priority: high
      assignees: ["[[Alice]]", "[[Bob]]"]
      inline_anchors: [^pricing-section, ^revenue-model]
      references: ["[[Finance Policy]]", "[[Sales Playbook]]"]
```

**Grammar**:
```
stub_entry_structured := stub_type_key ':' NEWLINE INDENT properties
properties := property+
property := property_key ':' property_value
property_key := IDENTIFIER
property_value := STRING | NUMBER | BOOLEAN | ARRAY | NULL
```

**Parsing rules**:
- The single key in the outer object is the stub type
- The `description` property is required
- Other properties are optional, validated against configuration
- Unknown properties are preserved but not displayed

### 2.3 Mixed Syntax

Documents can contain both compact and structured stubs:

```yaml
stubs:
  - link: "Simple citation stub"
  - expand: "Another simple stub"
  - controversy:
      description: "Complex stub with metadata"
      stub_form: blocking
      assignees: ["[[Alice]]"]
  - question: "Back to compact syntax"
```

**Parsing rules**:
- Each array entry is parsed independently
- Detection: if value is string → compact; if value is object → structured
- Order is preserved for display

---

## 3. Parser Implementation

### 3.1 Core Algorithm

```typescript
interface ParsedStub {
  id: string;
  type: string;
  description: string;
  properties: Record<string, unknown>;
  syntax: 'compact' | 'structured';
  position: {
    lineStart: number;
    lineEnd: number;
  };
  warnings: string[];
}

interface ParseResult {
  stubs: ParsedStub[];
  errors: ParseError[];
  warnings: ParseWarning[];
}

function parseStubsFrontmatter(
  frontmatter: Record<string, unknown>,
  config: StubsConfiguration
): ParseResult {
  const result: ParseResult = {
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
      message: `"${stubsKey}" must be an array`,
    });
    return result;
  }

  // Case 4: Parse each entry
  for (let i = 0; i < stubsArray.length; i++) {
    const entry = stubsArray[i];
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
  }

  return result;
}
```

### 3.2 Entry Parser

```typescript
function parseStubEntry(
  entry: unknown,
  index: number,
  config: StubsConfiguration
): { stub?: ParsedStub; error?: ParseError; warnings?: ParseWarning[] } {
  const warnings: ParseWarning[] = [];

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

  const keys = Object.keys(entry);

  // Must have exactly one key (the stub type)
  if (keys.length !== 1) {
    return {
      error: {
        type: 'invalid_entry',
        index,
        message: `Entry at index ${index} must have exactly one key (the stub type)`,
      },
    };
  }

  const stubType = keys[0];
  const value = (entry as Record<string, unknown>)[stubType];

  // Check if type is known
  const typeConfig = config.stubTypes.find(t => t.key === stubType);
  if (!typeConfig) {
    warnings.push({
      type: 'unknown_type',
      index,
      stubType,
      message: `Unknown stub type "${stubType}" at index ${index}`,
    });
  }

  // Determine syntax and parse
  if (typeof value === 'string') {
    // Compact syntax
    return {
      stub: {
        id: generateStubId(stubType, value, index),
        type: stubType,
        description: value,
        properties: typeConfig?.defaults ?? {},
        syntax: 'compact',
        position: { lineStart: -1, lineEnd: -1 }, // Populated later
        warnings: warnings.map(w => w.message),
      },
      warnings,
    };
  }

  if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
    // Structured syntax
    const props = value as Record<string, unknown>;

    if (!props.description || typeof props.description !== 'string') {
      return {
        error: {
          type: 'missing_description',
          index,
          message: `Structured stub at index ${index} missing required "description" property`,
        },
      };
    }

    // Validate properties against configuration
    const validatedProps = validateProperties(props, config.structuredProperties, warnings, index);

    return {
      stub: {
        id: generateStubId(stubType, props.description as string, index),
        type: stubType,
        description: props.description as string,
        properties: { ...(typeConfig?.defaults ?? {}), ...validatedProps },
        syntax: 'structured',
        position: { lineStart: -1, lineEnd: -1 },
        warnings: warnings.map(w => w.message),
      },
      warnings,
    };
  }

  // Invalid value type
  return {
    error: {
      type: 'invalid_value',
      index,
      message: `Value for "${stubType}" at index ${index} must be a string or object`,
    },
  };
}
```

### 3.3 Property Validation

```typescript
function validateProperties(
  props: Record<string, unknown>,
  definitions: StructuredPropertyDefinition[],
  warnings: ParseWarning[],
  index: number
): Record<string, unknown> {
  const validated: Record<string, unknown> = {};
  const knownKeys = new Set(definitions.map(d => d.key));
  knownKeys.add('description'); // Always valid

  for (const [key, value] of Object.entries(props)) {
    if (key === 'description') {
      continue; // Handled separately
    }

    if (!knownKeys.has(key)) {
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

    const def = definitions.find(d => d.key === key)!;
    const validationResult = validatePropertyValue(value, def);

    if (validationResult.valid) {
      validated[key] = validationResult.value;
    } else {
      warnings.push({
        type: 'invalid_property_value',
        index,
        property: key,
        message: validationResult.message,
      });
    }
  }

  return validated;
}

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
      if (!def.enumValues?.includes(value)) {
        return {
          valid: false,
          message: `"${def.key}" must be one of: ${def.enumValues?.join(', ')}`,
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
```

### 3.4 ID Generation

```typescript
function generateStubId(type: string, description: string, index: number): string {
  // Create deterministic ID from content
  const content = `${type}:${description}`;
  const hash = simpleHash(content);
  return `stub-${type}-${hash.slice(0, 8)}`;
}

function simpleHash(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  return Math.abs(hash).toString(16).padStart(8, '0');
}
```

---

## 4. Wikilink Handling

### 4.1 Quoted Wikilinks

Obsidian requires wikilinks in YAML to be quoted for clickability:

```yaml
assignees: ["[[Alice]]", "[[Bob]]"]
inline_anchors: ["[[Doc#^section]]"]
references: ["[[Related Document]]"]
```

### 4.2 Unquoting for Display

```typescript
function unquoteWikilink(value: string): string {
  return value
    .trim()
    .replace(/^"(.*)"$/, '$1')  // Remove outer quotes
    .replace(/^\[\[(.*)\]\]$/, '$1');  // Remove wikilink brackets (for display)
}

function preserveWikilink(value: string): string {
  // Keep the wikilink syntax but remove quotes
  return value.trim().replace(/^"(.*)"$/, '$1');
}
```

### 4.3 Array Processing

```typescript
function processAssignees(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value
    .filter((v): v is string => typeof v === 'string')
    .map(preserveWikilink);
}

function processAnchors(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value
    .filter((v): v is string => typeof v === 'string')
    .map(v => {
      // Handle both "^anchor" and "[[Doc#^anchor]]" formats
      const trimmed = v.trim().replace(/^"(.*)"$/, '$1');
      if (trimmed.startsWith('^')) {
        return trimmed;
      }
      // Extract anchor from wikilink
      const match = trimmed.match(/#(\^[a-zA-Z0-9_-]+)/);
      return match ? match[1] : trimmed;
    });
}
```

---

## 5. Position Tracking

### 5.1 Line Number Resolution

To enable click-to-navigate, we need to track where stubs appear in frontmatter:

```typescript
interface PositionInfo {
  lineStart: number;  // Line where stub entry starts
  lineEnd: number;    // Line where stub entry ends
}

function parseWithPositions(
  content: string,
  config: StubsConfiguration
): ParseResult {
  // Extract frontmatter
  const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);
  if (!frontmatterMatch) {
    return { stubs: [], errors: [], warnings: [] };
  }

  const frontmatterContent = frontmatterMatch[1];
  const frontmatterStartLine = 1; // After opening ---

  // Parse YAML
  const parsed = yaml.load(frontmatterContent) as Record<string, unknown>;

  // Parse stubs
  const result = parseStubsFrontmatter(parsed, config);

  // Resolve positions by re-scanning frontmatter
  resolvePositions(result.stubs, frontmatterContent, frontmatterStartLine, config);

  return result;
}

function resolvePositions(
  stubs: ParsedStub[],
  frontmatterContent: string,
  startLine: number,
  config: StubsConfiguration
): void {
  const lines = frontmatterContent.split('\n');
  const stubsKey = config.frontmatterKey;

  let inStubsArray = false;
  let currentStubIndex = 0;
  let currentStubStartLine = -1;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // Detect stubs array start
    if (trimmed.startsWith(`${stubsKey}:`)) {
      inStubsArray = true;
      continue;
    }

    if (!inStubsArray) continue;

    // Detect end of stubs array (unindented line or new key)
    if (trimmed && !line.startsWith(' ') && !line.startsWith('\t')) {
      break;
    }

    // Detect stub entry start (- key:)
    if (trimmed.startsWith('-')) {
      // Close previous stub
      if (currentStubStartLine !== -1 && currentStubIndex < stubs.length) {
        stubs[currentStubIndex - 1].position.lineEnd = startLine + i - 1;
      }

      currentStubStartLine = startLine + i;
      if (currentStubIndex < stubs.length) {
        stubs[currentStubIndex].position.lineStart = currentStubStartLine;
      }
      currentStubIndex++;
    }
  }

  // Close last stub
  if (currentStubStartLine !== -1 && currentStubIndex <= stubs.length) {
    stubs[currentStubIndex - 1].position.lineEnd = startLine + lines.length - 1;
  }
}
```

---

## 6. Error Handling

### 6.1 Error Types

```typescript
type ParseErrorType =
  | 'invalid_format'      // stubs key is not an array
  | 'invalid_entry'       // entry is not an object or has wrong structure
  | 'missing_description' // structured entry lacks description
  | 'invalid_value'       // value is neither string nor object
  | 'yaml_error';         // underlying YAML parse error

type ParseWarningType =
  | 'unknown_type'           // stub type not in configuration
  | 'unknown_property'       // property not in configuration
  | 'invalid_property_value' // property value doesn't match type
  | 'deprecated_syntax';     // using old syntax format

interface ParseError {
  type: ParseErrorType;
  index?: number;
  message: string;
  line?: number;
}

interface ParseWarning {
  type: ParseWarningType;
  index?: number;
  stubType?: string;
  property?: string;
  message: string;
}
```

### 6.2 Error Recovery

The parser should be resilient:

```typescript
function parseWithRecovery(
  frontmatter: Record<string, unknown>,
  config: StubsConfiguration
): ParseResult {
  const result: ParseResult = {
    stubs: [],
    errors: [],
    warnings: [],
  };

  try {
    const stubsArray = frontmatter[config.frontmatterKey];

    if (!Array.isArray(stubsArray)) {
      // Try to recover from common mistakes
      if (typeof stubsArray === 'object' && stubsArray !== null) {
        // User might have used object instead of array
        result.errors.push({
          type: 'invalid_format',
          message: `"${config.frontmatterKey}" must be an array. Did you forget the "-" prefix?`,
        });
      }
      return result;
    }

    // Parse each entry independently (don't let one bad entry break others)
    for (let i = 0; i < stubsArray.length; i++) {
      try {
        const parsed = parseStubEntry(stubsArray[i], i, config);
        if (parsed.stub) result.stubs.push(parsed.stub);
        if (parsed.error) result.errors.push(parsed.error);
        if (parsed.warnings) result.warnings.push(...parsed.warnings);
      } catch (err) {
        result.errors.push({
          type: 'invalid_entry',
          index: i,
          message: `Failed to parse entry at index ${i}: ${err}`,
        });
        // Continue with next entry
      }
    }

    return result;
  } catch (err) {
    result.errors.push({
      type: 'yaml_error',
      message: `YAML parsing failed: ${err}`,
    });
    return result;
  }
}
```

---

## 7. Test Cases

### 7.1 Happy Path Tests

```typescript
describe('Stubs Parser', () => {
  describe('compact syntax', () => {
    it('parses single compact stub', () => {
      const yaml = `
stubs:
  - link: "Add citation"
`;
      const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);

      expect(result.stubs).toHaveLength(1);
      expect(result.stubs[0]).toMatchObject({
        type: 'link',
        description: 'Add citation',
        syntax: 'compact',
      });
      expect(result.errors).toHaveLength(0);
    });

    it('parses multiple compact stubs', () => {
      const yaml = `
stubs:
  - link: "Citation 1"
  - expand: "More content"
  - question: "Open question"
`;
      const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);
      expect(result.stubs).toHaveLength(3);
    });
  });

  describe('structured syntax', () => {
    it('parses structured stub with all properties', () => {
      const yaml = `
stubs:
  - controversy:
      description: "Team disagreement"
      stub_form: blocking
      priority: high
      assignees: ["[[Alice]]", "[[Bob]]"]
`;
      const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);

      expect(result.stubs[0]).toMatchObject({
        type: 'controversy',
        description: 'Team disagreement',
        syntax: 'structured',
        properties: {
          stub_form: 'blocking',
          priority: 'high',
          assignees: ['[[Alice]]', '[[Bob]]'],
        },
      });
    });
  });

  describe('mixed syntax', () => {
    it('parses mixed compact and structured stubs', () => {
      const yaml = `
stubs:
  - link: "Simple stub"
  - controversy:
      description: "Complex stub"
      stub_form: blocking
  - expand: "Another simple"
`;
      const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);

      expect(result.stubs).toHaveLength(3);
      expect(result.stubs[0].syntax).toBe('compact');
      expect(result.stubs[1].syntax).toBe('structured');
      expect(result.stubs[2].syntax).toBe('compact');
    });
  });
});
```

### 7.2 Edge Case Tests

```typescript
describe('edge cases', () => {
  it('handles missing stubs key', () => {
    const yaml = `title: "No stubs"`;
    const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);
    expect(result.stubs).toHaveLength(0);
    expect(result.errors).toHaveLength(0);
  });

  it('handles empty stubs array', () => {
    const yaml = `stubs: []`;
    const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);
    expect(result.stubs).toHaveLength(0);
  });

  it('handles null stubs value', () => {
    const yaml = `stubs: null`;
    const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);
    expect(result.stubs).toHaveLength(0);
  });

  it('warns on unknown stub type', () => {
    const yaml = `
stubs:
  - custom_type: "Unknown type"
`;
    const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);
    expect(result.stubs).toHaveLength(1);
    expect(result.warnings).toContainEqual(
      expect.objectContaining({ type: 'unknown_type' })
    );
  });

  it('skips malformed entry without breaking others', () => {
    const yaml = `
stubs:
  - link: "Valid stub"
  - invalid_entry  # Not an object
  - expand: "Also valid"
`;
    const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);
    expect(result.stubs).toHaveLength(2);
    expect(result.errors).toHaveLength(1);
  });
});
```

### 7.3 Property Validation Tests

```typescript
describe('property validation', () => {
  it('validates enum values', () => {
    const yaml = `
stubs:
  - link:
      description: "Test"
      stub_form: invalid_value
`;
    const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);
    expect(result.warnings).toContainEqual(
      expect.objectContaining({
        type: 'invalid_property_value',
        property: 'stub_form',
      })
    );
  });

  it('preserves unknown properties with warning', () => {
    const yaml = `
stubs:
  - link:
      description: "Test"
      custom_field: "preserved"
`;
    const result = parseStubsFrontmatter(parseYaml(yaml), defaultConfig);
    expect(result.stubs[0].properties.custom_field).toBe('preserved');
    expect(result.warnings).toContainEqual(
      expect.objectContaining({ type: 'unknown_property' })
    );
  });
});
```

---

## 8. See Also

- [PRD-stubs-support.md](./PRD-stubs-support.md) - Product requirements
- [SPEC-configuration-schema.md](./SPEC-configuration-schema.md) - Configuration schema

---

**Document History**:
- 2025-12-01: Initial draft
