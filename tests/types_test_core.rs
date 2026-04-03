//! Additional integration tests for core response types.

use claude_agent::core::server_info::*;

#[test]
fn mcp_connection_status_equality() {
    assert_eq!(McpConnectionStatus::Connected, McpConnectionStatus::Connected);
    assert_ne!(McpConnectionStatus::Connected, McpConnectionStatus::Failed);
}

#[test]
fn mcp_connection_status_all_variants() {
    let _ = McpConnectionStatus::Connected;
    let _ = McpConnectionStatus::Pending;
    let _ = McpConnectionStatus::Failed;
    let _ = McpConnectionStatus::NeedsAuth;
    let _ = McpConnectionStatus::Disabled;
}

#[test]
fn mcp_tool_info_serde() {
    let json = r#"{"name":"search","description":"Search tool"}"#;
    let tool: McpToolInfo = serde_json::from_str(json).unwrap();
    assert_eq!(tool.name, "search");
    assert_eq!(tool.description.as_deref(), Some("Search tool"));
}

#[test]
fn mcp_tool_info_minimal() {
    let json = r#"{"name":"bare"}"#;
    let tool: McpToolInfo = serde_json::from_str(json).unwrap();
    assert_eq!(tool.name, "bare");
    assert!(tool.description.is_none());
}

#[test]
fn mcp_status_response_empty_default() {
    let response = McpStatusResponse::empty();
    assert!(response.mcp_servers.is_empty());
    assert_eq!(McpStatusResponse::default().mcp_servers.len(), 0);
}

#[test]
fn mcp_server_status_all_fields() {
    let json = r#"{
        "name":"full-server",
        "status":"needs_auth",
        "server_info":{"version":"1.0"},
        "error":"Auth required",
        "config":{"url":"http://localhost"},
        "scope":"user",
        "tools":[{"name":"t1"}]
    }"#;
    let status: McpServerStatus = serde_json::from_str(json).unwrap();
    assert_eq!(status.name, "full-server");
    assert_eq!(status.status, McpConnectionStatus::NeedsAuth);
    assert!(status.server_info.is_some());
    assert_eq!(status.error.as_deref(), Some("Auth required"));
    assert!(status.config.is_some());
    assert_eq!(status.scope.as_deref(), Some("user"));
    assert_eq!(status.tools.len(), 1);
}

#[test]
fn mcp_server_status_minimal() {
    let json = r#"{"name":"minimal","status":"connected"}"#;
    let status: McpServerStatus = serde_json::from_str(json).unwrap();
    assert_eq!(status.name, "minimal");
    assert_eq!(status.status, McpConnectionStatus::Connected);
    assert!(status.server_info.is_none());
    assert!(status.error.is_none());
    assert!(status.config.is_none());
    assert!(status.scope.is_none());
    assert!(status.tools.is_empty());
}
