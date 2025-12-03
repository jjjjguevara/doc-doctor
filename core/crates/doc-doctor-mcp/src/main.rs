//! Doc Doctor MCP Server
//!
//! Model Context Protocol server exposing J-Editorial operations.
//!
//! # Usage
//!
//! The server communicates via JSON-RPC 2.0 over stdio.
//!
//! ## Claude Desktop Configuration
//!
//! Add to `~/.config/claude/claude_desktop_config.json`:
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "doc-doctor": {
//!       "command": "dd-mcp",
//!       "args": []
//!     }
//!   }
//! }
//! ```

mod integrations;
mod protocol;
mod tools;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

use protocol::{JsonRpcRequest, JsonRpcResponse};
use tools::ToolRegistry;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is for MCP protocol)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("doc_doctor_mcp=debug".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Doc Doctor MCP Server starting...");

    // Create tool registry
    let registry = ToolRegistry::new();

    // Run the server
    run_server(registry).await
}

async fn run_server(registry: ToolRegistry) -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            // EOF - client disconnected
            info!("Client disconnected");
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        debug!("Received: {}", trimmed);

        // Parse the request
        let response = match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(request) => handle_request(&registry, request).await,
            Err(e) => {
                error!("Parse error: {}", e);
                JsonRpcResponse::error(
                    serde_json::Value::Null,
                    -32700,
                    format!("Parse error: {}", e),
                )
            }
        };

        // Send the response
        let response_json = serde_json::to_string(&response)?;
        debug!("Sending: {}", response_json);

        stdout.write_all(response_json.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }

    Ok(())
}

async fn handle_request(registry: &ToolRegistry, request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        // MCP initialization
        "initialize" => {
            info!("Client initializing");
            JsonRpcResponse::success(
                id,
                serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "doc-doctor",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            )
        }

        // List available tools
        "tools/list" => {
            let tools = registry.list_tools();
            JsonRpcResponse::success(
                id,
                serde_json::json!({
                    "tools": tools
                }),
            )
        }

        // Call a tool
        "tools/call" => {
            let params = request.params.unwrap_or(serde_json::Value::Null);

            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            match registry.call_tool(tool_name, arguments) {
                Ok(result) => JsonRpcResponse::success(
                    id,
                    serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": result
                        }]
                    }),
                ),
                Err(e) => JsonRpcResponse::success(
                    id,
                    serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Error: {}", e)
                        }],
                        "isError": true
                    }),
                ),
            }
        }

        // Notifications (no response needed, but we handle gracefully)
        "notifications/initialized" => {
            info!("Client initialized notification received");
            // No response for notifications, but return empty success
            JsonRpcResponse::success(id, serde_json::Value::Null)
        }

        // Unknown method
        method => {
            error!("Unknown method: {}", method);
            JsonRpcResponse::error(id, -32601, format!("Method not found: {}", method))
        }
    }
}
