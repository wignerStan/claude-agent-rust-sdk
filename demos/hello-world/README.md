# Hello World Demo

A basic demonstration of Claude Agent SDK usage, migrated from the original TypeScript implementation.

## Purpose

This demo showcases fundamental SDK features:
- Basic client initialization and connection
- Query execution with message streaming
- Tool whitelisting for security
- Custom working directory configuration
- Message content extraction and display

## Original Implementation

Migrated from: `claude-agent-sdk-demos/hello-world/hello-world.ts`

### Key Differences

| TypeScript | Rust |
|-----------|------|
| `query()` function | `ClaudeAgentClient::query()` |
| Async generators | `Stream<Item = Result<Message>>` |
| `for await...of` | `while let Some(...) = stream.next().await` |
| `message.type === 'assistant'` | `matches!(message, Message::Assistant(_))` |
| Optional chaining | Pattern matching with `if let` |

## Prerequisites

- Rust 1.75+
- Claude Code CLI installed and authenticated
- ANTHROPIC_API_KEY environment variable set

## Setup

```bash
# From the claude-agent-rust directory
cd demos/hello-world

# Create agent directory
mkdir -p agent/custom_scripts

# Run the demo
cargo run
```

## Usage

### Basic Execution

```bash
# Run with default settings
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Run with trace logging
RUST_LOG=trace cargo run
```

### Code Example

```rust
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure options
    let options = ClaudeAgentOptions {
        max_turns: Some(100),
        model: Some("claude-opus-4-1-20250805".to_string()),
        allowed_tools: vec![
            "Read".to_string(),
            "Write".to_string(),
            "Bash".to_string(),
            // ... more tools
        ],
        ..Default::default()
    };

    // Create and connect client
    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await?;

    // Send query
    let mut stream = client.query("Hello, Claude!").await?;

    // Process messages
    while let Some(result) = stream.next().await {
        match result {
            Ok(Message::Assistant(msg)) => {
                for block in msg.content {
                    if let ContentBlock::Text(text) = block {
                        println!("{}", text.text);
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
            _ => {}
        }
    }

    // Disconnect
    client.disconnect().await?;
    Ok(())
}
```

## Features Demonstrated

### 1. Client Initialization

```rust
let mut client = ClaudeAgentClient::new(Some(options));
client.connect().await?;
```

### 2. Tool Whitelisting

```rust
allowed_tools: vec![
    "Task".to_string(),
    "Bash".to_string(),
    "Read".to_string(),
    "Write".to_string(),
    // ... more tools
],
```

### 3. Message Streaming

```rust
let mut stream = client.query("Hello, Claude!").await?;

while let Some(result) = stream.next().await {
    match result {
        Ok(message) => { /* process message */ }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### 4. Content Extraction

```rust
if let Message::Assistant(msg) = message {
    for block in msg.content {
        if let ContentBlock::Text(text_block) = block {
            println!("{}", text_block.text);
        }
    }
}
```

## Testing

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_hello_world_basic_query
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html --output-dir coverage/

# View report
open coverage/index.html
```

### Test Files

- `tests/integration_test.rs` - Integration tests with mock transport

### Coverage Target

≥90% coverage required for this demo.

## Architecture

```
hello-world/
├── Cargo.toml              # Project dependencies
├── src/
│   └── main.rs             # Main application
├── tests/
│   └── integration_test.rs  # Integration tests
└── README.md               # This file
```

## Error Handling

The demo uses Rust's `Result<T, E>` type for error handling:

```rust
use anyhow::{Context, Result};

client.connect().await.context("Failed to connect")?;
```

Errors are propagated with the `?` operator and provide context using `anyhow::Context`.

## Migration Notes

### Simplifications

The Rust version simplifies some aspects of the original TypeScript demo:

1. **Hook System**: The original TypeScript demo used hooks for file path validation. The Rust version focuses on core SDK usage, with hooks demonstrated in other demos.

2. **File Path Validation**: The original enforced that .js/.ts files be written to `custom_scripts` directory. This is left as an exercise for the reader.

3. **Executable Path**: The original specified `executable: "node"`. The Rust SDK uses the Claude Code CLI directly.

### Enhancements

1. **Strong Typing**: Rust's type system provides compile-time safety for message handling.

2. **Error Context**: Using `anyhow::Context` provides better error messages.

3. **Logging**: Integration with `tracing` for structured logging.

## Troubleshooting

### Connection Errors

```
Error: Failed to connect to Claude Code
```

**Solution**: Ensure Claude Code CLI is installed and authenticated:
```bash
claude --version
claude login
```

### API Key Errors

```
Error: ANTHROPIC_API_KEY not found
```

**Solution**: Set the environment variable:
```bash
export ANTHROPIC_API_KEY=your_key_here
```

Or create a `.env` file:
```
ANTHROPIC_API_KEY=your_key_here
```

### Build Errors

```
Error: failed to run custom build command
```

**Solution**: Ensure Rust 1.75+ is installed:
```bash
rustc --version
rustup update
```

## Next Steps

After mastering this demo, explore:

- **Hello World V2**: Session API and multi-turn conversations
- **Resume Generator**: Web search and document generation
- **Research Agent**: Multi-agent orchestration

## License

MIT OR Apache-2.0
