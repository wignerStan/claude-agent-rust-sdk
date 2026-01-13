# Product Guide - Claude Agent Rust SDK

## Vision
To provide a comprehensive, production-grade Rust SDK for building autonomous agents and AI-powered tools. We aim to give developers a robust runtime environment that handles the complexities of agent orchestration—state management, security, cost control, and tool execution—so they can focus on defining intelligent behaviors.

## Target Users
- **Rust Tool Builders:** Developers creating CLI tools, TUIs, or services that require embedded AI capabilities.
- **Platform Engineers:** Teams building internal developer platforms or specialized agents that need strict security and resource limits.
- **AI Researchers:** Users experimenting with multi-agent systems, subagents, and complex task delegation.

## Core Value Propositions
- **Complete Agent Runtime:** Out-of-the-box management for sessions, conversation history, and cost budgeting.
- **Safety & Control:** Granular permission systems, file checkpointing with "rewind" capabilities, and secure secret handling.
- **Flexible Extensibility:** Define tools in native Rust, attach local plugins, or connect external skills via MCP.
- **Observability:** Rich hook system for intercepting messages, detailed usage metrics, and streaming status updates.

## Key Features
- **Smart Execution:** Support for subagents, chain-of-thought "thinking" blocks, and structured output generation.
- **Multi-Transport Support:** Connect via Stdio for local processes, or HTTP/SSE for remote agent services.
- **Interactive Protocols:** Built-in handling for user approvals, slash commands, and streaming user input.
- **Developer Experience:** Type-safe APIs, extensive testing infrastructure (E2E, property-based), and idiomatic Rust patterns.

## Success Metrics
- **Integration Ease:** Developers can integrate full agentic capabilities into existing Rust binaries with minimal boilerplate.
- **Safety Compliance:** Agents respect defined budget limits and tool permission policies by default.
- **Performance:** Efficient handling of high-throughput streaming and large context windows.
