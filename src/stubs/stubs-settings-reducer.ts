/**
 * Stubs Settings Reducer
 *
 * Handles all settings actions for the stubs configuration.
 * Follows the same reducer pattern as the main plugin.
 */

import {
    StubsConfiguration,
    StubTypeDefinition,
    StructuredPropertyDefinition,
} from './stubs-types';
import type { StubsSettingsActions } from './stubs-types';
export type { StubsSettingsActions } from './stubs-types';
import { generateId, getNextSortOrder, getNextDefaultColor } from './stubs-defaults';

/**
 * Update stubs configuration based on action
 */
function updateStubsState(config: StubsConfiguration, action: StubsSettingsActions): void {
    switch (action.type) {
        // =====================================================================
        // STUB TYPE MANAGEMENT (Level 1)
        // =====================================================================

        case 'STUBS_ADD_TYPE': {
            const { key, displayName, color, icon } = action.payload;
            // Validate key doesn't already exist
            const existingKeys = Object.values(config.stubTypes).map((t) => t.key);
            if (existingKeys.includes(key)) {
                console.warn(`Stub type key "${key}" already exists`);
                return;
            }

            const id = generateId();
            const newType: StubTypeDefinition = {
                id,
                key: key.toLowerCase().replace(/\s+/g, '-'),
                displayName,
                color: color || getNextDefaultColor(config),
                icon,
                sortOrder: getNextSortOrder(config),
                defaults: {},
            };
            config.stubTypes[id] = newType;
            break;
        }

        case 'STUBS_UPDATE_TYPE': {
            const { id, updates } = action.payload;
            if (!config.stubTypes[id]) {
                console.warn(`Stub type with id "${id}" not found`);
                return;
            }

            // If key is being updated, validate it doesn't conflict
            if (updates.key) {
                const existingKeys = Object.values(config.stubTypes)
                    .filter((t) => t.id !== id)
                    .map((t) => t.key);
                if (existingKeys.includes(updates.key)) {
                    console.warn(`Stub type key "${updates.key}" already exists`);
                    return;
                }
            }

            // If displayName is being updated, validate it doesn't conflict
            if (updates.displayName) {
                const existingNames = Object.values(config.stubTypes)
                    .filter((t) => t.id !== id)
                    .map((t) => t.displayName.toLowerCase());
                if (existingNames.includes(updates.displayName.toLowerCase())) {
                    console.warn(`Stub type name "${updates.displayName}" already exists`);
                    return;
                }
            }

            config.stubTypes[id] = { ...config.stubTypes[id], ...updates };
            break;
        }

        case 'STUBS_DELETE_TYPE': {
            const { id } = action.payload;
            if (!config.stubTypes[id]) {
                console.warn(`Stub type with id "${id}" not found`);
                return;
            }
            delete config.stubTypes[id];
            break;
        }

        case 'STUBS_REORDER_TYPES': {
            const { orderedIds } = action.payload;
            orderedIds.forEach((id, index) => {
                if (config.stubTypes[id]) {
                    config.stubTypes[id].sortOrder = index + 1;
                }
            });
            break;
        }

        // =====================================================================
        // STRUCTURED PROPERTY MANAGEMENT (Level 2)
        // =====================================================================

        case 'STUBS_ADD_PROPERTY': {
            const { key, displayName, type } = action.payload;
            // Validate key doesn't already exist
            const existingKeys = Object.values(config.structuredProperties).map((p) => p.key);
            if (existingKeys.includes(key)) {
                console.warn(`Property key "${key}" already exists`);
                return;
            }

            const id = generateId();
            const newProperty: StructuredPropertyDefinition = {
                id,
                key: key.toLowerCase().replace(/\s+/g, '_'),
                displayName,
                type,
                sortOrder: Object.keys(config.structuredProperties).length + 1,
                enumValues: type === 'enum' ? [] : undefined,
            };
            config.structuredProperties[id] = newProperty;
            break;
        }

        case 'STUBS_UPDATE_PROPERTY': {
            const { id, updates } = action.payload;
            if (!config.structuredProperties[id]) {
                console.warn(`Property with id "${id}" not found`);
                return;
            }

            // If key is being updated, validate it doesn't conflict
            if (updates.key) {
                const existingKeys = Object.values(config.structuredProperties)
                    .filter((p) => p.id !== id)
                    .map((p) => p.key);
                if (existingKeys.includes(updates.key)) {
                    console.warn(`Property key "${updates.key}" already exists`);
                    return;
                }
            }

            config.structuredProperties[id] = { ...config.structuredProperties[id], ...updates };
            break;
        }

        case 'STUBS_DELETE_PROPERTY': {
            const { id } = action.payload;
            if (!config.structuredProperties[id]) {
                console.warn(`Property with id "${id}" not found`);
                return;
            }
            delete config.structuredProperties[id];
            break;
        }

        case 'STUBS_TOGGLE_PROPERTY_INCLUDE': {
            const { id } = action.payload;
            if (!config.structuredProperties[id]) {
                console.warn(`Property with id "${id}" not found`);
                return;
            }
            const prop = config.structuredProperties[id];
            // Default to true if not set, then toggle
            prop.includeInStructured = !(prop.includeInStructured ?? true);
            break;
        }

        case 'STUBS_ADD_ENUM_VALUE': {
            const { propertyId, value, displayName } = action.payload;
            const property = config.structuredProperties[propertyId];
            if (!property || property.type !== 'enum') {
                console.warn(`Property "${propertyId}" is not an enum type`);
                return;
            }

            if (!property.enumValues) {
                property.enumValues = [];
            }
            if (!property.enumDisplayNames) {
                property.enumDisplayNames = [];
            }

            if (!property.enumValues.includes(value)) {
                property.enumValues.push(value);
                property.enumDisplayNames.push(displayName || value);
            }
            break;
        }

        case 'STUBS_REMOVE_ENUM_VALUE': {
            const { propertyId, value } = action.payload;
            const property = config.structuredProperties[propertyId];
            if (!property || property.type !== 'enum' || !property.enumValues) {
                return;
            }

            const index = property.enumValues.indexOf(value);
            if (index > -1) {
                property.enumValues.splice(index, 1);
                property.enumDisplayNames?.splice(index, 1);
            }
            break;
        }

        // =====================================================================
        // ANCHOR SETTINGS
        // =====================================================================

        case 'STUBS_SET_ANCHOR_PREFIX': {
            const { prefix } = action.payload;
            // Validate prefix (alphanumeric, lowercase)
            const sanitized = prefix.toLowerCase().replace(/[^a-z0-9-]/g, '');
            if (sanitized.length > 0) {
                config.anchors.prefix = sanitized;
            }
            break;
        }

        case 'STUBS_SET_ANCHOR_ID_STYLE': {
            config.anchors.idStyle = action.payload.style;
            break;
        }

        case 'STUBS_SET_ANCHOR_ID_LENGTH': {
            const { length } = action.payload;
            if (length >= 4 && length <= 12) {
                config.anchors.randomIdLength = length;
            }
            break;
        }

        // =====================================================================
        // DECORATION SETTINGS
        // =====================================================================

        case 'STUBS_SET_DECORATIONS_ENABLED': {
            config.decorations.enabled = action.payload.enabled;
            break;
        }

        case 'STUBS_SET_DECORATION_STYLE': {
            config.decorations.style = action.payload.style;
            break;
        }

        case 'STUBS_SET_DECORATION_OPACITY': {
            const { opacity } = action.payload;
            if (opacity >= 0 && opacity <= 1) {
                config.decorations.opacity = opacity;
            }
            break;
        }

        // =====================================================================
        // SIDEBAR SETTINGS
        // =====================================================================

        case 'STUBS_SET_SIDEBAR_FONT_SIZE': {
            const { fontSize } = action.payload;
            if (fontSize >= 8 && fontSize <= 24) {
                config.sidebar.fontSize = fontSize;
            }
            break;
        }

        case 'STUBS_SET_SIDEBAR_EXPANDED_DEFAULT': {
            config.sidebar.expandedByDefault = action.payload.expanded;
            break;
        }

        case 'STUBS_TOGGLE_TYPE_VISIBILITY': {
            const { typeId } = action.payload;
            const index = config.sidebar.hiddenTypes.indexOf(typeId);
            if (index > -1) {
                config.sidebar.hiddenTypes.splice(index, 1);
            } else {
                config.sidebar.hiddenTypes.push(typeId);
            }
            break;
        }

        case 'STUBS_SET_HIDDEN_TYPES': {
            config.sidebar.hiddenTypes = action.payload.hiddenTypes;
            break;
        }

        case 'STUBS_SET_SHOW_SEARCH': {
            config.sidebar.showSearchInput = action.payload.show;
            break;
        }

        case 'STUBS_SET_SHOW_TYPE_FILTER': {
            config.sidebar.showTypeFilter = action.payload.show;
            break;
        }

        // =====================================================================
        // GENERAL SETTINGS
        // =====================================================================

        case 'STUBS_SET_ENABLED': {
            config.enabled = action.payload.enabled;
            break;
        }

        case 'STUBS_SET_FRONTMATTER_KEY': {
            const { key } = action.payload;
            // Validate key (alphanumeric, lowercase, underscores)
            const sanitized = key.toLowerCase().replace(/[^a-z0-9_]/g, '');
            if (sanitized.length > 0) {
                config.frontmatterKey = sanitized;
            }
            break;
        }

        case 'STUBS_SET_INCLUDE_DEFAULT_PROPERTIES': {
            if (!config.structuredStubs) {
                config.structuredStubs = { includeDefaultProperties: true };
            }
            config.structuredStubs.includeDefaultProperties = action.payload.include;
            break;
        }
    }
}

/**
 * Stubs settings reducer
 */
export function stubsSettingsReducer(
    config: StubsConfiguration,
    action: StubsSettingsActions
): StubsConfiguration {
    updateStubsState(config, action);
    return config;
}
