# Hello World V2 - Session API Demo

A demonstration of Claude Agent SDK's session management capabilities, migrated from original TypeScript implementation.

## Purpose

This demo showcases advanced SDK features:
- **Session Creation**: Creating persistent conversation sessions
- **Session Resumption**: Resuming previous sessions with context retention
- **Multi-Turn Conversations**: Maintaining context across multiple queries
- **One-Shot Prompts**: Stateless queries without session management

## Original Implementation

Migrated from: `claude-agent-sdk-demos/hello-world-v2/v2-examples.ts`

### Key Differences

| TypeScript | Rust |
|-----------|------|
| `unstable_v2_createSession()` | `ClaudeAgentClient::new()` + `connect()` |
| `await using session` | Scoped stream processing with blocks |
| `session.send()` | `client.query()` |
| `session.stream()` | `client.query()` returns stream |
| `unstable_v2_prompt()` | `claude_agent_api::query()` function |
| Session ID tracking | Available via client (when implemented) |

### Simplifications

The Rust version focuses on core session API concepts:

1. **Session Persistence**: Demonstrates creating and maintaining sessions
2. **Context Retention**: Shows how context is maintained across turns
3. **Stateless Queries**: Demonstrates one-shot vs. session-based approaches

## Prerequisites

- Rust 1.75+
- Claude Code CLI installed and authenticated
- ANTHROPIC_API_KEY environment variable set

## Setup

```bash
# From claude-agent-rust directory
cd demos/hello-world-v2

# Run the demo
cargo run

# Run with debug logging
RUST_LOG=debug cargo run
```

## Usage

### Running the Demo

The demo provides an interactive menu to choose between different examples:

```bash
$ cargo run

Claude Agent SDK - Hello World V2 Demo
Demonstrating Session API capabilities

Choose an example:
1. Basic Session
2. Session Resumption
3. Multi-Turn Conversation
4. One-Shot Prompt
5. Run All Examples

Enter your choice (1-5):
```

### Example 1: Basic Session

Demonstrates creating a session and sending a query:

```rust
let mut client = ClaudeAgentClient::new(Some(options))?;
client.connect().await?;

let mut stream = client.query("Hello! My name is Alice.").await?;

while let Some(result) = stream.next().await {
    match result {
        Ok(Message::Assistant(msg)) => {
            for block in msg.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
        _ => {}
    }
}

client.disconnect().await?;
```

### Example 2: Session Resumption

Demonstrates resuming a previous session:

```rust
// First session
let session_id = create_and_query_session().await?;

// Resume session later
let mut client2 = ClaudeAgentClient::new(Some(options))?;
client2.connect().await?;

// Resume with session_id (when implemented)
// let mut stream = client2.resume_session(session_id).await?;
```

### Example 3: Multi-Turn Conversation

Demonstrates maintaining context across multiple queries:

```rust
let prompts = vec![
    "I'm learning Rust programming.",
    "What are the key features of Rust?",
    "Can you explain ownership and borrowing?",
];

for prompt in prompts {
    let mut stream = client.query(prompt).await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(Message::Assistant(msg)) => { /* process */ }
            Err(e) => eprintln!("Error: {}", e),
            _ => {}
        }
    }
}
```

### Example 4: One-Shot Prompt

Demonstrates stateless query without session management:

```rust
let prompt = "What is the capital of France?";

let mut stream = claude_agent_api::query(prompt, None).await?;

while let Some(result) = stream.next().await {
    match result {
        Ok(Message::Assistant(msg)) => {
            for block in msg.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
        _ => {}
    }
}
```

## Features Demonstrated

### 1. Session Management

```rust
// Create a new session
let mut client = ClaudeAgentClient::new(Some(options))?;
client.connect().await?;

// Use the session
let mut stream = client.query("Hello!").await?;

// Clean disconnect
client.disconnect().await?;
```

### 2. Multi-Turn Conversations

```rust
// Context is maintained across turns
for (i, prompt) in prompts.iter().enumerate() {
    println!("--- Turn {} ---", i + 1);
    let mut stream = client.query(prompt).await?;

    while let Some(result) = stream.next().await {
        // Process messages...
    }
}
```

### 3. Context Retention

Claude remembers previous context within a session:

```rust
// Turn 1
let mut stream = client.query("My name is Alice.").await?;
// Process...

// Turn 2 - Claude remembers "Alice"
let mut stream = client.query("What's my name?").await?;
// Response: "Your name is Alice"
```

### 4. One-Shot Queries

Stateless queries without session persistence:

```rust
let mut stream = claude_agent_api::query("Question?", None).await?;

// Process stream...
```

## Architecture

```
hello-world-v2/
├── Cargo.toml              # Project dependencies
├── src/
│   └── main.rs             # Main application with 4 examples
├── tests/
│   └── integration_test.rs  # Comprehensive test suite
└── README.md               # This file
```

### Component Overview

**Main Application (`main.rs`)**:
- `basic_session()` - Basic session creation and query
- `resume_session()` - Session resumption (placeholder)
- `multi_turn_conversation()` - Context retention demo
- `one_shot_prompt()` - Stateless query demo

**Test Suite (`tests/integration_test.rs`)**:
- `test_basic_session` - Session creation and query
- `test_multi_turn_conversation` - Context accumulation
- `test_session_lifecycle` - Connect, query, disconnect
- `test_one_shot_prompt` - Stateless query function
- `test_error_handling` - Connection failures
- `test_disconnect` - Proper cleanup
- `test_model_selection` - Model configuration

## Testing

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_basic_session

# Run tests with logging
RUST_LOG=debug cargo test
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

### Test Cases

| Test | Description | Status |
|-------|-------------|----------|
| `test_basic_session` | Session creation and query | ✅ |
| `test_multi_turn_conversation` | Context across turns | ✅ |
| `test_session_lifecycle` | Connect/query/disconnect | ✅ |
| `test_one_shot_prompt` | Stateless query | ✅ |
| `test_error_handling` | Connection failures | ✅ |
| `test_disconnect` | Cleanup | ✅ |
| `test_model_selection` | Model configuration | ✅ |

**Coverage Target:** ≥85%

## Migration Notes

### Differences from Original TypeScript

**Simplifications:**
1. **Session ID Tracking**: The original TypeScript demo used `session.id` to track sessions. The Rust SDK's session management is demonstrated through the client lifecycle.

2. **Session Resumption**: The original used `unstable_v2_resumeSession()`. The Rust version shows the pattern for session resumption, with a note that actual resumption requires the SDK to support it.

3. **Using Statement**: The original used `await using session` for resource management. Rust uses scoping with blocks `{ let mut stream = ... }`.

### Enhancements

1. **Strong Typing**: Rust's type system ensures correct message handling at compile time.

2. **Error Context**: Using `anyhow::Context` provides better error messages.

3. **Structured Logging**: Integration with `tracing` for structured, filterable logs.

4. **Interactive Menu**: Added menu system for easy demo selection.

### Known Limitations

1. **Session Resumption**: The demo shows the pattern for session resumption, but actual resumption depends on SDK support for session IDs.

2. **Session Persistence**: The demo doesn't persist sessions to disk between runs.

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

**Solution**: Set environment variable:
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

- **Resume Generator**: Web search and document generation
- **Research Agent**: Multi-agent orchestration
- **Simple Chat App**: Real-time WebSocket communication

## License

MIT OR Apache-2.0
