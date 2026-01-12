# Claude Agent SDK (Rust)

A pure Rust migration of the Claude Agent SDK, providing a powerful interface to Claude Code with official Model Context Protocol (MCP) integration.

## Features

- **High-Level API**: Easy-to-use client for one-shot queries and interactive sessions.
- **Model Context Protocol (MCP)**: First-class support for MCP servers and tools.
- **Agentic Workflow**: Built-in support for tool use, computer use, and multi-turn conversations.
- **Robustness**: Type-safe control flow, error handling, and reconnection logic.
- **Extensible**: Hook system for intercepting agent actions and messages.

## Project Structure

- `crates/types`: Core type definitions and error types.
- `crates/transport`: Low-level communication with Claude Code CLI (async/await).
- `crates/mcp`: Model Context Protocol (MCP) integration manager.
- `crates/core`: Internal agent logic, session management, and control flow.
- `crates/api`: High-level public API for developers.

## Getting Started

### Prerequisites

- Rust 1.75+
- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) installed and available in `PATH`.
  - Authenticate via `claude login`.

### Installation

Add `claude-agent-api` to your `Cargo.toml`:

```toml
[dependencies]
claude-agent-api = { path = "crates/api" } # Currently local
tokio = { version = "1.0", features = ["full"] }
```

### Quick Start

```rust
use claude_agent_api::ClaudeAgentClient;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Client
    let mut client = ClaudeAgentClient::default();
    client.connect().await?;

    // 2. Send a query
    let mut stream = client.query("What is 2 + 2?").await?;

    // 3. Process the response stream
    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => println!("Received: {:?}", msg),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

## Examples

Check out the `examples` directory for more use cases:

- `quick_start.rs`: Basic query.
- `mcp_calculator.rs`: integrating a simple calculator tool via MCP.
- `hooks.rs`: Using hooks to intercept messages.
- `streaming_mode.rs`: real-time usage.

Run an example:
```bash
cargo run -p claude-agent-api --example quick_start
```

## Development

### Building

```bash
cargo build --all-features
```

### Running Tests

```bash
# Unit tests
cargo test --workspace

# E2E Tests (Requires Anthropic API Key)
./run_e2e.sh
```

## License

MIT OR Apache-2.0

