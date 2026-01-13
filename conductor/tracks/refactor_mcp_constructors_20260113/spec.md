# Specification: Refactor MCP Constructors to Return Result

## Overview
Currently, the constructors for MCP transport servers (`SseMcpServer`, `HttpMcpServer`, and `StdioMcpServer`) use `.expect()` or `.unwrap()` during initialization (e.g., when building HTTP clients or spawning processes). This can cause the application to panic if initialization fails. This track refactors these constructors to return a `Result<Self, ClaudeAgentError>` for better error handling and application stability.

## Functional Requirements
1.  **Error Type Update**: Add a new `Initialization(String)` variant to the `ClaudeAgentError` enum in `crates/types/src/error.rs` to specifically represent startup failures.
2.  **StdioMcpServer Refactor**: Update the constructor logic to return `Result`. (Note: `StdioMcpServer::new` is currently infallible but `register_tool` handles spawning; we will ensure the pattern is consistent across the crate).
3.  **SseMcpServer Refactor**: Update `new()` and `with_timeout()` to return `Result<Self, ClaudeAgentError>`, replacing `.expect("Failed to build HTTP client")`.
4.  **HttpMcpServer Refactor**: Update `new()` and `with_timeout()` to return `Result<Self, ClaudeAgentError>`, replacing `.expect("Failed to build HTTP client")`.
5.  **API Migration**: This is a breaking change. All internal call sites, examples, and tests must be updated to handle the `Result`.

## Non-Functional Requirements
- **Consistency**: All MCP transport constructors should follow the same `Result`-based pattern.
- **Robustness**: Prevent process crashes during initialization due to environment issues (e.g., TLS initialization failure, missing binaries).

## Acceptance Criteria
- [ ] `ClaudeAgentError` has an `Initialization` variant.
- [ ] `SseMcpServer::new` and `with_timeout` return `Result`.
- [ ] `HttpMcpServer::new` and `with_timeout` return `Result`.
- [ ] All tests pass: Run `cargo test --workspace`.
- [ ] End-to-End tests pass: Run `./run_e2e.sh`.
- [ ] All examples in `crates/api/examples/` compile and run correctly (manual verification or script).
- [ ] No new `unwrap()` or `expect()` calls introduced in initialization paths.

## Out of Scope
- Changing the async nature of the transports.
- Refactoring the `McpServer` trait itself (unless necessary for the return types).
