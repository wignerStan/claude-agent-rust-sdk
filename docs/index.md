# Claude Agent Rust SDK Documentation

Welcome to the comprehensive documentation for the Claude Agent Rust SDK. This SDK provides a robust, type-safe interface for interacting with Claude via the Claude Code CLI and Model Context Protocol (MCP).

## Getting Started
- [Basic Usage](guides/streaming_input.md): Start your first conversation with Claude.
- [Session Management](guides/session_management.md): Persist context across turns.

## Core Guides

### Interaction & Control
- [Streaming Input](guides/streaming_input.md): Real-time response handling.
- [Handling Permissions](guides/handling_permissions.md): Secure tool execution.
- [User Approvals](guides/user_approvals.md): Human-in-the-loop patterns.
- [Control Execution with Hooks](guides/hooks.md): Event-driven agent control.
- [Slash Commands](guides/slash_commands.md): Triggering system actions.

### Agent Capabilities
- [Subagents](guides/subagents.md): Delegating tasks to specialized agents.
- [Agent Skills](guides/agent_skills.md): Exploring the agent's toolbelt.
- [MCP Integration](guides/mcp.md): Connecting external data and tools.
- [Custom Tools](guides/custom_tools.md): Writing skills in pure Rust.

### Configuration & Data
- [Modifying System Prompts](guides/system_prompts.md): Defining agent persona.
- [Tracking Cost & Usage](guides/cost_usage.md): Monitoring token consumption.
- [Structured Outputs](guides/structured_output.md): Extracting data with JSON Schema.
- [Plugins](guides/plugins.md): Extending the agent ecosystem.

### MCP Transports
- [Transport Selection Guide](guides/transport-selection.md): Choosing the right transport.
- [HTTP Transport](guides/http-transport.md): HTTP-based JSON-RPC connections.
- [SSE Transport](guides/sse-transport.md): Server-Sent Events transport.

### Safety & Advanced Features
- [File Checkpointing & Rewind](guides/file_checkpointing.md): Safe undos of agent actions.

## Integration & Deployment
- [Hosting the SDK](#): Coming soon.
- [Secure Deployment](#): Best practices for production.
- [Testing Guide](guides/testing.md): Comprehensive testing strategy.

## Reference
- [API Documentation](https://docs.rs/claude-agent-api): High-level Rust API reference.
- [Crate Documentation](https://github.com/anthropics/claude-agent-sdk-rust): Source code and README.
