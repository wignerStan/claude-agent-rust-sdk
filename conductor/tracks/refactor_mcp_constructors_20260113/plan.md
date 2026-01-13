# Implementation Plan: Refactor MCP Constructors to Return Result

## Phase 1: Core Type Update [checkpoint: 14fcecd]
- [x] Task: Add `Initialization(String)` variant to `ClaudeAgentError` in `crates/types/src/error.rs` 8143299
- [x] Task: Conductor - User Manual Verification 'Core Type Update' (Protocol in workflow.md)

## Phase 2: SseMcpServer Refactor [checkpoint: d554812]
- [x] Task: Write failing tests for `SseMcpServer` initialization failures (e.g., invalid timeout/config simulation if possible) fc29e72
- [x] Task: Refactor `SseMcpServer::new` and `SseMcpServer::with_timeout` to return `Result<Self, ClaudeAgentError>` fc29e72
- [x] Task: Update call sites in `crates/mcp/src/transport_factory.rs` and tests fc29e72
- [x] Task: Conductor - User Manual Verification 'SseMcpServer Refactor' (Protocol in workflow.md)

## Phase 3: HttpMcpServer Refactor
- [x] Task: Write failing tests for `HttpMcpServer` initialization failures 1dfb916
- [x] Task: Refactor `HttpMcpServer::new` and `HttpMcpServer::with_timeout` to return `Result<Self, ClaudeAgentError>` 1dfb916
- [x] Task: Update call sites in `crates/mcp/src/transport_factory.rs` and tests 1dfb916
- [ ] Task: Conductor - User Manual Verification 'HttpMcpServer Refactor' (Protocol in workflow.md)

## Phase 4: StdioMcpServer Consistency
- [ ] Task: Update `StdioMcpServer::new` to return `Result<Self, ClaudeAgentError>` for API consistency
- [ ] Task: Update all call sites in `crates/mcp/` tests and examples
- [ ] Task: Conductor - User Manual Verification 'StdioMcpServer Consistency' (Protocol in workflow.md)

## Phase 5: API and Example Migration
- [ ] Task: Update `crates/api/src/client.rs` and `query.rs` to handle `Result` from MCP server creation
- [ ] Task: Update all examples in `crates/api/examples/`
- [ ] Task: Update integration and E2E tests in `crates/api/tests/`
- [ ] Task: Conductor - User Manual Verification 'API and Example Migration' (Protocol in workflow.md)

## Phase 6: Final Verification
- [ ] Task: Run full test suite: `cargo test --workspace`
- [ ] Task: Run E2E live tests: `./run_e2e.sh`
- [ ] Task: Verify all examples compile: `cargo build --examples`
- [ ] Task: Conductor - User Manual Verification 'Final Verification' (Protocol in workflow.md)
