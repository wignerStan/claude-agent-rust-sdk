# SSE Transport Guide

This guide covers using the SSE-based MCP transport in `claude-agent-rust`.

## Overview

The `SseMcpServer` provides an SSE (Server-Sent Events) based MCP client that connects to remote MCP servers. It uses HTTP POST for sending requests and is designed to work with SSE-enabled MCP servers.

## Features

- **SSE-Compatible**: Designed for servers that use SSE for real-time updates
- **JSON-RPC 2.0**: Full protocol support over HTTP POST
- **Timeout Handling**: Configurable request timeouts (default: 30 seconds)
- **Tool Operations**: List tools, call tools, and handle client messages

## Basic Usage

```rust
use claude_agent_mcp::transports::SseMcpServer;
use claude_agent_mcp::manager::McpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an SSE MCP server client
    let server = SseMcpServer::new(
        "my_sse_server".to_string(),
        "http://localhost:8080/sse".to_string(),
    )?;

    // List available tools
    let tools = server.list_tools().await?;
    println!("Available tools: {:?}", tools);

    // Call a tool
    let result = server.call_tool(
        "analyze",
        serde_json::json!({"data": "sample"}),
    ).await?;
    println!("Result: {:?}", result);

    Ok(())
}
```

## Custom Timeout

```rust
use claude_agent_mcp::transports::SseMcpServer;
use std::time::Duration;

// Create with 120-second timeout for long-running operations
let server = SseMcpServer::with_timeout(
    "analysis_server".to_string(),
    "http://localhost:8080/sse".to_string(),
    Duration::from_secs(120),
)?;
```

## Error Handling

The `SseMcpServer` returns `ClaudeAgentError::Mcp` for various error conditions:

```rust
use claude_agent_mcp::transports::SseMcpServer;
use claude_agent_mcp::manager::McpServer;

async fn example() {
    let server = SseMcpServer::new("test".to_string(), "http://localhost:8080/sse".to_string())
        .expect("Failed to create server");

    match server.call_tool("process", serde_json::json!({})).await {
        Ok(result) => {
            // Check for tool-level errors
            if result.get("isError").and_then(|v| v.as_bool()) == Some(true) {
                eprintln!("Tool execution failed");
            } else {
                println!("Success: {:?}", result);
            }
        }
        Err(e) => eprintln!("Transport error: {}", e),
    }
}
```

## Comparison with HTTP Transport

| Feature | HttpMcpServer | SseMcpServer |
|---------|--------------|--------------|
| Request Method | HTTP POST | HTTP POST |
| Response Format | JSON | JSON |
| Real-time Updates | No | Designed for SSE servers |
| Use Case | Simple request/response | Long-running operations |

## See Also

- [HTTP Transport Guide](./http-transport.md)
- [Stdio Transport Guide](./stdio-transport.md)
- [MCP Protocol Specification](https://modelcontextprotocol.io/)
