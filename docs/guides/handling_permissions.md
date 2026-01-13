# Handling Permissions

The Claude Agent Rust SDK includes a robust permission system to ensure that sensitive tools (like filesystem or shell access) are only used with explicit user approval.

## Permission Modes

You can configure the global permission mode in `ClaudeAgentOptions`:

```rust
use claude_agent_types::{ClaudeAgentOptions, PermissionMode};

let options = ClaudeAgentOptions {
    permission_mode: Some(PermissionMode::Prompt), // Default
    ..Default::default()
};
```

| Mode | Behavior |
|------|----------|
| `Approved` | All tools are authorized automatically. |
| `Prompt` | Requires user confirmation for each tool use. |
| `Deny` | All tool uses are rejected. |

## Tool Permission Callbacks

For more granular control, you can register a permission callback. This callback is triggered whenever the agent attempts to use a tool.

```rust
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::{ToolPermissionResult, ToolPermissionContext};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClaudeAgentClient::default();

    // Register a custom permission handler
    client.on_permission_request(|ctx| {
        Box::pin(async move {
            println!("Agent wants to use tool: {}", ctx.tool_name);
            println!("Arguments: {}", ctx.arguments);

            // Return logic (e.g., prompt user, check whitelist)
            if ctx.tool_name == "read_file" {
                ToolPermissionResult::Approve
            } else {
                ToolPermissionResult::Deny("Safety check failed".to_string())
            }
        })
    });

    client.connect().await?;
    // ...
}
```

## Security Best Practices

- Always use `PermissionMode::Prompt` (the default) for untrusted prompts.
- Implement specialized callbacks for sensitive operations like `write_file` or `execute_shell_command`.
- Use the `allowed_tools` option to restrict the set of tools available to the agent.
