use claude_agent_mcp::{McpServerManager, SdkMcpServer};
use serde_json::json;

#[tokio::test]
async fn test_manager_registration_and_listing() {
    let manager = McpServerManager::new();

    // Register SDK server
    let mut sdk_server = SdkMcpServer::new("sdk-server");
    sdk_server.register_tool("test_tool", None, json!({}), |_| {
        Box::pin(async { Ok(json!({"result": "ok"})) })
    });

    manager.register(Box::new(sdk_server)).await;

    // Register "Stdio" server (dummy for this test since we don't spawn real process)
    // Actually StdioMcpServer logic tries to spawn process on connect.
    // We can register it, but listing tools will fail if it tries to connect.
    // But list_all_tools iterates and calls list_tools().
    // So we should verify with just SDK server for now, or Mock.

    // Check servers list
    let servers = manager.list_servers().await;
    assert_eq!(servers.len(), 1);
    assert!(servers.contains(&"sdk-server".to_string()));

    // Check tools list
    let all_tools = manager
        .list_all_tools()
        .await
        .expect("Failed to list tools");
    assert_eq!(all_tools.len(), 1);
    let (server_name, tool_info) = &all_tools[0];
    assert_eq!(server_name, "sdk-server");
    assert_eq!(tool_info.name, "test_tool");
}
