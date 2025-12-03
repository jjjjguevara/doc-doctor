/**
 * MCP Settings Reducer
 *
 * Handles MCP configuration state updates.
 */

import { MCPSettings } from './mcp-types';

/**
 * MCP settings actions
 */
export type MCPSettingsActions =
    | { type: 'MCP_SET_ENABLED'; payload: { enabled: boolean } }
    | { type: 'MCP_SET_BINARY_PATH'; payload: { path: string } }
    | { type: 'MCP_SET_AUTO_CONNECT'; payload: { enabled: boolean } }
    | { type: 'MCP_SET_CONNECTION_TIMEOUT'; payload: { timeout: number } }
    | { type: 'MCP_SET_SHOW_STATUS_BAR'; payload: { enabled: boolean } };

/**
 * MCP settings reducer
 */
export const mcpSettingsReducer = (
    state: MCPSettings,
    action: MCPSettingsActions
): void => {
    switch (action.type) {
        case 'MCP_SET_ENABLED':
            state.enabled = action.payload.enabled;
            break;
        case 'MCP_SET_BINARY_PATH':
            state.binaryPath = action.payload.path;
            break;
        case 'MCP_SET_AUTO_CONNECT':
            state.autoConnect = action.payload.enabled;
            break;
        case 'MCP_SET_CONNECTION_TIMEOUT':
            state.connectionTimeout = action.payload.timeout;
            break;
        case 'MCP_SET_SHOW_STATUS_BAR':
            state.showStatusBar = action.payload.enabled;
            break;
    }
};
