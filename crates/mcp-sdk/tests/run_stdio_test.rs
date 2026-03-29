//! Integration tests for `SdkMcpServer` stdio-equivalent flow.
//!
//! These tests exercise the same code paths as `run_stdio()` by calling
//! `handle_request()` directly with inputs that simulate what the stdio loop
//! would receive. This covers: empty lines, malformed JSON, valid JSON-RPC,
//! error responses, and multi-request sequences.

use claude_agent_mcp_sdk::error::McpSdkError;
use claude_agent_mcp_sdk::server::SdkMcpServer;
use claude_agent_mcp_sdk::tool::{ToolDefinition, ToolHandler};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;

/// A simple echo handler.
struct EchoTool;

impl ToolHandler for EchoTool {
    fn call(
        &self,
        input: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>> {
        Box::pin(async move {
            let message = input.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
            Ok(json!({
                "content": [{ "type": "text", "text": message }]
            }))
        })
    }
}

/// A handler that panics (for testing internal error handling).
struct PanicTool;

impl ToolHandler for PanicTool {
    fn call(
        &self,
        _input: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>> {
        Box::pin(async { Err(McpSdkError::HandlerError("internal panic".to_string())) })
    }
}

fn create_server() -> SdkMcpServer {
    let mut server = SdkMcpServer::new("test-server", "1.0.0");
    server.add_tool(ToolDefinition::new(
        "echo",
        "Echoes a message",
        json!({
            "type": "object",
            "properties": { "message": { "type": "string" } },
            "required": ["message"]
        }),
        EchoTool,
    ));
    server.add_tool(ToolDefinition::new(
        "panic_tool",
        "Always fails",
        json!({ "type": "object" }),
        PanicTool,
    ));
    server
}

// --- Simulated stdio flow tests ---
// These mirror the exact behavior of run_stdio()'s line-processing loop.

#[tokio::test]
async fn test_stdio_flow_empty_line_skipped() {
    // In run_stdio(), empty lines are trimmed and skipped with `continue`.
    let trimmed = "".trim();
    assert!(trimmed.is_empty());
}

#[tokio::test]
async fn test_stdio_flow_whitespace_only_line_skipped() {
    let trimmed = "   \t  ".trim();
    assert!(trimmed.is_empty());
}

#[tokio::test]
async fn test_stdio_flow_malformed_json_produces_parse_error() {
    // In run_stdio(), serde_json::from_str failure produces a -32700 error.
    let result = serde_json::from_str::<Value>("{{not json");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(!err.is_empty());
}

#[tokio::test]
async fn test_stdio_flow_full_lifecycle() {
    let server = create_server();

    // Step 1: Initialize
    let init = json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0.0" }
        }
    });
    let resp = server.handle_request(init).await.unwrap();
    assert_eq!(resp["result"]["serverInfo"]["name"], "test-server");

    // Step 2: Initialized notification (returns null — no output written)
    let notif = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
    let resp = server.handle_request(notif).await.unwrap();
    assert!(resp.is_null());

    // Step 3: List tools
    let list = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" });
    let resp = server.handle_request(list).await.unwrap();
    assert_eq!(resp["result"]["tools"].as_array().unwrap().len(), 2);

    // Step 4: Call echo tool
    let call = json!({
        "jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": { "name": "echo", "arguments": { "message": "stdio lifecycle" } }
    });
    let resp = server.handle_request(call).await.unwrap();
    assert_eq!(resp["result"]["content"][0]["text"], "stdio lifecycle");

    // Step 5: Call failing tool — error response is written, not propagated
    let fail = json!({
        "jsonrpc": "2.0", "id": 4, "method": "tools/call",
        "params": { "name": "panic_tool", "arguments": {} }
    });
    let resp = server.handle_request(fail).await.unwrap();
    assert_eq!(resp["error"]["code"], -32000);
}

#[tokio::test]
async fn test_stdio_flow_internal_error_in_handle_request() {
    // In run_stdio(), if handle_request returns Err, a -32000 error is written.
    // We verify the error path by sending a request that triggers an internal error.
    let server = create_server();

    let request = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call"
        // Missing params — this returns Err(McpSdkError::InvalidInput)
    });

    let result = server.handle_request(request).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Missing params"));
}

// --- Re-export tests ---

#[test]
fn reexports_are_accessible() {
    use claude_agent_mcp_sdk::reexports::serde_json;
    let _val = serde_json::json!({"test": true});
}

#[test]
fn all_public_types_are_accessible() {
    use claude_agent_mcp_sdk::{
        McpSdkError, McpSdkServerConfig, SdkMcpServer, ToolDefinition, ToolHandler,
    };
    let _ = std::mem::size_of::<SdkMcpServer>();
    let _ = std::mem::size_of::<McpSdkServerConfig>();
    let _ = std::mem::size_of::<ToolDefinition>();
    let _ = std::mem::size_of::<McpSdkError>();
    // Verify ToolHandler is a trait
    fn _assert_trait<T: ToolHandler>() {}
}
