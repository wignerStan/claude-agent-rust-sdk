# Session Management

The Claude Agent Rust SDK provides powerful session management capabilities, allowing you to persist conversations and maintain context across multiple turns or even different executions of your application.

## Understanding Session IDs

Every interaction with the `ClaudeAgent` is associated with a `session_id`. If you don't provide one, a random ID is generated.

### Continuing a Conversation

To continue a previous conversation, pass the same `session_id` in the `ClaudeAgentOptions`:

```rust
use claude_agent_types::ClaudeAgentOptions;
use claude_agent_api::ClaudeAgentClient;

let options = ClaudeAgentOptions {
    session_id: Some("my-unique-session-id".to_string()),
    ..Default::default()
};

let mut client = ClaudeAgentClient::new(Some(options));
```

## How Persistence Works

The `claude` CLI maintains the actual conversation state on the server or in local cache. By providing a `session_id`, you are telling the CLI to resume that specific state.

### New Session (Standard)
```rust
let options = ClaudeAgentOptions {
    session_continuation: Some(false), // Force new session
    ..Default::default()
};
```

### Resume Session
```rust
let options = ClaudeAgentOptions {
    session_continuation: Some(true), // Attempt to resume
    ..Default::default()
};
```

## Best Practices

- **Unique IDs**: Use UUIDs or other collision-resistant strings for production session IDs.
- **Client Reuse**: For high-performance multi-turn loops, reuse the same `ClaudeAgentClient` instance.
- **Cleanup**: Sessions are relatively long-lived in the CLI cache, but don't rely on them for indefinite storage.
