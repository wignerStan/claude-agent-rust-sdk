//! MCP Calculator example - Implementing a custom MCP tool.

use claude_agent_mcp::SdkMcpServer;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create SDK MCP server for custom tools
    let mut server = SdkMcpServer::new("calculator");

    // Register a calculator tool
    server.register_tool(
        "add",
        Some("Add two numbers".to_string()),
        json!({
            "type": "object",
            "properties": {
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["a", "b"]
        }),
        |args: Value| {
            Box::pin(async move {
                let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                Ok(json!({"content": [{"type": "text", "text": format!("{}", a + b)}]}))
            })
        },
    );

    println!("Calculator MCP server created with 'add' tool");
    println!("In a full example, this would be registered with ClaudeAgent");

    // To use with ClaudeAgent:
    // agent.mcp_manager_mut().register("calculator", Box::new(server));

    Ok(())
}
