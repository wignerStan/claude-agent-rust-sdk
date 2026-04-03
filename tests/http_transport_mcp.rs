//! Integration tests for HTTP MCP transport.
//!
//! Note: With the rmcp-based transport, these tests verify construction and
//! configuration rather than full HTTP roundtrips (which require a live MCP server).
//! Full integration tests would use a subprocess MCP server via stdio transport.

use claude_agent::mcp::manager::McpServer;
use claude_agent::mcp::transports::{HttpMcpServer, SseMcpServer};

#[test]
fn test_http_mcp_server_creation() {
    let server =
        HttpMcpServer::new("test".to_string(), "http://localhost:8080/mcp".to_string()).unwrap();
    assert_eq!(server.name(), "test");
}

#[test]
fn test_http_mcp_server_with_timeout() {
    let server = HttpMcpServer::with_timeout(
        "test".to_string(),
        "http://localhost:8080/mcp".to_string(),
        std::time::Duration::from_secs(30),
    )
    .unwrap();
    assert_eq!(server.name(), "test");
}

#[test]
fn test_sse_mcp_server_creation() {
    let server =
        SseMcpServer::new("test".to_string(), "http://localhost:8080/sse".to_string()).unwrap();
    assert_eq!(server.name(), "test");
}

#[test]
fn test_sse_mcp_server_with_timeout() {
    let server = SseMcpServer::with_timeout(
        "test".to_string(),
        "http://localhost:8080/sse".to_string(),
        std::time::Duration::from_secs(30),
    )
    .unwrap();
    assert_eq!(server.name(), "test");
}

#[tokio::test]
async fn test_http_mcp_server_connection_refused() {
    // rmcp will fail during the MCP handshake to a non-existent server
    let server =
        HttpMcpServer::new("test".to_string(), "http://127.0.0.1:59999".to_string()).unwrap();
    let result = server.list_tools().await;
    assert!(result.is_err());
}
