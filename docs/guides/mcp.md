# MCP in the SDK

The Model Context Protocol (MCP) is a core feature of the Claude Agent Rust SDK, allowing the agent to interact with a wide ecosystem of external data and tools.

## Architecture

The SDK uses an internal `McpServerManager` to coordinate multiple MCP servers.

| Component | Responsibility |
|-----------|----------------|
| `McpServerManager` | Registration, lifecycle, and tool aggregation. |
| `StdioMcpServer` | Transport for local subprocess-based servers. |
| `SdkMcpServer` | Wrapper for using tools defined directly in the SDK. |

## Registering an MCP Server

The easiest way to add MCP capabilities is through `ClaudeAgentOptions`:

```rust
use claude_agent_types::{ClaudeAgentOptions, McpServerConfig};

let options = ClaudeAgentOptions {
    mcp_servers: vec![
        McpServerConfig::Stdio {
            name: "calculator".to_string(),
            command: "npx".to_string(),
            args: vec!["-y", "@modelcontextprotocol/server-calculator".to_string()],
        }
    ],
    ..Default::default()
};
```

## How Tools are Discovered

When the agent starts, the SDK:
1. Connects to all configured MCP servers.
2. Calls `list_tools` on each server.
3. Merges the results into a global tool registry.
4. Provides this registry to the Claude CLI.

## Best Practices

- **Security**: Be careful when registering third-party MCP servers. They can execute code or access your filesystem.
- **Timeouts**: MCP calls are asynchronous. Ensure your application handles potential network or process delays gracefully.
- **Resource Management**: The SDK automatically manages the lifecycle of stdio-based servers.
