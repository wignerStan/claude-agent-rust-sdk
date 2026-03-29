//! Integration tests simulating the stdio JSON-RPC flow.
//!
//! These tests exercise the full request lifecycle via `handle_request()`,
//! covering the same paths that `run_stdio()` would take when processing
//! lines from stdin.

use claude_agent_mcp_sdk::error::McpSdkError;
use claude_agent_mcp_sdk::server::SdkMcpServer;
use claude_agent_mcp_sdk::tool::ToolDefinition;
use claude_agent_mcp_sdk::tool::ToolHandler;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

/// A simple echo handler that returns the "message" field from input.
struct EchoTool;

impl ToolHandler for EchoTool {
    fn call(
        &self,
        input: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>> {
        Box::pin(async move {
            let message = input.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": message }]
            }))
        })
    }
}

/// A handler that always returns an error.
struct FailTool;

impl ToolHandler for FailTool {
    fn call(
        &self,
        _input: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>> {
        Box::pin(async { Err(McpSdkError::HandlerError("tool crashed".to_string())) })
    }
}

fn create_test_server() -> SdkMcpServer {
    let mut server = SdkMcpServer::new("test-server", "1.0.0");
    server.add_tool(ToolDefinition::new(
        "echo",
        "Echoes a message",
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": { "type": "string", "description": "Message to echo" }
            },
            "required": ["message"]
        }),
        EchoTool,
    ));
    server.add_tool(ToolDefinition::new(
        "fail",
        "Always fails",
        serde_json::json!({ "type": "object" }),
        FailTool,
    ));
    server
}

/// Simulates the stdio initialize flow: client sends initialize, server responds.
#[tokio::test]
async fn test_run_stdio_initialize_flow() {
    let server = create_test_server();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "0.1.0" }
        }
    });

    let response = server.handle_request(request).await.unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(response["result"]["serverInfo"]["name"], "test-server");
    assert_eq!(response["result"]["serverInfo"]["version"], "1.0.0");
    assert!(response["result"]["capabilities"]["tools"].is_object());
}

/// Simulates the stdio tools/list flow: initialize first, then list tools.
#[tokio::test]
async fn test_run_stdio_tools_list_flow() {
    let server = create_test_server();

    // Step 1: Initialize
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0.0" }
        }
    });
    let init_response = server.handle_request(init_request).await.unwrap();
    assert_eq!(init_response["result"]["serverInfo"]["name"], "test-server");

    // Step 2: Send initialized notification
    let notif_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let notif_response = server.handle_request(notif_request).await.unwrap();
    assert!(notif_response.is_null());

    // Step 3: List tools
    let list_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    let list_response = server.handle_request(list_request).await.unwrap();
    let tools = list_response["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);

    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"echo"));
    assert!(tool_names.contains(&"fail"));

    // Verify schema is included
    let echo_tool = tools.iter().find(|t| t["name"] == "echo").unwrap();
    assert_eq!(echo_tool["inputSchema"]["type"], "object");
    assert!(echo_tool["inputSchema"]["properties"]["message"].is_object());
}

/// Simulates the stdio tools/call flow: initialize, then call the echo tool.
#[tokio::test]
async fn test_run_stdio_tools_call_flow() {
    let server = create_test_server();

    // Initialize
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0.0" }
        }
    });
    server.handle_request(init_request).await.unwrap();

    // Call echo tool
    let call_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "echo",
            "arguments": { "message": "hello from integration test" }
        }
    });

    let response = server.handle_request(call_request).await.unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert_eq!(response["result"]["content"][0]["text"], "hello from integration test");
}

/// Test that invalid JSON method names (missing method entirely) produce an error.
#[tokio::test]
async fn test_run_stdio_invalid_json() {
    let server = create_test_server();

    // Completely missing "method" field -- simulates a malformed request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 99
    });

    let response = server.handle_request(request).await.unwrap();
    assert_eq!(response["error"]["code"], -32601);
    assert!(response["error"]["message"].as_str().unwrap().contains("Method not found"));
}

/// Test that an unknown method returns the correct JSON-RPC error.
#[tokio::test]
async fn test_run_stdio_unknown_method() {
    let server = create_test_server();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "foo/bar"
    });

    let response = server.handle_request(request).await.unwrap();
    assert_eq!(response["error"]["code"], -32601);
    assert!(response["error"]["message"].as_str().unwrap().contains("Method not found: foo/bar"));
}

/// Test that handler errors in tools/call are returned as JSON-RPC errors.
#[tokio::test]
async fn test_run_stdio_tools_call_handler_error() {
    let server = create_test_server();

    // Initialize
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0.0" }
        }
    });
    server.handle_request(init_request).await.unwrap();

    // Call the failing tool
    let call_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "fail",
            "arguments": {}
        }
    });

    let response = server.handle_request(call_request).await.unwrap();
    assert_eq!(response["error"]["code"], -32000);
    assert!(response["error"]["message"].as_str().unwrap().contains("tool crashed"));
    assert_eq!(response["id"], 5);
}
