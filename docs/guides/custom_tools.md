# Custom Tools

While MCP is the preferred way to integrate external capabilities, the Claude Agent Rust SDK also allows you to define custom tools directly in your code using `SdkMcpServer` and `JsonSchema`.

## Defining a Custom Tool

Use the `schemars` crate to define the input schema for your tool automatically.

```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(JsonSchema, Deserialize)]
struct GetWeatherArgs {
    city: String,
    units: Option<String>,
}
```

## Registering the Tool

You can add your tool to an `SdkMcpServer` and then register that server with the agent.

```rust
use claude_agent_mcp::{SdkMcpServer, ToolDefinition};
use claude_agent_api::ClaudeAgentClient;

let mut server = SdkMcpServer::new("my-custom-tools");

server.register_tool(
    ToolDefinition::from_type::<GetWeatherArgs>("get_weather", Some("Fetch local weather".to_string())),
    |args| {
        Box::pin(async move {
            let city = args["city"].as_str().unwrap();
            serde_json::json!({ "temperature": 22, "condition": "Sunny" })
        })
    }
);
```

## Manual Tool Execution Flow

1. **Tool Discovery**: The agent identifies the tool from the schema.
2. **Tool Use Block**: The agent sends a `ToolUse` message in the stream.
3. **Execution**: The SDK executes your registered closure.
4. **Tool Result Block**: The SDK sends the result back to the CLI.

## Advanced Tip: Complex Schemas

The SDK supports complex nested objects and enums through `JsonSchema`. This allows you to build highly expressive tool interfaces with strict validation.
