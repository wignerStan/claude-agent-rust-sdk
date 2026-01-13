# HTTP Transport Guide

This guide covers using the HTTP-based MCP transport in `claude-agent-rust`.

## Overview

The `HttpMcpServer` provides a fully functional HTTP-based MCP client that connects to remote MCP servers using JSON-RPC 2.0 over HTTP POST.

## Features

- **JSON-RPC 2.0**: Full protocol support with request IDs and error handling
- **Timeout Handling**: Configurable request timeouts (default: 30 seconds)
- **Tool Operations**: List tools, call tools, and handle client messages
- **Error Handling**: Proper error propagation for HTTP and JSON-RPC errors

## Basic Usage

```rust
use claude_agent_mcp::transports::HttpMcpServer;
use claude_agent_mcp::manager::McpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an HTTP MCP server client
    let server = HttpMcpServer::new(
        "my_mcp_server".to_string(),
        "http://localhost:8080/mcp".to_string(),
    )?;

    // List available tools
    let tools = server.list_tools().await?;
    println!("Available tools: {:?}", tools);

    // Call a tool
    let result = server.call_tool(
        "greet",
        serde_json::json!({"name": "World"}),
    ).await?;
    println!("Result: {:?}", result);

    Ok(())
}
```

## Custom Timeout

```rust
use claude_agent_mcp::transports::HttpMcpServer;
use std::time::Duration;

// Create with 60-second timeout
let server = HttpMcpServer::with_timeout(
    "my_server".to_string(),
    "http://localhost:8080/mcp".to_string(),
    Duration::from_secs(60),
)?;
```

## Error Handling

The `HttpMcpServer` returns `ClaudeAgentError::Mcp` for various error conditions:

```rust
use claude_agent_mcp::transports::HttpMcpServer;
use claude_agent_mcp::manager::McpServer;

async fn example() {
    let server = HttpMcpServer::new("test".to_string(), "http://localhost:8080".to_string())
        .expect("Failed to create server");

    match server.list_tools().await {
        Ok(tools) => println!("Got {} tools", tools.len()),
        Err(e) => {
            // Error types:
            // - "HTTP request failed: ..." - Connection errors
            // - "HTTP error: 500 Internal Server Error" - HTTP status errors
            // - "JSON parse error: ..." - Invalid response format
            // - "JSON-RPC error (code -32600): ..." - Protocol errors
            eprintln!("Error: {}", e);
        }
    }
}
```

## Integration with McpServerManager

```rust
use claude_agent_mcp::manager::McpServerManager;
use claude_agent_mcp::transports::HttpMcpServer;
use std::sync::Arc;

async fn example() {
    let mut manager = McpServerManager::new();

    let http_server = Arc::new(HttpMcpServer::new(
        "remote_tools".to_string(),
        "http://tools.example.com/mcp".to_string(),
    ));

    manager.register(http_server);

    // Now the manager can use tools from the HTTP server
    let tools = manager.list_all_tools().await;
}
```

## Protocol Details

The `HttpMcpServer` uses the standard JSON-RPC 2.0 protocol over HTTP POST:

**Request Format:**
```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
}
```

**Response Format:**
```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "tools": [
            {
                "name": "greet",
                "description": "Greet someone",
                "input_schema": {"type": "object"}
            }
        ]
    }
}
```

## See Also

- [SSE Transport Guide](./sse-transport.md)
- [Stdio Transport Guide](./stdio-transport.md)
- [MCP Protocol Specification](https://modelcontextprotocol.io/)
