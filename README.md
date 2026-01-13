# Claude Agent Rust SDK

A comprehensive Rust SDK for building agents that interact with Claude Code CLI via the Model Context Protocol (MCP).

[![Crates.io](https://img.shields.io/crates/d/claude-agent-rust)](https://crates.io/crates/claude-agent-rust)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache-2.0-blue)](https://github.com/claude-agent-rust/claude-agent-rust/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.83+-orange)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/github/actions/workflows/CI/claude-agent-rust/claude-agent-rust/badge.svg)](https://github.com/claude-agent-rust/claude-agent-rust/actions)
[![Security](https://img.shields.io/badge/Security-cargo--deny-green)](https://github.com/EmbarkStudios/cargo-deny)
[![codecov](https://codecov.io/gh/claude-agent-rust/claude-agent-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/claude-agent-rust/claude-agent-rust)

## Features

- **Transport Layer**: Subprocess-based communication with Claude Code CLI
- **MCP Server Management**: Multi-transport support (Stdio, HTTP, SSE) with rate limiting
- **Security**: Secret handling with `secrecy`, constant-time comparisons with `subtle`
- **Session Management**: Checkpoint-based conversation state tracking
- **Control Protocol**: Bidirectional communication with Claude Code CLI
- **Hook System**: Extensible callback system for custom behavior
- **Testing**: Comprehensive property-based, unit, and E2E testing
- **Streaming Parser**: Highly efficient JSON-RPC message parsing
- **Type Definitions**: Comprehensive type system for all SDK components

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
claude-agent-rust = "0.1.0"
```

## Quick Start

### Basic Usage

```rust
use claude_agent_core::{ClaudeAgent, ClaudeAgentOptions};
use claude_agent_types::Message;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions::default();
    let mut agent = ClaudeAgent::new(options);

    // Connect to Claude
    agent.connect(None).await?;

    // Send a query
    let mut stream = agent.query("What is 2+2?").await?;

    // Process streaming responses
    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => println!("Response: {:?}", msg),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    // Cleanup
    agent.disconnect().await?;
    Ok(())
}
```

### MCP Server Integration

```rust
use claude_agent_mcp::{McpServerManager, McpServer};
use claude_agent_mcp::StdioMcpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = McpServerManager::new();

    // Register a custom MCP server
    let server = StdioMcpServer::new(
        "my_server".to_string(),
        "python".to_string(),
        vec!["-m".to_string(), "my_mcp_server".to_string()]
    );

    manager.register(Box::new(server));

    // List available tools
    let tools = manager.list_all_tools().await?;
    println!("Available tools: {:?}", tools);

    Ok(())
}
```

## Project Structure

```
claude-agent-rust/
├── Cargo.toml              # Workspace configuration
├── crates/                 # Workspace members
│   ├── types/            # Type definitions
│   ├── transport/         # Transport layer
│   ├── mcp/              # MCP server management
│   ├── core/             # Core agent logic
│   └── api/              # API client
├── demos/                 # Example applications
├── CHANGELOG.md           # Change history
├── ROADMAP.md            # Development roadmap
├── CONFIGURATION.md      # Configuration documentation
├── LICENSE-MIT            # MIT license
├── LICENSE-APACHE         # Apache 2.0 license
├── CONTRIBUTING.md        # Contribution guidelines
├── CODE_OF_CONDUCT.md    # Community standards
└── README.md              # This file
```

## Documentation

- [CHANGELOG.md](CHANGELOG.md) - Detailed change history
- [ROADMAP.md](ROADMAP.md) - Development roadmap and milestones
- [CONFIGURATION.md](CONFIGURATION.md) - Configuration file documentation
- [PRODUCTION_CONFIG.md](PRODUCTION_CONFIG.md) - Production setup guide
- [API Documentation](https://docs.rs/claude-agent-rust) - Comprehensive API docs

## Development

### Building

```bash
# Build all workspace members
cargo build --workspace

# Build specific crate
cargo build -p claude-agent-core
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test -p claude-agent-transport -- test_name
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace --all-targets

# Check code
cargo check --workspace
```

### Using Just

```bash
# Build workspace
just build

# Run all tests
just test

# Run all quality checks
just qa

# Format code
just fmt

# Run linter
just lint

# Check code
just check
```

## Configuration

The SDK uses workspace inheritance for dependencies. See [CONFIGURATION.md](CONFIGURATION.md) for detailed configuration options.

### Environment Variables

- `CLAUDE_CLI_PATH` - Path to Claude Code CLI binary (optional)
- `RUST_LOG` - Logging level (e.g., `debug`, `info`, `warn`, `error`)

## License

This project is dual-licensed under:

- MIT License - See [LICENSE-MIT](LICENSE-MIT)
- Apache License 2.0 - See [LICENSE-APACHE](LICENSE-APACHE)

You may choose to use either license for your use case.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Code of Conduct

Please be respectful and inclusive. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for details.

## Support

- [GitHub Issues](https://github.com/claude-agent-rust/claude-agent-rust/issues)
- [GitHub Discussions](https://github.com/claude-agent-rust/claude-agent-rust/discussions)
- [Documentation](https://docs.rs/claude-agent-rust)

## Roadmap

See [ROADMAP.md](ROADMAP.md) for planned features and development priorities.

---

**Note:** This SDK is in active development. APIs may change as we approach v1.0.0.
