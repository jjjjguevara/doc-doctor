/**
 * Smart Connections Settings Reducer
 *
 * Handles settings updates for the Smart Connections integration.
 */

import type { SmartConnectionsSettings } from './types';

export type SmartConnectionsSettingsActions =
    | { type: 'SET_SMART_CONNECTIONS_ENABLED'; payload: { enabled: boolean } }
    | { type: 'SET_SMART_CONNECTIONS_AUTO_POPULATE'; payload: { enabled: boolean } }
    | { type: 'SET_SMART_CONNECTIONS_WARN_DUPLICATES'; payload: { enabled: boolean } }
    | { type: 'SET_SMART_CONNECTIONS_SUGGEST_LINKS'; payload: { enabled: boolean } }
    | { type: 'SET_SMART_CONNECTIONS_RELATED_LIMIT'; payload: { limit: number } }
    | { type: 'SET_SMART_CONNECTIONS_RELATED_THRESHOLD'; payload: { threshold: number } }
    | { type: 'SET_SMART_CONNECTIONS_RELATED_PROPERTY_NAME'; payload: { propertyName: string } }
    | { type: 'SET_SMART_CONNECTIONS_DUPLICATE_THRESHOLD'; payload: { threshold: number } }
    | { type: 'SET_SMART_CONNECTIONS_LINK_CONFIDENCE'; payload: { confidence: number } }
    | { type: 'SET_SMART_CONNECTIONS_CACHE_ENABLED'; payload: { enabled: boolean } }
    | { type: 'SET_SMART_CONNECTIONS_CACHE_DURATION'; payload: { minutes: number } };

export const smartConnectionsSettingsReducer = (
    settings: SmartConnectionsSettings,
    action: SmartConnectionsSettingsActions,
): void => {
    switch (action.type) {
        case 'SET_SMART_CONNECTIONS_ENABLED':
            settings.enabled = action.payload.enabled;
            break;
        case 'SET_SMART_CONNECTIONS_AUTO_POPULATE':
            settings.autoPopulateRelated = action.payload.enabled;
            break;
        case 'SET_SMART_CONNECTIONS_WARN_DUPLICATES':
            settings.warnOnDuplicates = action.payload.enabled;
            break;
        case 'SET_SMART_CONNECTIONS_SUGGEST_LINKS':
            settings.suggestLinks = action.payload.enabled;
            break;
        case 'SET_SMART_CONNECTIONS_RELATED_LIMIT':
            settings.relatedNotesLimit = action.payload.limit;
            break;
        case 'SET_SMART_CONNECTIONS_RELATED_THRESHOLD':
            settings.relatedThreshold = action.payload.threshold;
            break;
        case 'SET_SMART_CONNECTIONS_RELATED_PROPERTY_NAME':
            settings.relatedPropertyName = action.payload.propertyName;
            break;
        case 'SET_SMART_CONNECTIONS_DUPLICATE_THRESHOLD':
            settings.duplicateThreshold = action.payload.threshold;
            break;
        case 'SET_SMART_CONNECTIONS_LINK_CONFIDENCE':
            settings.linkSuggestionMinConfidence = action.payload.confidence;
            break;
        case 'SET_SMART_CONNECTIONS_CACHE_ENABLED':
            settings.cacheResults = action.payload.enabled;
            break;
        case 'SET_SMART_CONNECTIONS_CACHE_DURATION':
            settings.cacheDurationMinutes = action.payload.minutes;
            break;
    }
};
