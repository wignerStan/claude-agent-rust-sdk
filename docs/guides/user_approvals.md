# User Approvals and Input

The SDK supports interactive patterns where the agent requires approval for certain actions or specifically requests additional input from the user.

## Handle Interactive Prompts

When the `claude` CLI needs user input (e.g., during a git merge or for clarification), it sends a control request that the SDK captures.

### Implementing Approval Loops

You can handle approvals through the `on_permission_request` callback or by monitoring the message stream for specific block types.

```rust
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ToolPermissionResult;

async fn run_agent() {
    let mut client = ClaudeAgentClient::default();

    // Automatic approval based on tool type
    client.on_test_hook(|event| {
        // Log event for transparency
        true
    });

    client.connect().await.unwrap();
}
```

## Control Protocol for Inputs

The SDK automatically handles the underlying control protocol. When a "permission prompt" message is received from the CLI, the SDK can:
1. Trigger a callback.
2. Wait for user input via standard input (if attached).
3. Resume execution based on the response.

## Best Practices for UX

- **Non-blocking input**: If building a GUI, ensure your input collection doesn't block the main event loop.
- **Contextual information**: Show the user the exact command and arguments the agent is proposing.
- **Timeout handling**: Prepare for cases where the user doesn't respond to a prompt for an extended period.
