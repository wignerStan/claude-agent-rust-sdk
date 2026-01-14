//! Integration tests for HTTP and SSE MCP transports using wiremock.

use claude_agent_mcp::manager::McpServer;
use claude_agent_mcp::transports::{HttpMcpServer, SseMcpServer};
use serde_json::json;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test HttpMcpServer with a mock MCP server
#[tokio::test]
async fn test_http_mcp_server_list_tools() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "tools": [
                    {
                        "name": "test_tool",
                        "description": "A test tool",
                        "input_schema": { "type": "object" }
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let server =
        HttpMcpServer::new("test".to_string(), mock_server.uri()).expect("Failed to create server");
    let tools = server.list_tools().await.unwrap();

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "test_tool");
}

/// Test HttpMcpServer call_tool
#[tokio::test]
async fn test_http_mcp_server_call_tool() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "content": [{ "type": "text", "text": "Hello, World!" }],
                "isError": false
            }
        })))
        .mount(&mock_server)
        .await;

    let server =
        HttpMcpServer::new("test".to_string(), mock_server.uri()).expect("Failed to create server");
    let result = server.call_tool("greet", json!({"name": "World"})).await.unwrap();

    assert_eq!(result.get("isError").and_then(|v| v.as_bool()), Some(false));
}

/// Test HttpMcpServer error handling for HTTP failures
#[tokio::test]
async fn test_http_mcp_server_http_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST")).respond_with(ResponseTemplate::new(500)).mount(&mock_server).await;

    let server =
        HttpMcpServer::new("test".to_string(), mock_server.uri()).expect("Failed to create server");
    let result = server.list_tools().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("HTTP error"));
}

/// Test HttpMcpServer error handling for JSON-RPC errors
#[tokio::test]
async fn test_http_mcp_server_jsonrpc_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32600,
                "message": "Invalid Request"
            }
        })))
        .mount(&mock_server)
        .await;

    let server =
        HttpMcpServer::new("test".to_string(), mock_server.uri()).expect("Failed to create server");
    let result = server.list_tools().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid Request"));
}

/// Test SseMcpServer with a mock MCP server
#[tokio::test]
async fn test_sse_mcp_server_list_tools() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "tools": [
                    {
                        "name": "sse_tool",
                        "description": "An SSE test tool",
                        "input_schema": { "type": "object" }
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let server =
        SseMcpServer::new("test".to_string(), mock_server.uri()).expect("Failed to create server");
    let tools = server.list_tools().await.unwrap();

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "sse_tool");
}

/// Test SseMcpServer call_tool
#[tokio::test]
async fn test_sse_mcp_server_call_tool() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "content": [{ "type": "text", "text": "SSE Result" }],
                "isError": false
            }
        })))
        .mount(&mock_server)
        .await;

    let server =
        SseMcpServer::new("test".to_string(), mock_server.uri()).expect("Failed to create server");
    let result = server.call_tool("sse_action", json!({})).await.unwrap();

    assert_eq!(result.get("isError").and_then(|v| v.as_bool()), Some(false));
}

/// Test HttpMcpServer handle_client_message
#[tokio::test]
async fn test_http_mcp_server_handle_client_message_initialize() {
    let mock_server = MockServer::start().await;
    let server = HttpMcpServer::new("test_server".to_string(), mock_server.uri())
        .expect("Failed to create server");

    let message = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize"
    });

    let result = server.handle_client_message(message).await.unwrap();

    assert!(result.get("result").is_some());
    let server_info = result
        .get("result")
        .and_then(|r| r.get("serverInfo"))
        .and_then(|s| s.get("name"))
        .and_then(|n| n.as_str());
    assert_eq!(server_info, Some("test_server"));
}

/// Test HttpMcpServer connection timeout
#[tokio::test]
async fn test_http_mcp_server_connection_refused() {
    // Use a port that's unlikely to be in use
    let server = HttpMcpServer::new("test".to_string(), "http://127.0.0.1:59999".to_string())
        .expect("Failed to create server");
    let result = server.list_tools().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("HTTP request failed"));
}
