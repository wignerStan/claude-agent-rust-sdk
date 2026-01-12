# Control Execution with Hooks

The Hook system in the Claude Agent Rust SDK allows you to intercept, inspect, and potentially modify the interaction between the SDK and the Claude Code CLI.

## Hook Registry

Hooks are registered on the `ClaudeAgent` or `ClaudeAgentClient`. They use a callback pattern where your function is called when a specific event occurs.

### Supported Hook Types

| Hook | Triggered When... |
|------|-----------|
| `PreUserMessage` | Before a user message is sent to the CLI. |
| `PostAssistantMessage` | Immediately after an assistant message is received. |
| `PreToolUse` | Before a tool execution is initiated. |
| `PostToolResult` | After a tool returns its result. |

## Registering a Hook

```rust
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::HookContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClaudeAgentClient::default();

    // Register a hook to log all assistant responses
    client.add_hook(move |context| {
        if let Some(msg) = context.assistant_message {
            println!("Assistant says: {:?}", msg);
        }
        Ok(()) // Allow execution to proceed
    });

    client.connect().await?;
    // ...
}
```

## Advanced Use Cases

### Message Transformation
While hooks are primarily for inspection in the current version, they provide the foundation for future message transformation or filtering.

### Integration with External Systems
Use hooks to send telemetry, update external databases, or trigger third-party workflows based on agent activity.

> [!NOTE]
> Hooks are executed sequentially. Heavy computation in a hook will delay the agent response.
