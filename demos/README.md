# Claude Agent SDK Demos

This directory contains Rust demonstrations of the Claude Agent SDK, migrated from the original Python and TypeScript implementations.

## Overview

Each demo showcases different capabilities and use cases of the Claude Agent SDK:

| Demo | Description | Complexity | Status |
|-------|-------------|-------------|----------|
| **hello-world** | Basic SDK usage with hooks | Low | ✅ Complete |
| **hello-world-v2** | Session API and multi-turn conversations | Low | ✅ Complete |
| **resume-generator** | Web search and document generation | Medium | ✅ Complete |
| **research-agent** | Multi-agent research system with PDF generation | High | ✅ Complete |
| **simple-chatapp** | Real-time chat with WebSocket | Medium | ✅ Complete |
| **email-agent** | IMAP email assistant (simplified) | High | ✅ Complete |

## Getting Started

### Prerequisites

- Rust 1.75+
- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) installed and authenticated
- ANTHROPIC_API_KEY environment variable set

### Running Demos

Each demo can be run independently:

```bash
# Run a specific demo
cargo run -p hello-world

# Run with logging
RUST_LOG=debug cargo run -p hello-world

# Run tests for a demo
cargo test -p hello-world

# Run all demo tests
cargo test --workspace
```

## Project Structure

```
demos/
├── Cargo.toml              # Workspace configuration
├── README.md               # This file
├── common/                 # Shared utilities and test infrastructure
│   ├── src/
│   │   ├── lib.rs
│   │   └── test_utils.rs  # Mock transport, test helpers
├── hello-world/            # Basic SDK usage
├── hello-world-v2/         # Session API
├── resume-generator/        # Document generation
├── research-agent/          # Multi-agent system
├── simple-chatapp/          # WebSocket chat
└── email-agent/            # IMAP email assistant
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html --output-dir coverage/

# Run specific demo tests
cargo test -p hello-world
```

### Coverage Requirements

All demos maintain ≥80% test coverage:
- Unit tests for all modules
- Integration tests for workflows
- Mock transport for isolated testing
- E2E tests for live API (optional)

## Common Patterns

### 1. Client Initialization

```rust
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions {
        model: Some("claude-sonnet-4-5".to_string()),
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options))?;
    client.connect().await?;

    // Use client...

    client.disconnect().await?;
    Ok(())
}
```

### 2. Query and Stream Processing

```rust
use futures::StreamExt;

let mut stream = client.query("What is 2+2?").await?;

while let Some(result) = stream.next().await {
    match result {
        Ok(Message::Assistant(msg)) => {
            for block in msg.content {
                if let ContentBlock::Text(text) = block {
                    println!("{}", text.text);
                }
            }
        }
        Ok(Message::Result(_)) => break,
        Err(e) => eprintln!("Error: {}", e),
        _ => {}
    }
}
```

### 3. Hook System

```rust
use claude_agent_types::hooks::{HookEvent, HookMatcher};
use std::sync::Arc;

let hook = Arc::new(|input: HookInput, _id, _ctx| {
    Box::pin(async move {
        println!("Tool: {:?}", input.tool_name);
        Ok(HookOutput {
            continue_execution: true,
            ..Default::default()
        })
    })
});

let options = ClaudeAgentOptions {
    hooks: HashMap::from([(HookEvent::PreToolUse, vec![hook])]),
    ..Default::default()
};
```

## Migration Notes

These demos are migrated from the original Python/TypeScript implementations:

- **hello-world**: Migrated from TypeScript `claude-agent-sdk-demos/hello-world`
- **hello-world-v2**: Migrated from TypeScript `claude-agent-sdk-demos/hello-world-v2`
- **resume-generator**: Migrated from TypeScript `claude-agent-sdk-demos/resume-generator`
- **research-agent**: Migrated from Python `claude-agent-sdk-demos/research-agent`
- **simple-chatapp**: Migrated from TypeScript `claude-agent-sdk-demos/simple-chatapp`
- **email-agent**: Migrated from TypeScript `claude-agent-sdk-demos/email-agent` (simplified)

Key differences from original implementations:
- Strong typing with Rust's type system
- Async/await with tokio runtime
- Error handling with `Result<T, E>` instead of exceptions
- Ownership and borrowing for memory safety
- Streams instead of async generators

## Contributing

When adding new demos:

1. Create a new directory under `demos/`
2. Add to `demos/Cargo.toml` workspace members
3. Follow the project structure and patterns
4. Include comprehensive tests (≥80% coverage)
5. Add README.md with setup and usage instructions
6. Document migration notes from original implementation

## License

MIT OR Apache-2.0
