# Specification: Stubs Configuration Schema

**Version**: 0.1.0
**Date**: 2025-12-01
**Status**: Draft

---

## 1. Overview

This document specifies the configuration schema for the stubs feature. The design prioritizes:

1. **Flexibility**: Users define their own vocabulary
2. **Simplicity**: Sensible defaults for quick start
3. **Extensibility**: Schema supports future enhancements

---

## 2. Configuration Storage

Configuration is stored in Obsidian's plugin data file:

```
.obsidian/plugins/enhanced-annotations/data.json
```

The stubs configuration lives under the `stubs` key in the main settings object.

---

## 3. Schema Definition

### 3.1 TypeScript Interfaces

```typescript
/**
 * Root configuration for stubs feature
 */
interface StubsConfiguration {
  /**
   * Whether stubs feature is enabled
   * @default true
   */
  enabled: boolean;

  /**
   * The frontmatter key used for stubs array
   * @default "stubs"
   */
  frontmatterKey: string;

  /**
   * Defined stub types
   * @default DEFAULT_STUB_TYPES (see section 4)
   */
  stubTypes: StubTypeDefinition[];

  /**
   * Properties available in structured stub syntax
   * @default DEFAULT_STRUCTURED_PROPERTIES (see section 4)
   */
  structuredProperties: StructuredPropertyDefinition[];

  /**
   * Editor decoration settings
   */
  decorations: DecorationSettings;

  /**
   * Auto-suggest settings for stub insertion
   */
  autoSuggest: AutoSuggestSettings;

  /**
   * Sidebar display settings
   */
  sidebar: SidebarSettings;
}

/**
 * Definition for a stub type
 */
interface StubTypeDefinition {
  /**
   * The YAML key used in frontmatter
   * Must be a valid YAML key (lowercase, no spaces, alphanumeric + hyphen/underscore)
   * @example "link", "expand", "citation-needed"
   */
  key: string;

  /**
   * Human-readable display name for UI
   * @example "Citation Needed", "Expand Section"
   */
  displayName: string;

  /**
   * Color for sidebar indicator and decorations
   * Accepts: hex (#fff, #ffffff), rgb(r,g,b), hsl(h,s,l), CSS color names
   * @example "#e67e22", "rgb(230, 126, 34)", "orange"
   */
  color: string;

  /**
   * Optional Lucide icon name
   * @see https://lucide.dev/icons
   * @example "link", "help-circle", "alert-triangle"
   */
  icon?: string;

  /**
   * Optional description shown in tooltips
   * @example "Mark content that needs a citation or reference"
   */
  description?: string;

  /**
   * Default values for structured properties when creating this type
   * @example { "stub_form": "blocking" }
   */
  defaults?: Record<string, unknown>;

  /**
   * Sort order in UI (lower = first)
   * @default 0
   */
  sortOrder?: number;
}

/**
 * Definition for a structured property
 */
interface StructuredPropertyDefinition {
  /**
   * The YAML key used in structured stubs
   * @example "stub_form", "priority", "assignees"
   */
  key: string;

  /**
   * Human-readable display name
   * @example "Form", "Priority", "Assignees"
   */
  displayName: string;

  /**
   * Property value type
   */
  type: 'string' | 'enum' | 'array' | 'boolean' | 'number';

  /**
   * For enum type: allowed values
   * @example ["transient", "persistent", "blocking", "structural"]
   */
  enumValues?: string[];

  /**
   * For enum type: display names for values (parallel array)
   * @example ["Transient", "Persistent", "Blocking", "Structural"]
   */
  enumDisplayNames?: string[];

  /**
   * Whether this property is required in structured syntax
   * @default false
   */
  required?: boolean;

  /**
   * Default value when not specified
   */
  defaultValue?: unknown;

  /**
   * Description shown in UI
   */
  description?: string;

  /**
   * Whether to show in creation modal
   * @default true
   */
  showInModal?: boolean;
}

/**
 * Editor decoration settings
 */
interface DecorationSettings {
  /**
   * Whether to show inline decorations in editor
   * @default true
   */
  enabled: boolean;

  /**
   * Decoration style
   * - "gutter": Icon in editor gutter
   * - "highlight": Background highlight on anchor line
   * - "underline": Underline anchor text
   * - "badge": Small badge after anchor
   * @default "gutter"
   */
  style: 'gutter' | 'highlight' | 'underline' | 'badge';

  /**
   * Opacity for decorations (0-1)
   * @default 0.8
   */
  opacity: number;

  /**
   * Show tooltip on hover
   * @default true
   */
  showTooltip: boolean;
}

/**
 * Auto-suggest settings
 */
interface AutoSuggestSettings {
  /**
   * Whether stub auto-suggest is enabled
   * @default true
   */
  enabled: boolean;

  /**
   * Trigger phrase for stub suggestions
   * @default "//stub"
   */
  triggerPhrase: string;

  /**
   * Whether to auto-insert anchor at cursor when creating stub
   * @default "prompt" - ask user
   */
  autoInsertAnchor: 'always' | 'never' | 'prompt';

  /**
   * Anchor ID prefix for auto-generated anchors
   * @default "stub"
   */
  anchorPrefix: string;
}

/**
 * Sidebar display settings
 */
interface SidebarSettings {
  /**
   * Default expanded state for type groups
   * @default true
   */
  expandedByDefault: boolean;

  /**
   * Whether to show empty type groups
   * @default false
   */
  showEmptyGroups: boolean;

  /**
   * Sort stubs within groups
   * - "document": Order of appearance in document
   * - "alphabetical": Alphabetical by description
   * - "priority": By priority property (if present)
   * @default "document"
   */
  sortOrder: 'document' | 'alphabetical' | 'priority';

  /**
   * Truncate description at this length (0 = no truncation)
   * @default 50
   */
  descriptionMaxLength: number;

  /**
   * Show stub count badge on sidebar icon
   * @default true
   */
  showCountBadge: boolean;
}
```

### 3.2 JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Stubs Configuration",
  "type": "object",
  "properties": {
    "enabled": {
      "type": "boolean",
      "default": true
    },
    "frontmatterKey": {
      "type": "string",
      "default": "stubs",
      "pattern": "^[a-z][a-z0-9_-]*$"
    },
    "stubTypes": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["key", "displayName", "color"],
        "properties": {
          "key": {
            "type": "string",
            "pattern": "^[a-z][a-z0-9_-]*$"
          },
          "displayName": {
            "type": "string",
            "minLength": 1
          },
          "color": {
            "type": "string"
          },
          "icon": {
            "type": "string"
          },
          "description": {
            "type": "string"
          },
          "defaults": {
            "type": "object"
          },
          "sortOrder": {
            "type": "number",
            "default": 0
          }
        }
      }
    },
    "structuredProperties": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["key", "displayName", "type"],
        "properties": {
          "key": {
            "type": "string",
            "pattern": "^[a-z][a-z0-9_-]*$"
          },
          "displayName": {
            "type": "string"
          },
          "type": {
            "type": "string",
            "enum": ["string", "enum", "array", "boolean", "number"]
          },
          "enumValues": {
            "type": "array",
            "items": { "type": "string" }
          },
          "enumDisplayNames": {
            "type": "array",
            "items": { "type": "string" }
          },
          "required": {
            "type": "boolean",
            "default": false
          },
          "defaultValue": {},
          "description": {
            "type": "string"
          },
          "showInModal": {
            "type": "boolean",
            "default": true
          }
        }
      }
    },
    "decorations": {
      "type": "object",
      "properties": {
        "enabled": {
          "type": "boolean",
          "default": true
        },
        "style": {
          "type": "string",
          "enum": ["gutter", "highlight", "underline", "badge"],
          "default": "gutter"
        },
        "opacity": {
          "type": "number",
          "minimum": 0,
          "maximum": 1,
          "default": 0.8
        },
        "showTooltip": {
          "type": "boolean",
          "default": true
        }
      }
    },
    "autoSuggest": {
      "type": "object",
      "properties": {
        "enabled": {
          "type": "boolean",
          "default": true
        },
        "triggerPhrase": {
          "type": "string",
          "default": "//stub"
        },
        "autoInsertAnchor": {
          "type": "string",
          "enum": ["always", "never", "prompt"],
          "default": "prompt"
        },
        "anchorPrefix": {
          "type": "string",
          "default": "stub"
        }
      }
    },
    "sidebar": {
      "type": "object",
      "properties": {
        "expandedByDefault": {
          "type": "boolean",
          "default": true
        },
        "showEmptyGroups": {
          "type": "boolean",
          "default": false
        },
        "sortOrder": {
          "type": "string",
          "enum": ["document", "alphabetical", "priority"],
          "default": "document"
        },
        "descriptionMaxLength": {
          "type": "number",
          "default": 50
        },
        "showCountBadge": {
          "type": "boolean",
          "default": true
        }
      }
    }
  }
}
```

---

## 4. Default Configuration

### 4.1 Default Stub Types

Based on J-Editorial Tier 1 types, but generic enough for any workflow:

```typescript
const DEFAULT_STUB_TYPES: StubTypeDefinition[] = [
  {
    key: 'link',
    displayName: 'Citation Needed',
    color: '#e67e22',
    icon: 'link',
    description: 'Content needs a citation or reference',
    defaults: { stub_form: 'persistent' },
    sortOrder: 1,
  },
  {
    key: 'clarify',
    displayName: 'Clarify',
    color: '#3498db',
    icon: 'help-circle',
    description: 'Content is ambiguous and needs clarification',
    defaults: { stub_form: 'transient' },
    sortOrder: 2,
  },
  {
    key: 'expand',
    displayName: 'Expand',
    color: '#2ecc71',
    icon: 'plus-circle',
    description: 'Section needs more detail or content',
    defaults: { stub_form: 'transient' },
    sortOrder: 3,
  },
  {
    key: 'question',
    displayName: 'Question',
    color: '#9b59b6',
    icon: 'message-circle',
    description: 'Open question that needs research or decision',
    defaults: { stub_form: 'transient' },
    sortOrder: 4,
  },
  {
    key: 'controversy',
    displayName: 'Controversy',
    color: '#e74c3c',
    icon: 'alert-triangle',
    description: 'Conflicting perspectives or unresolved disagreement',
    defaults: { stub_form: 'blocking' },
    sortOrder: 5,
  },
  {
    key: 'blocker',
    displayName: 'Blocker',
    color: '#c0392b',
    icon: 'octagon',
    description: 'Cannot proceed until this is resolved',
    defaults: { stub_form: 'blocking' },
    sortOrder: 6,
  },
];
```

### 4.2 Default Structured Properties

```typescript
const DEFAULT_STRUCTURED_PROPERTIES: StructuredPropertyDefinition[] = [
  {
    key: 'stub_form',
    displayName: 'Form',
    type: 'enum',
    enumValues: ['transient', 'persistent', 'blocking', 'structural'],
    enumDisplayNames: ['Transient', 'Persistent', 'Blocking', 'Structural'],
    required: false,
    defaultValue: 'transient',
    description: 'Expected lifecycle and severity of the stub',
    showInModal: true,
  },
  {
    key: 'priority',
    displayName: 'Priority',
    type: 'enum',
    enumValues: ['low', 'medium', 'high', 'critical'],
    enumDisplayNames: ['Low', 'Medium', 'High', 'Critical'],
    required: false,
    description: 'Urgency level for resolution',
    showInModal: true,
  },
  {
    key: 'assignees',
    displayName: 'Assignees',
    type: 'array',
    required: false,
    description: 'People responsible for resolution (wikilinks)',
    showInModal: false, // Advanced, hide by default
  },
  {
    key: 'inline_anchors',
    displayName: 'Anchors',
    type: 'array',
    required: false,
    description: 'Block anchors linking to content locations',
    showInModal: false, // Managed automatically
  },
  {
    key: 'references',
    displayName: 'References',
    type: 'array',
    required: false,
    description: 'Related documents or external links',
    showInModal: false,
  },
];
```

### 4.3 Complete Default Configuration

```typescript
const DEFAULT_STUBS_CONFIGURATION: StubsConfiguration = {
  enabled: true,
  frontmatterKey: 'stubs',
  stubTypes: DEFAULT_STUB_TYPES,
  structuredProperties: DEFAULT_STRUCTURED_PROPERTIES,
  decorations: {
    enabled: true,
    style: 'gutter',
    opacity: 0.8,
    showTooltip: true,
  },
  autoSuggest: {
    enabled: true,
    triggerPhrase: '//stub',
    autoInsertAnchor: 'prompt',
    anchorPrefix: 'stub',
  },
  sidebar: {
    expandedByDefault: true,
    showEmptyGroups: false,
    sortOrder: 'document',
    descriptionMaxLength: 50,
    showCountBadge: true,
  },
};
```

---

## 5. Validation Rules

### 5.1 Stub Type Key Validation

```typescript
function isValidStubTypeKey(key: string): boolean {
  // Must start with lowercase letter
  // Can contain lowercase letters, numbers, hyphens, underscores
  // No spaces or special characters
  return /^[a-z][a-z0-9_-]*$/.test(key);
}

// Valid: "link", "citation-needed", "stub_001"
// Invalid: "Link", "citation needed", "123stub", ""
```

### 5.2 Color Validation

```typescript
function isValidColor(color: string): boolean {
  // Hex: #fff, #ffffff, #ffffffff
  if (/^#([0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(color)) {
    return true;
  }

  // RGB/RGBA
  if (/^rgba?\(\s*\d+\s*,\s*\d+\s*,\s*\d+(,\s*[\d.]+)?\s*\)$/.test(color)) {
    return true;
  }

  // HSL/HSLA
  if (/^hsla?\(\s*\d+\s*,\s*\d+%\s*,\s*\d+%(,\s*[\d.]+)?\s*\)$/.test(color)) {
    return true;
  }

  // CSS color names (subset)
  const cssColors = ['red', 'blue', 'green', 'orange', 'purple', 'yellow', ...];
  if (cssColors.includes(color.toLowerCase())) {
    return true;
  }

  return false;
}
```

### 5.3 Configuration Migration

When loading configuration, apply migrations for backward compatibility:

```typescript
function migrateConfiguration(stored: unknown): StubsConfiguration {
  const config = stored as Partial<StubsConfiguration>;

  // Version 0 → 1: Add sidebar settings
  if (!config.sidebar) {
    config.sidebar = DEFAULT_STUBS_CONFIGURATION.sidebar;
  }

  // Merge with defaults for any missing fields
  return deepMerge(DEFAULT_STUBS_CONFIGURATION, config);
}
```

---

## 6. Export/Import Format

### 6.1 Export Schema

```typescript
interface StubsConfigurationExport {
  /**
   * Export format version
   */
  version: string;

  /**
   * Export timestamp (ISO 8601)
   */
  exportDate: string;

  /**
   * Plugin version that created export
   */
  pluginVersion: string;

  /**
   * The configuration data
   */
  configuration: Partial<StubsConfiguration>;
}
```

### 6.2 Example Export

```json
{
  "version": "1.0.0",
  "exportDate": "2025-12-01T10:30:00Z",
  "pluginVersion": "0.2.0",
  "configuration": {
    "frontmatterKey": "stubs",
    "stubTypes": [
      {
        "key": "link",
        "displayName": "Citation Needed",
        "color": "#e67e22",
        "icon": "link"
      },
      {
        "key": "custom-type",
        "displayName": "My Custom Type",
        "color": "#8e44ad",
        "icon": "star"
      }
    ],
    "structuredProperties": [
      {
        "key": "stub_form",
        "displayName": "Form",
        "type": "enum",
        "enumValues": ["transient", "persistent", "blocking"]
      }
    ]
  }
}
```

### 6.3 Import Behavior

When importing:

1. Validate export format version
2. Validate all stub type keys and colors
3. Merge with existing configuration (user choice: replace or merge)
4. Preserve any custom types not in import
5. Show summary of changes before applying

---

## 7. Settings UI Binding

### 7.1 Settings Component Props

```typescript
interface StubTypeSettingsProps {
  stubTypes: StubTypeDefinition[];
  onChange: (types: StubTypeDefinition[]) => void;
  onAdd: () => void;
  onEdit: (index: number) => void;
  onDelete: (index: number) => void;
  onReorder: (fromIndex: number, toIndex: number) => void;
}

interface PropertySettingsProps {
  properties: StructuredPropertyDefinition[];
  onChange: (props: StructuredPropertyDefinition[]) => void;
  // ... similar to StubTypeSettingsProps
}
```

### 7.2 Validation Feedback

Settings UI should show validation errors inline:

```
Type Key
┌─────────────────────────────────────┐
│ My Type                             │  ← Invalid
└─────────────────────────────────────┘
⚠️ Key must be lowercase with no spaces. Use "my-type" instead.
```

---

## 8. See Also

- [PRD-stubs-support.md](./PRD-stubs-support.md) - Product requirements
- [SPEC-yaml-parsing.md](./SPEC-yaml-parsing.md) - YAML parsing specification

---

**Document History**:
- 2025-12-01: Initial draft
