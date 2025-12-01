/**
 * Stubs Settings Tab Component
 *
 * Provides UI for configuring stubs types, properties, and behavior.
 */

import { Setting, ButtonComponent, TextComponent, DropdownComponent } from 'obsidian';
import type LabeledAnnotations from '../../main';
import {
    StubsConfiguration,
    StubTypeDefinition,
    StructuredPropertyDefinition,
} from '../stubs-types';
import { getSortedStubTypes, getSortedProperties } from '../stubs-defaults';

type Props = {
    containerEl: HTMLElement;
    plugin: LabeledAnnotations;
};

/**
 * Main stubs settings section
 */
export const StubsSettings = ({ plugin, containerEl }: Props): void => {
    const config = plugin.settings.getValue().stubs;

    // Main heading
    new Setting(containerEl)
        .setName('Stubs')
        .setHeading()
        .setDesc('Configure stub types, properties, and sync behavior');

    // Enable/Disable stubs
    new Setting(containerEl)
        .setName('Enable stubs')
        .setDesc('Enable or disable the stubs feature entirely')
        .addToggle((toggle) => {
            toggle.setValue(config.enabled).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'STUBS_SET_ENABLED',
                    payload: { enabled: value },
                });
            });
        });

    // Frontmatter key
    new Setting(containerEl)
        .setName('Frontmatter key')
        .setDesc('The YAML frontmatter key used for stubs (default: "stubs")')
        .addText((text) => {
            text.setPlaceholder('stubs')
                .setValue(config.frontmatterKey)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_SET_FRONTMATTER_KEY',
                        payload: { key: value },
                    });
                });
        });

    // Stub Types Section
    renderStubTypesSection({ plugin, containerEl, config });

    // Structured Properties Section
    renderPropertiesSection({ plugin, containerEl, config });

    // Anchor Settings Section
    renderAnchorSettings({ plugin, containerEl, config });

    // Decoration Settings Section
    renderDecorationSettings({ plugin, containerEl, config });

    // Sidebar Settings Section
    renderSidebarSettings({ plugin, containerEl, config });
};

/**
 * Render stub types configuration section
 */
function renderStubTypesSection({
    plugin,
    containerEl,
    config,
}: {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
    config: StubsConfiguration;
}): void {
    new Setting(containerEl).setName('Stub Types').setHeading();

    const typesContainer = containerEl.createDiv('stubs-types-container');

    const sortedTypes = getSortedStubTypes(config);

    for (const typeDef of sortedTypes) {
        renderStubTypeItem({ plugin, containerEl: typesContainer, typeDef });
    }

    // Add new type button
    new Setting(typesContainer).addButton((btn) => {
        btn.setButtonText('Add stub type')
            .setCta()
            .onClick(() => {
                const key = `type-${Date.now()}`;
                plugin.settings.dispatch({
                    type: 'STUBS_ADD_TYPE',
                    payload: {
                        key,
                        displayName: 'New Type',
                        color: '#888888',
                    },
                });
                // Re-render
                typesContainer.empty();
                renderStubTypesSection({ plugin, containerEl: typesContainer, config });
            });
    });
}

/**
 * Render a single stub type configuration item
 */
function renderStubTypeItem({
    plugin,
    containerEl,
    typeDef,
}: {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
    typeDef: StubTypeDefinition;
}): void {
    const typeEl = containerEl.createDiv('stub-type-item');

    // Color indicator
    const colorIndicator = typeEl.createSpan('stub-type-color');
    colorIndicator.style.backgroundColor = typeDef.color;

    // Get current config for duplicate checking
    const currentConfig = plugin.settings.getValue().stubs;

    // Helper to check for duplicate name
    const isDuplicateName = (name: string, excludeId: string) => {
        return Object.values(currentConfig.stubTypes)
            .filter((t) => t.id !== excludeId)
            .some((t) => t.displayName.toLowerCase() === name.toLowerCase());
    };

    // Helper to check for duplicate key
    const isDuplicateKey = (key: string, excludeId: string) => {
        return Object.values(currentConfig.stubTypes)
            .filter((t) => t.id !== excludeId)
            .some((t) => t.key === key.toLowerCase().replace(/\s+/g, '-'));
    };

    // Key/Name settings
    const mainSetting = new Setting(typeEl)
        .setName(typeDef.displayName)
        .setDesc(`Key: ${typeDef.key}`)
        .addText((text) => {
            text.setPlaceholder('Display name')
                .setValue(typeDef.displayName)
                .onChange((value) => {
                    // Check for duplicate before dispatching
                    if (isDuplicateName(value, typeDef.id)) {
                        text.inputEl.style.borderColor = 'var(--text-error)';
                        text.inputEl.title = 'A stub type with this name already exists';
                        return;
                    }
                    text.inputEl.style.borderColor = '';
                    text.inputEl.title = '';

                    plugin.settings.dispatch({
                        type: 'STUBS_UPDATE_TYPE',
                        payload: {
                            id: typeDef.id,
                            updates: { displayName: value },
                        },
                    });
                });
        })
        .addText((text) => {
            text.setPlaceholder('key')
                .setValue(typeDef.key)
                .onChange((value) => {
                    const normalizedKey = value.toLowerCase().replace(/\s+/g, '-');
                    // Check for duplicate before dispatching
                    if (isDuplicateKey(value, typeDef.id)) {
                        text.inputEl.style.borderColor = 'var(--text-error)';
                        text.inputEl.title = 'A stub type with this key already exists';
                        return;
                    }
                    text.inputEl.style.borderColor = '';
                    text.inputEl.title = '';

                    plugin.settings.dispatch({
                        type: 'STUBS_UPDATE_TYPE',
                        payload: {
                            id: typeDef.id,
                            updates: { key: normalizedKey },
                        },
                    });
                });
        })
        .addColorPicker((picker) => {
            picker.setValue(typeDef.color).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'STUBS_UPDATE_TYPE',
                    payload: {
                        id: typeDef.id,
                        updates: { color: value },
                    },
                });
                colorIndicator.style.backgroundColor = value;
            });
        })
        .addButton((btn) => {
            btn.setIcon('trash')
                .setWarning()
                .setTooltip('Delete type')
                .onClick(() => {
                    plugin.settings.dispatch({
                        type: 'STUBS_DELETE_TYPE',
                        payload: { id: typeDef.id },
                    });
                    typeEl.remove();
                });
        });

    // Default description for compact stubs
    const descSetting = new Setting(typeEl)
        .setName('Default description')
        .setDesc('Default text used when creating compact stubs of this type');

    // Create a resizable textarea instead of a text input
    const textareaContainer = descSetting.controlEl.createDiv('stub-description-textarea-container');
    const textarea = textareaContainer.createEl('textarea', {
        cls: 'stub-description-textarea',
        attr: {
            placeholder: 'e.g., Citation needed',
            rows: '2',
        },
    });
    textarea.value = typeDef.defaultStubDescription || '';
    textarea.style.width = '100%';
    textarea.style.minHeight = '60px';
    textarea.style.resize = 'vertical';
    textarea.style.fontFamily = 'inherit';
    textarea.style.fontSize = 'inherit';
    textarea.style.padding = '8px';
    textarea.style.borderRadius = '4px';
    textarea.style.border = '1px solid var(--background-modifier-border)';
    textarea.style.backgroundColor = 'var(--background-primary)';
    textarea.style.color = 'var(--text-normal)';

    textarea.addEventListener('input', () => {
        plugin.settings.dispatch({
            type: 'STUBS_UPDATE_TYPE',
            payload: {
                id: typeDef.id,
                updates: { defaultStubDescription: textarea.value },
            },
        });
    });
}

/**
 * Render structured properties configuration section
 */
function renderPropertiesSection({
    plugin,
    containerEl,
    config,
}: {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
    config: StubsConfiguration;
}): void {
    new Setting(containerEl)
        .setName('Structured Properties')
        .setHeading()
        .setDesc('Properties available in structured stub syntax. Toggle each property to include it when inserting structured stubs (^^^).');

    const propsContainer = containerEl.createDiv('stubs-properties-container');

    const sortedProps = getSortedProperties(config);

    for (const propDef of sortedProps) {
        renderPropertyItem({ plugin, containerEl: propsContainer, propDef, config });
    }

    // Add new property button
    new Setting(propsContainer).addButton((btn) => {
        btn.setButtonText('Add property')
            .setCta()
            .onClick(() => {
                const key = `prop_${Date.now()}`;
                plugin.settings.dispatch({
                    type: 'STUBS_ADD_PROPERTY',
                    payload: {
                        key,
                        displayName: 'New Property',
                        type: 'string',
                    },
                });
                // Re-render
                propsContainer.empty();
                renderPropertiesSection({ plugin, containerEl: propsContainer, config });
            });
    });
}

/**
 * Render a single property configuration item
 */
function renderPropertyItem({
    plugin,
    containerEl,
    propDef,
    config,
}: {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
    propDef: StructuredPropertyDefinition;
    config: StubsConfiguration;
}): void {
    const propEl = containerEl.createDiv('stub-property-item');

    new Setting(propEl)
        .setName(propDef.displayName)
        .setDesc(`Key: ${propDef.key} | Type: ${propDef.type}`)
        .addText((text) => {
            text.setPlaceholder('Display name')
                .setValue(propDef.displayName)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_UPDATE_PROPERTY',
                        payload: {
                            id: propDef.id,
                            updates: { displayName: value },
                        },
                    });
                });
        })
        .addText((text) => {
            text.setPlaceholder('key')
                .setValue(propDef.key)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_UPDATE_PROPERTY',
                        payload: {
                            id: propDef.id,
                            updates: { key: value.toLowerCase().replace(/\s+/g, '_') },
                        },
                    });
                });
        })
        .addDropdown((dropdown) => {
            dropdown
                .addOptions({
                    string: 'String',
                    enum: 'Enum',
                    array: 'Array',
                    boolean: 'Boolean',
                    number: 'Number',
                })
                .setValue(propDef.type)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_UPDATE_PROPERTY',
                        payload: {
                            id: propDef.id,
                            updates: { type: value as StructuredPropertyDefinition['type'] },
                        },
                    });
                });
        })
        .addToggle((toggle) => {
            toggle
                .setTooltip('Include in structured stubs (^^^)')
                .setValue(propDef.includeInStructured ?? true)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_UPDATE_PROPERTY',
                        payload: {
                            id: propDef.id,
                            updates: { includeInStructured: value },
                        },
                    });
                });
        })
        .addButton((btn) => {
            btn.setIcon('trash')
                .setWarning()
                .setTooltip('Delete property')
                .onClick(() => {
                    plugin.settings.dispatch({
                        type: 'STUBS_DELETE_PROPERTY',
                        payload: { id: propDef.id },
                    });
                    propEl.remove();
                });
        });

    // Type-specific default value editors
    if (propDef.type === 'enum') {
        renderEnumValuesEditor({ plugin, containerEl: propEl, propDef });
    } else if (propDef.type === 'string') {
        new Setting(propEl)
            .setName('Default value')
            .setDesc('Default text value when creating structured stubs')
            .addText((text) => {
                text.setPlaceholder('Default text...')
                    .setValue(String(propDef.defaultValue ?? ''))
                    .onChange((value) => {
                        plugin.settings.dispatch({
                            type: 'STUBS_UPDATE_PROPERTY',
                            payload: {
                                id: propDef.id,
                                updates: { defaultValue: value || undefined },
                            },
                        });
                    });
            });
    } else if (propDef.type === 'number') {
        new Setting(propEl)
            .setName('Default value')
            .setDesc('Default number value when creating structured stubs')
            .addText((text) => {
                text.setPlaceholder('0')
                    .setValue(propDef.defaultValue !== undefined ? String(propDef.defaultValue) : '')
                    .onChange((value) => {
                        const numValue = value ? parseFloat(value) : undefined;
                        plugin.settings.dispatch({
                            type: 'STUBS_UPDATE_PROPERTY',
                            payload: {
                                id: propDef.id,
                                updates: { defaultValue: isNaN(numValue as number) ? undefined : numValue },
                            },
                        });
                    });
            });
    } else if (propDef.type === 'boolean') {
        new Setting(propEl)
            .setName('Default value')
            .setDesc('Default boolean value when creating structured stubs')
            .addToggle((toggle) => {
                toggle
                    .setValue(propDef.defaultValue === true)
                    .onChange((value) => {
                        plugin.settings.dispatch({
                            type: 'STUBS_UPDATE_PROPERTY',
                            payload: {
                                id: propDef.id,
                                updates: { defaultValue: value },
                            },
                        });
                    });
            });
    }
    // Array type doesn't need a default value editor - empty array is used
}

/**
 * Render enum values editor for enum properties
 */
function renderEnumValuesEditor({
    plugin,
    containerEl,
    propDef,
}: {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
    propDef: StructuredPropertyDefinition;
}): void {
    const enumContainer = containerEl.createDiv('enum-values-container');

    const enumValues = propDef.enumValues || [];
    const enumDisplayNames = propDef.enumDisplayNames || [];

    // Default value selector
    if (enumValues.length > 0) {
        new Setting(enumContainer)
            .setName('Default value')
            .setDesc('Default value used when creating structured stubs')
            .addDropdown((dropdown) => {
                dropdown.addOption('', '(none)');
                for (let i = 0; i < enumValues.length; i++) {
                    const value = enumValues[i];
                    const displayName = enumDisplayNames[i] || value;
                    dropdown.addOption(value, displayName);
                }
                dropdown.setValue(String(propDef.defaultValue ?? ''));
                dropdown.onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_UPDATE_PROPERTY',
                        payload: {
                            id: propDef.id,
                            updates: { defaultValue: value || undefined },
                        },
                    });
                });
            });
    }

    // Enum values list
    enumContainer.createEl('label', { text: 'Enum values:' });
    const valuesList = enumContainer.createDiv('enum-values-list');

    for (let i = 0; i < enumValues.length; i++) {
        const value = enumValues[i];
        const displayName = enumDisplayNames[i] || value;
        const isDefault = propDef.defaultValue === value;

        const valueEl = valuesList.createDiv('enum-value-item');
        valueEl.createSpan({ text: `${displayName} (${value})${isDefault ? ' ✓' : ''}` });

        const removeBtn = valueEl.createEl('button', { text: '×' });
        removeBtn.onclick = () => {
            plugin.settings.dispatch({
                type: 'STUBS_REMOVE_ENUM_VALUE',
                payload: {
                    propertyId: propDef.id,
                    value,
                },
            });
            valueEl.remove();
        };
    }

    // Add new enum value
    const addContainer = enumContainer.createDiv('add-enum-value');
    let newValue = '';
    let newDisplayName = '';

    new Setting(addContainer)
        .addText((text) => {
            text.setPlaceholder('value').onChange((v) => {
                newValue = v;
            });
        })
        .addText((text) => {
            text.setPlaceholder('Display name').onChange((v) => {
                newDisplayName = v;
            });
        })
        .addButton((btn) => {
            btn.setButtonText('Add').onClick(() => {
                if (newValue) {
                    plugin.settings.dispatch({
                        type: 'STUBS_ADD_ENUM_VALUE',
                        payload: {
                            propertyId: propDef.id,
                            value: newValue,
                            displayName: newDisplayName || newValue,
                        },
                    });
                }
            });
        });
}

/**
 * Render anchor settings section
 */
function renderAnchorSettings({
    plugin,
    containerEl,
    config,
}: {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
    config: StubsConfiguration;
}): void {
    new Setting(containerEl).setName('Anchor Settings').setHeading();

    // Anchor prefix
    new Setting(containerEl)
        .setName('Anchor prefix')
        .setDesc('Prefix for generated anchor IDs (default: "stub")')
        .addText((text) => {
            text.setPlaceholder('stub')
                .setValue(config.anchors.prefix)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_SET_ANCHOR_PREFIX',
                        payload: { prefix: value },
                    });
                });
        });

    // ID style
    new Setting(containerEl)
        .setName('ID generation style')
        .setDesc('How to generate new anchor IDs')
        .addDropdown((dropdown) => {
            dropdown
                .addOptions({
                    random: 'Random (e.g., ^stub-a1b2c3)',
                    sequential: 'Sequential (e.g., ^stub-001)',
                    'type-prefixed': 'Type-prefixed (e.g., ^stub-link-a1b2)',
                })
                .setValue(config.anchors.idStyle)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_SET_ANCHOR_ID_STYLE',
                        payload: { style: value as 'random' | 'sequential' | 'type-prefixed' },
                    });
                });
        });

    // Random ID length
    new Setting(containerEl)
        .setName('Random ID length')
        .setDesc('Length of random portion in anchor IDs (4-12)')
        .addSlider((slider) => {
            slider
                .setLimits(4, 12, 1)
                .setValue(config.anchors.randomIdLength)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_SET_ANCHOR_ID_LENGTH',
                        payload: { length: value },
                    });
                });
        });
}

/**
 * Render decoration settings section
 */
function renderDecorationSettings({
    plugin,
    containerEl,
    config,
}: {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
    config: StubsConfiguration;
}): void {
    new Setting(containerEl).setName('Inline Decorations').setHeading();

    // Enable decorations
    new Setting(containerEl)
        .setName('Enable decorations')
        .setDesc('Highlight stub anchors in the editor')
        .addToggle((toggle) => {
            toggle.setValue(config.decorations.enabled).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'STUBS_SET_DECORATIONS_ENABLED',
                    payload: { enabled: value },
                });
            });
        });

    // Decoration style
    new Setting(containerEl)
        .setName('Decoration style')
        .setDesc('How to display stub anchors in the editor')
        .addDropdown((dropdown) => {
            dropdown
                .addOptions({
                    background: 'Background highlight',
                    underline: 'Underline',
                    badge: 'Badge/chip',
                    gutter: 'Gutter icon only',
                })
                .setValue(config.decorations.style)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_SET_DECORATION_STYLE',
                        payload: {
                            style: value as 'background' | 'underline' | 'badge' | 'gutter',
                        },
                    });
                });
        });

    // Decoration opacity
    new Setting(containerEl)
        .setName('Decoration opacity')
        .setDesc('Opacity of anchor highlights (0.1-1.0)')
        .addSlider((slider) => {
            slider
                .setLimits(0.1, 1.0, 0.1)
                .setValue(config.decorations.opacity)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_SET_DECORATION_OPACITY',
                        payload: { opacity: value },
                    });
                });
        });
}

/**
 * Render sidebar settings section
 */
function renderSidebarSettings({
    plugin,
    containerEl,
    config,
}: {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
    config: StubsConfiguration;
}): void {
    new Setting(containerEl).setName('Sidebar Settings').setHeading();

    // Font size
    new Setting(containerEl)
        .setName('Font size')
        .setDesc('Font size for stub items in sidebar (8-24)')
        .addSlider((slider) => {
            slider
                .setLimits(8, 24, 1)
                .setValue(config.sidebar.fontSize)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'STUBS_SET_SIDEBAR_FONT_SIZE',
                        payload: { fontSize: value },
                    });
                });
        });

    // Expanded by default
    new Setting(containerEl)
        .setName('Expand types by default')
        .setDesc('Automatically expand all type groups in sidebar')
        .addToggle((toggle) => {
            toggle.setValue(config.sidebar.expandedByDefault).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'STUBS_SET_SIDEBAR_EXPANDED_DEFAULT',
                    payload: { expanded: value },
                });
            });
        });

    // Show search input
    new Setting(containerEl)
        .setName('Show search input')
        .setDesc('Display search input in sidebar controls')
        .addToggle((toggle) => {
            toggle.setValue(config.sidebar.showSearchInput).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'STUBS_SET_SHOW_SEARCH',
                    payload: { show: value },
                });
            });
        });

    // Show type filter
    new Setting(containerEl)
        .setName('Show type filter')
        .setDesc('Display type filter in sidebar controls')
        .addToggle((toggle) => {
            toggle.setValue(config.sidebar.showTypeFilter).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'STUBS_SET_SHOW_TYPE_FILTER',
                    payload: { show: value },
                });
            });
        });
}
