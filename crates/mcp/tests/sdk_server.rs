use claude_agent_mcp::{McpServer, SdkMcpServer};
use claude_agent_types::ClaudeAgentError;
use serde_json::json;

#[tokio::test]
async fn test_sdk_server_tool_registration_and_call() {
    let mut server = SdkMcpServer::new("test-server");

    // Register "greet" tool
    server.register_tool(
        "greet",
        Some("Greets a user".to_string()),
        json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name"]
        }),
        |args| {
            Box::pin(async move {
                let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("World");
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Hello, {}!", name)
                    }]
                }))
            })
        },
    );

    // List tools
    let tools = server.list_tools().await.expect("Failed to list tools");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "greet");

    // Call tool
    let result = server
        .call_tool("greet", json!({"name": "Alice"}))
        .await
        .expect("Call failed");

    // Check result
    let content = result.get("content").expect("No content");
    assert_eq!(content[0]["text"], "Hello, Alice!");
}

#[tokio::test]
async fn test_tool_error_handling() {
    let mut server = SdkMcpServer::new("error-server");

    server.register_tool("fail", None, json!({}), |_| {
        Box::pin(async move { Err(ClaudeAgentError::Mcp("Expected fail".to_string())) })
    });

    let result = server.call_tool("fail", json!({})).await;
    assert!(result.is_err());
    match result {
        Err(ClaudeAgentError::Mcp(msg)) => assert_eq!(msg, "Expected fail"),
        _ => panic!("Expected message error"),
    }
}
