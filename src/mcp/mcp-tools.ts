/**
 * MCP Tools - Typed Wrappers
 *
 * Provides typed access to all 28 MCP tools.
 */

import { MCPClient } from './mcp-client';
import {
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
} from './mcp-types';

/**
 * MCP Tools wrapper for typed tool access
 */
export class MCPTools {
    constructor(private client: MCPClient) {}

    // =========================================================================
    // ANALYSIS OPERATIONS
    // =========================================================================

    /**
     * Parse document content and extract L1 properties
     */
    async parseDocument(content: string): Promise<ParseDocumentResult> {
        return this.client.callTool<ParseDocumentResult>('parse_document', { content });
    }

    /**
     * Full document analysis with L2 dimensions
     */
    async analyzeDocument(content: string): Promise<AnalyzeDocumentResult> {
        return this.client.callTool<AnalyzeDocumentResult>('analyze_document', { content });
    }

    /**
     * Validate frontmatter against schema
     */
    async validateDocument(content: string, strict = false): Promise<ValidateDocumentResult> {
        return this.client.callTool<ValidateDocumentResult>('validate_document', { content, strict });
    }

    /**
     * Read and optionally analyze a document from disk
     */
    async readDocument(path: string, analyze = true): Promise<{
        content: string;
        analysis?: AnalyzeDocumentResult;
    }> {
        return this.client.callTool('read_document', { path, analyze });
    }

    // =========================================================================
    // STUB MANAGEMENT
    // =========================================================================

    /**
     * Add a stub to document frontmatter
     */
    async addStub(
        content: string,
        stubType: string,
        description: string,
        options?: {
            stubForm?: 'transient' | 'persistent' | 'blocking';
            priority?: 'low' | 'medium' | 'high' | 'critical';
            anchor?: string;
        }
    ): Promise<AddStubResult> {
        return this.client.callTool<AddStubResult>('add_stub', {
            content,
            stub_type: stubType,
            description,
            stub_form: options?.stubForm,
            priority: options?.priority,
            anchor: options?.anchor,
        });
    }

    /**
     * Resolve (remove) a stub by index
     */
    async resolveStub(content: string, stubIndex: number): Promise<ResolveStubResult> {
        return this.client.callTool<ResolveStubResult>('resolve_stub', {
            content,
            stub_index: stubIndex,
        });
    }

    /**
     * Update stub properties
     */
    async updateStub(
        content: string,
        stubIndex: number,
        updates: {
            description?: string;
            stubForm?: 'transient' | 'persistent' | 'blocking';
            priority?: 'low' | 'medium' | 'high' | 'critical';
            stubType?: string;
        }
    ): Promise<UpdateStubResult> {
        return this.client.callTool<UpdateStubResult>('update_stub', {
            content,
            stub_index: stubIndex,
            description: updates.description,
            stub_form: updates.stubForm,
            priority: updates.priority,
            stub_type: updates.stubType,
        });
    }

    /**
     * List all stubs in a document
     */
    async listStubs(
        content: string,
        filter?: {
            stubType?: string;
            stubForm?: string;
            priority?: string;
        }
    ): Promise<{ stubs: StubInfo[]; total: number }> {
        return this.client.callTool('list_stubs', {
            content,
            stub_type: filter?.stubType,
            stub_form: filter?.stubForm,
            priority: filter?.priority,
        });
    }

    // =========================================================================
    // ANCHOR MANAGEMENT
    // =========================================================================

    /**
     * Find all inline anchors in document content
     */
    async findStubAnchors(content: string): Promise<FindAnchorsResult> {
        return this.client.callTool<FindAnchorsResult>('find_stub_anchors', { content });
    }

    /**
     * Link a stub to an inline anchor
     */
    async linkStubAnchor(
        content: string,
        stubIndex: number,
        anchorId: string
    ): Promise<AnchorLinkResult> {
        return this.client.callTool<AnchorLinkResult>('link_stub_anchor', {
            content,
            stub_index: stubIndex,
            anchor_id: anchorId,
        });
    }

    /**
     * Unlink a stub from an inline anchor
     */
    async unlinkStubAnchor(
        content: string,
        stubIndex: number,
        anchorId: string
    ): Promise<AnchorLinkResult> {
        return this.client.callTool<AnchorLinkResult>('unlink_stub_anchor', {
            content,
            stub_index: stubIndex,
            anchor_id: anchorId,
        });
    }

    // =========================================================================
    // CALCULATIONS
    // =========================================================================

    /**
     * Calculate health score
     */
    async calculateHealth(refinement: number, stubCount: number, blockingCount: number): Promise<HealthResult> {
        return this.client.callTool<HealthResult>('calculate_health', {
            refinement,
            stub_count: stubCount,
            blocking_count: blockingCount,
        });
    }

    /**
     * Calculate usefulness margin
     */
    async calculateUsefulness(refinement: number, audience: string): Promise<UsefulnessResult> {
        return this.client.callTool<UsefulnessResult>('calculate_usefulness', {
            refinement,
            audience,
        });
    }

    /**
     * Get audience gate thresholds
     */
    async getAudienceGates(): Promise<{
        self: number;
        team: number;
        internal: number;
        external: number;
    }> {
        return this.client.callTool('get_audience_gates', {});
    }

    // =========================================================================
    // VAULT OPERATIONS
    // =========================================================================

    /**
     * Scan vault directory for documents
     */
    async scanVault(
        path: string,
        options?: {
            recursive?: boolean;
            includeAnalysis?: boolean;
            pattern?: string;
        }
    ): Promise<VaultScanResult> {
        return this.client.callTool<VaultScanResult>('scan_vault', {
            path,
            recursive: options?.recursive ?? true,
            include_analysis: options?.includeAnalysis ?? true,
            pattern: options?.pattern,
        });
    }

    /**
     * Find all blocking stubs across documents
     */
    async findBlockingStubs(paths: string[]): Promise<BlockingStubsResult> {
        return this.client.callTool<BlockingStubsResult>('find_blocking_stubs', { paths });
    }

    /**
     * Detect stale documents
     */
    async detectStaleDocuments(
        paths: string[],
        cadences?: Record<string, number>
    ): Promise<{
        stale_documents: Array<{
            path: string;
            form: string;
            days_since_modified: number;
            expected_cadence: number;
        }>;
        total: number;
    }> {
        return this.client.callTool('detect_stale_documents', {
            paths,
            cadences,
        });
    }

    // =========================================================================
    // SCHEMA
    // =========================================================================

    /**
     * Get JSON schema for frontmatter or stubs
     */
    async getSchema(schemaType: 'frontmatter' | 'stub' | 'full'): Promise<string> {
        return this.client.callTool<string>('get_schema', { schema_type: schemaType });
    }

    /**
     * Get stub type definitions
     */
    async getStubTypes(): Promise<{
        types: Array<{
            key: string;
            label: string;
            description: string;
            vector_family: string;
            default_form: string;
        }>;
    }> {
        return this.client.callTool('get_stub_types', {});
    }

    // =========================================================================
    // FRONTMATTER OPERATIONS
    // =========================================================================

    /**
     * Update refinement value
     */
    async updateRefinement(content: string, refinement: number): Promise<{ updated_content: string }> {
        return this.client.callTool('update_refinement', { content, refinement });
    }

    /**
     * Update audience value
     */
    async updateAudience(content: string, audience: string): Promise<{ updated_content: string }> {
        return this.client.callTool('update_audience', { content, audience });
    }

    /**
     * Update form value
     */
    async updateForm(content: string, form: string): Promise<{ updated_content: string }> {
        return this.client.callTool('update_form', { content, form });
    }

    /**
     * Add a reference to frontmatter
     */
    async addReference(content: string, reference: string): Promise<{ updated_content: string }> {
        return this.client.callTool('add_reference', { content, reference });
    }

    /**
     * Remove a reference from frontmatter
     */
    async removeReference(content: string, reference: string): Promise<{ updated_content: string }> {
        return this.client.callTool('remove_reference', { content, reference });
    }

    /**
     * Add a tag to frontmatter
     */
    async addTag(content: string, tag: string): Promise<{ updated_content: string }> {
        return this.client.callTool('add_tag', { content, tag });
    }

    /**
     * Remove a tag from frontmatter
     */
    async removeTag(content: string, tag: string): Promise<{ updated_content: string }> {
        return this.client.callTool('remove_tag', { content, tag });
    }

    // =========================================================================
    // BATCH OPERATIONS
    // =========================================================================

    /**
     * Batch analyze multiple documents
     */
    async batchAnalyze(paths: string[]): Promise<{
        results: Array<{
            path: string;
            analysis?: AnalyzeDocumentResult;
            error?: string;
        }>;
        total: number;
        successful: number;
    }> {
        return this.client.callTool('batch_analyze', { paths });
    }

    /**
     * Suggest improvements for a document
     */
    async suggestImprovements(content: string): Promise<{
        suggestions: Array<{
            type: 'stub' | 'refinement' | 'audience' | 'form';
            description: string;
            priority: 'low' | 'medium' | 'high';
            impact: string;
        }>;
    }> {
        return this.client.callTool('suggest_improvements', { content });
    }
}
