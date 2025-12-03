/**
 * MCP Module - Public Exports
 *
 * MCP client for communicating with doc-doctor-mcp binary.
 */

export { MCPClient } from './mcp-client';
export { MCPTools } from './mcp-tools';

// Settings reducer
export type { MCPSettingsActions } from './mcp-settings-reducer';
export { mcpSettingsReducer } from './mcp-settings-reducer';

// Configuration
export type { MCPClientConfig, MCPSettings } from './mcp-types';
export { DEFAULT_MCP_CONFIG, DEFAULT_MCP_SETTINGS, BINARY_SEARCH_PATHS } from './mcp-types';

// Protocol types
export type {
    MCPTool,
    MCPToolResult,
    MCPPropertySchema,
    JsonRpcRequest,
    JsonRpcResponse,
    JsonRpcError,
} from './mcp-types';

// Connection types
export type { MCPConnectionState, MCPClientEvent, MCPClientEventHandler } from './mcp-types';

// Result types
export type {
    ParseDocumentResult,
    AnalyzeDocumentResult,
    ValidateDocumentResult,
    AddStubResult,
    ResolveStubResult,
    UpdateStubResult,
    AnchorLinkResult,
    FindAnchorsResult,
    HealthResult,
    UsefulnessResult,
    VaultScanResult,
    BlockingStubsResult,
    StubInfo,
    AnchorInfo,
    VaultDocument,
    ValidationError,
    ValidationWarning,
} from './mcp-types';
