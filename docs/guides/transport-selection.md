# Transport Selection Guide

This guide explains how to configure and select MCP transports in `claude-agent-rust`.

## Overview

The MCP crate supports three transport types:

| Transport | Description | Use Case |
|-----------|-------------|----------|
| **Stdio** | Subprocess-based, communicates via stdin/stdout | Local tools, scripts |
| **HTTP** | HTTP POST with JSON-RPC 2.0 | Remote MCP servers |
| **SSE** | Server-Sent Events with HTTP POST | Real-time remote servers |

## Configuration

### Using `McpServerConfig`

```rust
use claude_agent_types::config::{McpServerConfig, McpTransportType};

// HTTP transport
let http_config = McpServerConfig {
    transport: McpTransportType::Http,
    url: Some("http://localhost:8080/mcp".to_string()),
    timeout_secs: Some(60),
    ..Default::default()
};

// Stdio transport
let stdio_config = McpServerConfig {
    transport: McpTransportType::Stdio,
    command: Some("python".to_string()),
    args: vec!["-m".to_string(), "my_mcp_server".to_string()],
    ..Default::default()
};

// SSE transport
let sse_config = McpServerConfig {
    transport: McpTransportType::Sse,
    url: Some("http://localhost:8080/sse".to_string()),
    ..Default::default()
};
```

### Using the Transport Factory

```rust
use claude_agent_mcp::create_mcp_server;
use claude_agent_types::config::{McpServerConfig, McpTransportType};

let config = McpServerConfig {
    transport: McpTransportType::Http,
    url: Some("http://tools.example.com/mcp".to_string()),
    ..Default::default()
};

let server = create_mcp_server("remote_tools".to_string(), config)?;

// Register with manager
manager.register(server);
```

## Auto Transport Selection

The `Auto` transport type automatically selects the best transport:

```rust
use claude_agent_types::config::{McpServerConfig, McpTransportType};

// Auto mode with URL -> uses HTTP
let config = McpServerConfig {
    transport: McpTransportType::Auto,
    url: Some("http://localhost:8080".to_string()),
    ..Default::default()
};

// Auto mode with command -> uses Stdio
let config = McpServerConfig {
    transport: McpTransportType::Auto,
    command: Some("node".to_string()),
    args: vec!["server.js".to_string()],
    ..Default::default()
};
```

## Transport Comparison

### Stdio Transport
- **Pros**: Works locally, no network overhead, secure
- **Cons**: Limited to local processes
- **Best for**: Local scripts, development

### HTTP Transport
- **Pros**: Works with remote servers, stateless
- **Cons**: Higher latency, requires network
- **Best for**: Remote MCP servers, microservices

### SSE Transport
- **Pros**: Real-time updates, persistent connection
- **Cons**: More complex setup
- **Best for**: Long-running operations, streaming results

## Error Handling

All transports return `ClaudeAgentError` for failures:

```rust
use claude_agent_mcp::create_mcp_server;
use claude_agent_types::config::{McpServerConfig, McpTransportType};

let config = McpServerConfig {
    transport: McpTransportType::Http,
    // Missing URL!
    ..Default::default()
};

match create_mcp_server("test".to_string(), config) {
    Ok(server) => println!("Created server"),
    Err(e) => eprintln!("Configuration error: {}", e),
}
```

## See Also

- [HTTP Transport Guide](./http-transport.md)
- [SSE Transport Guide](./sse-transport.md)
