//! MCP Tool Registry and Handlers
//!
//! Exposes doc-doctor functionality as MCP tools via the Application Switchboard.
//!
//! # Architecture
//!
//! The ToolRegistry uses the Switchboard pattern - all tools delegate to a
//! centralized switchboard instead of implementing their own logic. This ensures:
//!
//! - Single source of truth for all operations
//! - Consistent behavior between CLI, MCP, and WASM
//! - Easy testing and mocking

mod handlers;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use doc_doctor_application::{
    ApplicationSwitchboard, NewStub, StubFilter, StubUpdates, Switchboard,
};
use doc_doctor_domain::EmbeddedSchemaProvider;
use doc_doctor_parser_yaml::YamlParser;

use crate::integrations::git::GitIntegration;
use crate::integrations::smart_connections::SmartConnectionsIntegration;
use crate::protocol::McpTool;

/// Tool handler function type
type ToolHandler = Box<dyn Fn(serde_json::Value) -> Result<String, String> + Send + Sync>;

/// Registry of available MCP tools
///
/// All tools delegate to the Application Switchboard for consistent behavior
/// across CLI, MCP, and WASM interfaces.
pub struct ToolRegistry {
    tools: HashMap<String, (McpTool, ToolHandler)>,
    switchboard: Arc<ApplicationSwitchboard<YamlParser, YamlParser, EmbeddedSchemaProvider>>,
    git: Arc<GitIntegration>,
    smart_connections: Arc<std::sync::RwLock<SmartConnectionsIntegration>>,
}

impl ToolRegistry {
    /// Create a new tool registry with all tools registered
    pub fn new() -> Self {
        let parser = Arc::new(YamlParser::new());
        let writer = Arc::clone(&parser);
        let schema_provider = Arc::new(EmbeddedSchemaProvider);

        let switchboard = Arc::new(ApplicationSwitchboard::new(parser, writer, schema_provider));
        let git = Arc::new(GitIntegration::new());
        let smart_connections = Arc::new(std::sync::RwLock::new(SmartConnectionsIntegration::new()));

        let mut registry = Self {
            tools: HashMap::new(),
            switchboard,
            git,
            smart_connections,
        };

        // Register all tools
        registry.register_tools();
        registry
    }

    /// Register all available tools
    fn register_tools(&mut self) {
        // Analysis tools
        self.register_parse_document();
        self.register_analyze_document();
        self.register_validate_document();
        self.register_list_stubs();
        self.register_find_stub_anchors();

        // Stub management tools
        self.register_add_stub();
        self.register_resolve_stub();
        self.register_update_stub();
        self.register_link_stub_anchor();
        self.register_unlink_stub_anchor();

        // Calculation tools
        self.register_calculate_health();
        self.register_calculate_usefulness();
        self.register_calculate_dimensions();
        self.register_calculate_vector_physics();

        // Information tools
        self.register_get_audience_gates();
        self.register_get_schema();

        // Batch tools
        self.register_batch_analyze();

        // File system tools
        self.register_read_document();
        self.register_scan_vault();
        self.register_find_blocking_stubs();

        // Git integration tools
        self.register_snapshot_before_edit();
        self.register_commit_stub_resolution();
        self.register_get_document_history();
        self.register_diff_document_versions();

        // RAG/Smart Connections tools
        self.register_find_related_documents();
        self.register_suggest_links();
        self.register_detect_duplicates();
        self.register_draft_with_context();
    }

    /// Register a tool
    fn register(&mut self, tool: McpTool, handler: ToolHandler) {
        self.tools.insert(tool.name.clone(), (tool, handler));
    }

    /// List all available tools
    pub fn list_tools(&self) -> Vec<&McpTool> {
        self.tools.values().map(|(tool, _)| tool).collect()
    }

    /// Call a tool by name
    pub fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<String, String> {
        match self.tools.get(name) {
            Some((_, handler)) => handler(arguments),
            None => Err(format!("Unknown tool: {}", name)),
        }
    }

    // =========================================================================
    // Tool Registrations - Using Switchboard
    // =========================================================================

    fn register_parse_document(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "parse_document",
            "Parse a markdown document and extract L1 properties (refinement, audience, stubs, etc.)",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    }
                },
                "required": ["content"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let props = switchboard
                .parse_document(content)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&props).map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_analyze_document(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "analyze_document",
            "Analyze a document: parse L1 properties and calculate L2 dimensions (health, usefulness, freshness)",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    }
                },
                "required": ["content"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let analysis = switchboard
                .analyze_document(content)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&serde_json::json!({
                "properties": analysis.properties,
                "dimensions": analysis.dimensions,
                "warnings": analysis.warnings,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_validate_document(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "validate_document",
            "Validate document frontmatter against the J-Editorial schema",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    },
                    "strict": {
                        "type": "boolean",
                        "description": "Reject unknown fields (default: false)",
                        "default": false
                    }
                },
                "required": ["content"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let strict = args
                .get("strict")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let result = switchboard
                .validate_document(content, strict)
                .map_err(|e| e.to_string())?;

            // Manually build JSON since ValidationResult doesn't derive Serialize
            let errors: Vec<serde_json::Value> = result.errors.iter().map(|e| {
                serde_json::json!({
                    "path": e.path,
                    "message": e.message,
                    "position": e.position.as_ref().map(|p| format!("line {}, col {}", p.line, p.column))
                })
            }).collect();

            let warnings: Vec<serde_json::Value> = result
                .warnings
                .iter()
                .map(|w| {
                    serde_json::json!({
                        "path": w.path,
                        "message": w.message,
                        "suggestion": w.suggestion
                    })
                })
                .collect();

            serde_json::to_string_pretty(&serde_json::json!({
                "is_valid": result.is_valid,
                "errors": errors,
                "warnings": warnings
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_list_stubs(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "list_stubs",
            "List all stubs (gaps/issues) from a document with their properties",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    },
                    "type_filter": {
                        "type": "string",
                        "description": "Filter by stub type (e.g., 'link', 'expand', 'fix')"
                    },
                    "blocking_only": {
                        "type": "boolean",
                        "description": "Only show blocking stubs",
                        "default": false
                    }
                },
                "required": ["content"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let filter = Some(StubFilter {
                stub_type: args
                    .get("type_filter")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                blocking_only: args
                    .get("blocking_only")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                priority: None,
            });
            let stubs = switchboard
                .list_stubs(content, filter)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&stubs).map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_find_stub_anchors(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "find_stub_anchors",
            "Find inline anchor references (^anchor-id) in document content and match them to stubs",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    }
                },
                "required": ["content"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let matches = switchboard
                .find_stub_anchors(content)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&serde_json::json!({
                "anchors": matches.anchors,
                "stub_anchors": matches.stub_anchors,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    // =========================================================================
    // Stub Management Tools (NEW)
    // =========================================================================

    fn register_add_stub(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "add_stub",
            "Add a stub to document frontmatter. Returns updated document content.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    },
                    "stub_type": {
                        "type": "string",
                        "description": "Stub type (e.g., 'expand', 'link', 'verify', 'fix')"
                    },
                    "description": {
                        "type": "string",
                        "description": "Description of what needs to be done"
                    },
                    "priority": {
                        "type": "string",
                        "description": "Priority level (low, medium, high, critical)",
                        "enum": ["low", "medium", "high", "critical"]
                    },
                    "anchor": {
                        "type": "string",
                        "description": "Optional inline anchor to link (e.g., 'section-name')"
                    }
                },
                "required": ["content", "stub_type", "description"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let stub_type = args
                .get("stub_type")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'stub_type'")?;
            let description = args
                .get("description")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'description'")?;

            let new_stub = NewStub {
                stub_type: stub_type.to_string(),
                description: description.to_string(),
                priority: args
                    .get("priority")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                stub_form: None,
                anchor: args
                    .get("anchor")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            };

            let result = switchboard
                .add_stub(content, new_stub)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&serde_json::json!({
                "updated_content": result.updated_content,
                "stub_index": result.stub_index,
                "stub": result.stub,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_resolve_stub(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "resolve_stub",
            "Remove a resolved stub from document frontmatter. Returns updated document content.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    },
                    "stub_index": {
                        "type": "integer",
                        "description": "Index of the stub to resolve (0-based)"
                    }
                },
                "required": ["content", "stub_index"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let stub_index = args
                .get("stub_index")
                .and_then(|v| v.as_u64())
                .ok_or("Missing 'stub_index'")? as usize;

            let result = switchboard
                .resolve_stub(content, stub_index)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&serde_json::json!({
                "updated_content": result.updated_content,
                "resolved_stub": result.resolved_stub,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_update_stub(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "update_stub",
            "Update stub properties (description, priority, form). Returns updated document content.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    },
                    "stub_index": {
                        "type": "integer",
                        "description": "Index of the stub to update (0-based)"
                    },
                    "description": {
                        "type": "string",
                        "description": "New description (optional)"
                    },
                    "priority": {
                        "type": "string",
                        "description": "New priority (optional)",
                        "enum": ["low", "medium", "high", "critical"]
                    },
                    "stub_form": {
                        "type": "string",
                        "description": "New stub form (optional)",
                        "enum": ["transient", "persistent", "blocking", "structural"]
                    }
                },
                "required": ["content", "stub_index"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let stub_index = args
                .get("stub_index")
                .and_then(|v| v.as_u64())
                .ok_or("Missing 'stub_index'")? as usize;

            let updates = StubUpdates {
                description: args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                priority: args
                    .get("priority")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                stub_form: args
                    .get("stub_form")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            };

            let result = switchboard
                .update_stub(content, stub_index, updates)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&serde_json::json!({
                "updated_content": result.updated_content,
                "stub": result.stub,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_link_stub_anchor(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "link_stub_anchor",
            "Link a stub to an inline anchor reference. Returns updated document content.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    },
                    "stub_index": {
                        "type": "integer",
                        "description": "Index of the stub to link (0-based)"
                    },
                    "anchor_id": {
                        "type": "string",
                        "description": "Anchor ID to link (without the ^ prefix)"
                    }
                },
                "required": ["content", "stub_index", "anchor_id"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let stub_index = args
                .get("stub_index")
                .and_then(|v| v.as_u64())
                .ok_or("Missing 'stub_index'")? as usize;
            let anchor_id = args
                .get("anchor_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'anchor_id'")?;

            let result = switchboard
                .link_stub_anchor(content, stub_index, anchor_id)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&serde_json::json!({
                "updated_content": result.updated_content,
                "stub": result.stub,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_unlink_stub_anchor(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "unlink_stub_anchor",
            "Remove an anchor from a stub's inline_anchors array. Returns updated document content.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    },
                    "stub_index": {
                        "type": "integer",
                        "description": "Index of the stub to unlink from (0-based)"
                    },
                    "anchor_id": {
                        "type": "string",
                        "description": "Anchor ID to remove (without the ^ prefix)"
                    }
                },
                "required": ["content", "stub_index", "anchor_id"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let stub_index = args
                .get("stub_index")
                .and_then(|v| v.as_u64())
                .ok_or("Missing 'stub_index'")? as usize;
            let anchor_id = args
                .get("anchor_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'anchor_id'")?;

            let result = switchboard
                .unlink_stub_anchor(content, stub_index, anchor_id)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&serde_json::json!({
                "updated_content": result.updated_content,
                "stub": result.stub,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    // =========================================================================
    // Calculation Tools
    // =========================================================================

    fn register_calculate_health(&mut self) {
        let tool = McpTool::new(
            "calculate_health",
            "Calculate document health score from refinement and stubs. Formula: health = 0.7×refinement + 0.3×(1-stub_penalty)",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "refinement": {
                        "type": "number",
                        "description": "Refinement score (0.0-1.0)",
                        "minimum": 0,
                        "maximum": 1
                    },
                    "stubs": {
                        "type": "array",
                        "description": "Array of stub objects",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": { "type": "string" },
                                "description": { "type": "string" },
                                "stub_form": { "type": "string" }
                            }
                        }
                    }
                },
                "required": ["refinement"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| handlers::calc_health(args));

        self.register(tool, handler);
    }

    fn register_calculate_usefulness(&mut self) {
        let tool = McpTool::new(
            "calculate_usefulness",
            "Calculate usefulness margin for a target audience. Margin = refinement - gate",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "refinement": {
                        "type": "number",
                        "description": "Refinement score (0.0-1.0)",
                        "minimum": 0,
                        "maximum": 1
                    },
                    "audience": {
                        "type": "string",
                        "description": "Target audience: personal, internal, trusted, or public",
                        "enum": ["personal", "internal", "trusted", "public"]
                    }
                },
                "required": ["refinement", "audience"]
            }),
        );

        let handler: ToolHandler = Box::new(|args| handlers::calc_usefulness(args));

        self.register(tool, handler);
    }

    fn register_calculate_dimensions(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "calculate_dimensions",
            "Calculate all L2 state dimensions for a document (health, usefulness, trust, freshness)",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Markdown document content with YAML frontmatter"
                    }
                },
                "required": ["content"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let props = switchboard
                .parse_document(content)
                .map_err(|e| e.to_string())?;
            let dimensions = switchboard.calc_dimensions(&props);
            serde_json::to_string_pretty(&dimensions).map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_calculate_vector_physics(&mut self) {
        let tool = McpTool::new(
            "calculate_vector_physics",
            "Calculate vector physics (potential energy, friction, magnitude) for stub prioritization",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "stub": {
                        "type": "object",
                        "description": "Stub object with urgency, impact, complexity",
                        "properties": {
                            "type": { "type": "string" },
                            "description": { "type": "string" },
                            "urgency": { "type": "number" },
                            "impact": { "type": "number" },
                            "complexity": { "type": "number" }
                        }
                    },
                    "context": {
                        "type": "object",
                        "description": "Context for calculations",
                        "properties": {
                            "editorial_velocity": { "type": "number" },
                            "has_external_dependencies": { "type": "boolean" },
                            "has_controversy": { "type": "boolean" }
                        }
                    }
                },
                "required": ["stub"]
            }),
        );

        let handler: ToolHandler = Box::new(|args| handlers::calculate_vector_physics(args));

        self.register(tool, handler);
    }

    // =========================================================================
    // Information Tools
    // =========================================================================

    fn register_get_audience_gates(&mut self) {
        let tool = McpTool::new(
            "get_audience_gates",
            "Get the refinement gate thresholds for each audience level",
            serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        );

        let handler: ToolHandler = Box::new(|_| handlers::get_audience_gates());

        self.register(tool, handler);
    }

    fn register_get_schema(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "get_schema",
            "Get JSON Schema for frontmatter or stubs",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "schema_type": {
                        "type": "string",
                        "description": "Schema type to retrieve",
                        "enum": ["frontmatter", "stubs"]
                    }
                },
                "required": ["schema_type"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let schema_type = args
                .get("schema_type")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'schema_type'")?;
            let schema = match schema_type {
                "frontmatter" => switchboard.get_frontmatter_schema(),
                "stubs" => switchboard.get_stubs_schema(),
                _ => return Err(format!("Unknown schema type: {}", schema_type)),
            };
            Ok(schema.to_string())
        });

        self.register(tool, handler);
    }

    // =========================================================================
    // Batch Tools
    // =========================================================================

    fn register_batch_analyze(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "batch_analyze",
            "Analyze multiple documents and return aggregate statistics",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "documents": {
                        "type": "array",
                        "description": "Array of document objects with path and content",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" },
                                "content": { "type": "string" }
                            },
                            "required": ["path", "content"]
                        }
                    }
                },
                "required": ["documents"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let documents = args
                .get("documents")
                .and_then(|v| v.as_array())
                .ok_or("Missing 'documents'")?;

            let mut results = Vec::new();
            let mut total_health = 0.0;
            let mut success_count = 0;

            for doc in documents {
                let path = doc
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let content = doc.get("content").and_then(|v| v.as_str()).unwrap_or("");

                match switchboard.analyze_document(content) {
                    Ok(analysis) => {
                        total_health += analysis.dimensions.health;
                        success_count += 1;
                        results.push(serde_json::json!({
                            "path": path,
                            "success": true,
                            "health": analysis.dimensions.health,
                            "refinement": analysis.properties.refinement.value(),
                            "stub_count": analysis.properties.stubs.len(),
                        }));
                    }
                    Err(e) => {
                        results.push(serde_json::json!({
                            "path": path,
                            "success": false,
                            "error": e.to_string(),
                        }));
                    }
                }
            }

            let avg_health = if success_count > 0 {
                total_health / success_count as f64
            } else {
                0.0
            };

            serde_json::to_string_pretty(&serde_json::json!({
                "total": documents.len(),
                "success": success_count,
                "failed": documents.len() - success_count,
                "average_health": avg_health,
                "results": results,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    // =========================================================================
    // File System Tools
    // =========================================================================

    fn register_read_document(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "read_document",
            "Read a markdown file from disk and analyze it. Returns content and analysis.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the markdown file"
                    },
                    "analyze": {
                        "type": "boolean",
                        "description": "Whether to include full analysis (default: true)",
                        "default": true
                    }
                },
                "required": ["path"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path'")?;
            let analyze = args
                .get("analyze")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            // Read file
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read '{}': {}", path, e))?;

            if analyze {
                let analysis = switchboard
                    .analyze_document(&content)
                    .map_err(|e| e.to_string())?;

                serde_json::to_string_pretty(&serde_json::json!({
                    "path": path,
                    "content": content,
                    "properties": analysis.properties,
                    "dimensions": analysis.dimensions,
                    "warnings": analysis.warnings,
                }))
                .map_err(|e| e.to_string())
            } else {
                serde_json::to_string_pretty(&serde_json::json!({
                    "path": path,
                    "content": content,
                }))
                .map_err(|e| e.to_string())
            }
        });

        self.register(tool, handler);
    }

    fn register_scan_vault(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "scan_vault",
            "Scan a directory for markdown files and analyze their health. Returns summary statistics.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the vault directory"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern for files (default: **/*.md)",
                        "default": "**/*.md"
                    },
                    "include_content": {
                        "type": "boolean",
                        "description": "Include document content in results (default: false)",
                        "default": false
                    }
                },
                "required": ["path"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let vault_path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path'")?;
            let pattern = args
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("**/*.md");
            let include_content = args
                .get("include_content")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Use glob to find files
            let full_pattern = format!("{}/{}", vault_path, pattern);
            let paths: Vec<_> = glob::glob(&full_pattern)
                .map_err(|e| format!("Invalid glob pattern: {}", e))?
                .filter_map(Result::ok)
                .collect();

            let mut results = Vec::new();
            let mut total_health = 0.0;
            let mut total_stubs = 0;
            let mut blocking_count = 0;
            let mut success_count = 0;

            for path in &paths {
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        match switchboard.analyze_document(&content) {
                            Ok(analysis) => {
                                total_health += analysis.dimensions.health;
                                total_stubs += analysis.properties.stubs.len();
                                blocking_count += analysis
                                    .properties
                                    .stubs
                                    .iter()
                                    .filter(|s| s.is_blocking())
                                    .count();
                                success_count += 1;

                                let mut result = serde_json::json!({
                                    "path": path.display().to_string(),
                                    "success": true,
                                    "health": analysis.dimensions.health,
                                    "refinement": analysis.properties.refinement.value(),
                                    "stub_count": analysis.properties.stubs.len(),
                                    "blocking_stubs": analysis.properties.stubs.iter().filter(|s| s.is_blocking()).count(),
                                });

                                if include_content {
                                    result["content"] = serde_json::Value::String(content);
                                }

                                results.push(result);
                            }
                            Err(e) => {
                                results.push(serde_json::json!({
                                    "path": path.display().to_string(),
                                    "success": false,
                                    "error": e.to_string(),
                                }));
                            }
                        }
                    }
                    Err(e) => {
                        results.push(serde_json::json!({
                            "path": path.display().to_string(),
                            "success": false,
                            "error": format!("Failed to read file: {}", e),
                        }));
                    }
                }
            }

            let avg_health = if success_count > 0 {
                total_health / success_count as f64
            } else {
                0.0
            };

            serde_json::to_string_pretty(&serde_json::json!({
                "vault_path": vault_path,
                "total_files": paths.len(),
                "analyzed": success_count,
                "failed": paths.len() - success_count,
                "average_health": avg_health,
                "total_stubs": total_stubs,
                "blocking_stubs": blocking_count,
                "results": results,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_find_blocking_stubs(&mut self) {
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "find_blocking_stubs",
            "Find all blocking stubs across documents in a vault. Returns stubs that prevent publication.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the vault directory"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern for files (default: **/*.md)",
                        "default": "**/*.md"
                    }
                },
                "required": ["path"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let vault_path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path'")?;
            let pattern = args
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("**/*.md");

            // Use glob to find files
            let full_pattern = format!("{}/{}", vault_path, pattern);
            let paths: Vec<_> = glob::glob(&full_pattern)
                .map_err(|e| format!("Invalid glob pattern: {}", e))?
                .filter_map(Result::ok)
                .collect();

            let mut blocking_stubs = Vec::new();
            let mut documents_with_blocking = 0;

            for path in &paths {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(props) = switchboard.parse_document(&content) {
                        let doc_blocking: Vec<_> = props
                            .stubs
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| s.is_blocking())
                            .map(|(idx, s)| {
                                serde_json::json!({
                                    "document": path.display().to_string(),
                                    "stub_index": idx,
                                    "type": s.stub_type.as_str(),
                                    "description": s.description,
                                    "priority": s.priority.to_string(),
                                })
                            })
                            .collect();

                        if !doc_blocking.is_empty() {
                            documents_with_blocking += 1;
                            blocking_stubs.extend(doc_blocking);
                        }
                    }
                }
            }

            serde_json::to_string_pretty(&serde_json::json!({
                "vault_path": vault_path,
                "total_documents": paths.len(),
                "documents_with_blocking": documents_with_blocking,
                "total_blocking_stubs": blocking_stubs.len(),
                "blocking_stubs": blocking_stubs,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    // =========================================================================
    // Git Integration Tools
    // =========================================================================

    fn register_snapshot_before_edit(&mut self) {
        let git = Arc::clone(&self.git);

        let tool = McpTool::new(
            "snapshot_before_edit",
            "Create a git commit snapshot before making changes to a document. Requires git repository.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the document file"
                    },
                    "message": {
                        "type": "string",
                        "description": "Optional commit message (default: 'Auto-snapshot before Doc-Doctor edit')"
                    }
                },
                "required": ["path"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path'")?;
            let message = args.get("message").and_then(|v| v.as_str());

            let status = git.check_availability();
            if !status.available {
                return serde_json::to_string_pretty(&serde_json::json!({
                    "success": false,
                    "integration_available": false,
                    "reminder": status.reminder,
                }))
                .map_err(|e| e.to_string());
            }

            let path_buf = PathBuf::from(path);
            match git.snapshot_before_edit(&path_buf, message) {
                Ok(result) => serde_json::to_string_pretty(&serde_json::json!({
                    "success": true,
                    "integration_available": true,
                    "commit_hash": result.commit_hash,
                    "message": result.message,
                    "files_changed": result.files_changed,
                })),
                Err(e) => serde_json::to_string_pretty(&serde_json::json!({
                    "success": false,
                    "integration_available": true,
                    "error": e.to_string(),
                })),
            }
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_commit_stub_resolution(&mut self) {
        let git = Arc::clone(&self.git);

        let tool = McpTool::new(
            "commit_stub_resolution",
            "Commit changes with a descriptive message about the resolved stub.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the document file"
                    },
                    "stub_type": {
                        "type": "string",
                        "description": "Type of the resolved stub"
                    },
                    "stub_description": {
                        "type": "string",
                        "description": "Description of the resolved stub"
                    },
                    "additional_message": {
                        "type": "string",
                        "description": "Optional additional commit message"
                    }
                },
                "required": ["path", "stub_type", "stub_description"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path'")?;
            let stub_type = args
                .get("stub_type")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'stub_type'")?;
            let stub_description = args
                .get("stub_description")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'stub_description'")?;
            let additional = args.get("additional_message").and_then(|v| v.as_str());

            let status = git.check_availability();
            if !status.available {
                return serde_json::to_string_pretty(&serde_json::json!({
                    "success": false,
                    "integration_available": false,
                    "reminder": status.reminder,
                }))
                .map_err(|e| e.to_string());
            }

            let path_buf = PathBuf::from(path);
            match git.commit_stub_resolution(&path_buf, stub_type, stub_description, additional) {
                Ok(result) => serde_json::to_string_pretty(&serde_json::json!({
                    "success": result.success,
                    "integration_available": true,
                    "commit_hash": result.commit_hash,
                    "message": result.message,
                    "note": result.note,
                })),
                Err(e) => serde_json::to_string_pretty(&serde_json::json!({
                    "success": false,
                    "integration_available": true,
                    "error": e.to_string(),
                })),
            }
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_get_document_history(&mut self) {
        let git = Arc::clone(&self.git);

        let tool = McpTool::new(
            "get_document_history",
            "Get git commit history for a specific document.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the document file"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of commits to return (default: 10)",
                        "default": 10
                    }
                },
                "required": ["path"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path'")?;
            let limit = args
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(10) as usize;

            let status = git.check_availability();
            if !status.available {
                return serde_json::to_string_pretty(&serde_json::json!({
                    "commits": [],
                    "integration_available": false,
                    "reminder": status.reminder,
                }))
                .map_err(|e| e.to_string());
            }

            let path_buf = PathBuf::from(path);
            match git.get_document_history(&path_buf, limit) {
                Ok(commits) => {
                    let commit_json: Vec<_> = commits
                        .iter()
                        .map(|c| {
                            serde_json::json!({
                                "hash": c.hash,
                                "author": c.author_name,
                                "email": c.author_email,
                                "date": c.date,
                                "message": c.message,
                            })
                        })
                        .collect();

                    serde_json::to_string_pretty(&serde_json::json!({
                        "path": path,
                        "commits": commit_json,
                        "count": commits.len(),
                        "integration_available": true,
                    }))
                }
                Err(e) => serde_json::to_string_pretty(&serde_json::json!({
                    "commits": [],
                    "integration_available": true,
                    "error": e.to_string(),
                })),
            }
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_diff_document_versions(&mut self) {
        let git = Arc::clone(&self.git);

        let tool = McpTool::new(
            "diff_document_versions",
            "Show diff between document versions across commits.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the document file"
                    },
                    "from_commit": {
                        "type": "string",
                        "description": "Starting commit hash or reference (e.g., 'HEAD~1')"
                    },
                    "to_commit": {
                        "type": "string",
                        "description": "Ending commit hash or reference (default: 'HEAD')",
                        "default": "HEAD"
                    }
                },
                "required": ["path", "from_commit"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path'")?;
            let from_commit = args
                .get("from_commit")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'from_commit'")?;
            let to_commit = args
                .get("to_commit")
                .and_then(|v| v.as_str())
                .unwrap_or("HEAD");

            let status = git.check_availability();
            if !status.available {
                return serde_json::to_string_pretty(&serde_json::json!({
                    "diff": "",
                    "integration_available": false,
                    "reminder": status.reminder,
                }))
                .map_err(|e| e.to_string());
            }

            let path_buf = PathBuf::from(path);
            match git.diff_document_versions(&path_buf, from_commit, to_commit) {
                Ok(result) => serde_json::to_string_pretty(&serde_json::json!({
                    "path": path,
                    "from_commit": result.from_commit,
                    "to_commit": result.to_commit,
                    "diff": result.diff,
                    "stat": result.stat,
                    "additions": result.additions,
                    "deletions": result.deletions,
                    "integration_available": true,
                })),
                Err(e) => serde_json::to_string_pretty(&serde_json::json!({
                    "diff": "",
                    "integration_available": true,
                    "error": e.to_string(),
                })),
            }
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    // =========================================================================
    // RAG/Smart Connections Tools
    // =========================================================================

    fn register_find_related_documents(&mut self) {
        let sc = Arc::clone(&self.smart_connections);

        let tool = McpTool::new(
            "find_related_documents",
            "Find semantically related documents using Smart Connections embeddings or keyword matching.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query or content to find related documents for"
                    },
                    "vault_path": {
                        "type": "string",
                        "description": "Path to the vault directory"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 5)",
                        "default": 5
                    },
                    "min_similarity": {
                        "type": "number",
                        "description": "Minimum similarity threshold 0-1 (default: 0.3)",
                        "default": 0.3
                    }
                },
                "required": ["query", "vault_path"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'query'")?;
            let vault_path = args
                .get("vault_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'vault_path'")?;
            let limit = args
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(5) as usize;
            let min_similarity = args
                .get("min_similarity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.3) as f32;

            let vault = PathBuf::from(vault_path);

            // Try to get or create SC integration for this vault
            let mut sc_guard = sc.write().map_err(|e| e.to_string())?;
            *sc_guard = SmartConnectionsIntegration::for_vault(&vault);

            let status = sc_guard.check_availability();
            let has_embeddings = status.available;

            // Try to load embeddings if available
            if has_embeddings {
                let _ = sc_guard.load_embeddings();
            }

            // Find related documents (uses keyword fallback if no embeddings)
            let results = sc_guard.find_related_by_content(query, &vault, limit, min_similarity);

            let result_json: Vec<_> = results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "path": r.path,
                        "similarity": r.similarity,
                        "excerpt": r.excerpt,
                    })
                })
                .collect();

            serde_json::to_string_pretty(&serde_json::json!({
                "query": query,
                "vault_path": vault_path,
                "results": result_json,
                "count": results.len(),
                "integration_available": has_embeddings,
                "method": if has_embeddings { "embeddings" } else { "keyword" },
                "reminder": if !has_embeddings { status.reminder } else { None },
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_suggest_links(&mut self) {
        let sc = Arc::clone(&self.smart_connections);

        let tool = McpTool::new(
            "suggest_links",
            "Suggest documents to link based on content similarity.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Document content to find links for"
                    },
                    "vault_path": {
                        "type": "string",
                        "description": "Path to the vault directory"
                    },
                    "existing_links": {
                        "type": "array",
                        "description": "Already linked documents to exclude",
                        "items": { "type": "string" },
                        "default": []
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum suggestions (default: 5)",
                        "default": 5
                    }
                },
                "required": ["content", "vault_path"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let vault_path = args
                .get("vault_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'vault_path'")?;
            let existing: Vec<String> = args
                .get("existing_links")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let limit = args
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(5) as usize;

            let vault = PathBuf::from(vault_path);

            let mut sc_guard = sc.write().map_err(|e| e.to_string())?;
            *sc_guard = SmartConnectionsIntegration::for_vault(&vault);

            let status = sc_guard.check_availability();

            let suggestions = sc_guard.suggest_links(content, &existing, &vault, limit);

            let suggestions_json: Vec<_> = suggestions
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "path": s.path,
                        "relevance": s.relevance,
                        "reason": s.reason,
                    })
                })
                .collect();

            serde_json::to_string_pretty(&serde_json::json!({
                "suggestions": suggestions_json,
                "count": suggestions.len(),
                "integration_available": status.available,
                "reminder": if !status.available { status.reminder } else { None },
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_detect_duplicates(&mut self) {
        let sc = Arc::clone(&self.smart_connections);

        let tool = McpTool::new(
            "detect_duplicates",
            "Find potentially duplicate content across the vault.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Content to check for duplicates"
                    },
                    "vault_path": {
                        "type": "string",
                        "description": "Path to the vault directory"
                    },
                    "threshold": {
                        "type": "number",
                        "description": "Similarity threshold 0-1 (default: 0.7)",
                        "default": 0.7
                    }
                },
                "required": ["content", "vault_path"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content'")?;
            let vault_path = args
                .get("vault_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'vault_path'")?;
            let threshold = args
                .get("threshold")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.7) as f32;

            let vault = PathBuf::from(vault_path);

            let sc_guard = sc.read().map_err(|e| e.to_string())?;

            let duplicates = sc_guard.detect_duplicates(content, &vault, threshold);

            let duplicates_json: Vec<_> = duplicates
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "path": d.path,
                        "section": d.section,
                        "similarity": d.similarity,
                        "recommendation": d.recommendation.to_string(),
                    })
                })
                .collect();

            serde_json::to_string_pretty(&serde_json::json!({
                "duplicates": duplicates_json,
                "count": duplicates.len(),
                "threshold": threshold,
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }

    fn register_draft_with_context(&mut self) {
        let sc = Arc::clone(&self.smart_connections);
        let switchboard = Arc::clone(&self.switchboard);

        let tool = McpTool::new(
            "draft_with_context",
            "Gather RAG context from related documents for content generation. Returns context, not generated content.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "stub_description": {
                        "type": "string",
                        "description": "Description of the stub/content to draft"
                    },
                    "document_content": {
                        "type": "string",
                        "description": "Current document content for context"
                    },
                    "vault_path": {
                        "type": "string",
                        "description": "Path to the vault directory"
                    },
                    "max_context_documents": {
                        "type": "integer",
                        "description": "Maximum context documents (default: 3)",
                        "default": 3
                    }
                },
                "required": ["stub_description", "vault_path"]
            }),
        );

        let handler: ToolHandler = Box::new(move |args| {
            let stub_description = args
                .get("stub_description")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'stub_description'")?;
            let document_content = args
                .get("document_content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let vault_path = args
                .get("vault_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'vault_path'")?;
            let max_docs = args
                .get("max_context_documents")
                .and_then(|v| v.as_u64())
                .unwrap_or(3) as usize;

            let vault = PathBuf::from(vault_path);

            // Get document properties if available
            let doc_props = if !document_content.is_empty() {
                switchboard.parse_document(document_content).ok()
            } else {
                None
            };

            let mut sc_guard = sc.write().map_err(|e| e.to_string())?;
            *sc_guard = SmartConnectionsIntegration::for_vault(&vault);

            let status = sc_guard.check_availability();

            // Find related documents using stub description as query
            let query = format!("{} {}", stub_description, document_content.chars().take(500).collect::<String>());
            let related = sc_guard.find_related_by_content(&query, &vault, max_docs, 0.2);

            // Build context from related documents
            let mut context_docs = Vec::new();
            for r in &related {
                if let Ok(content) = std::fs::read_to_string(&r.path) {
                    // Extract relevant excerpt
                    let excerpt = r.excerpt.clone().unwrap_or_else(|| {
                        content.chars().take(500).collect::<String>()
                    });

                    context_docs.push(serde_json::json!({
                        "path": r.path,
                        "relevance": r.similarity,
                        "excerpt": excerpt,
                    }));
                }
            }

            serde_json::to_string_pretty(&serde_json::json!({
                "stub_description": stub_description,
                "document_context": doc_props.map(|p| serde_json::json!({
                    "title": p.title,
                    "audience": p.audience.to_string(),
                    "form": p.form.to_string(),
                    "refinement": p.refinement.value(),
                })),
                "related_documents": context_docs,
                "context_count": context_docs.len(),
                "integration_available": status.available,
                "method": if status.available { "embeddings" } else { "keyword" },
                "reminder": if !status.available {
                    Some("Install Smart Connections plugin for better semantic matching")
                } else {
                    None
                },
            }))
            .map_err(|e| e.to_string())
        });

        self.register(tool, handler);
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
