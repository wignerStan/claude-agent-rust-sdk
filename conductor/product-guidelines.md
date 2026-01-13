# Product Guidelines - Claude Agent Rust SDK

## Code Style & Conventions
- **Idiomatic Rust:** Follow standard Rust naming conventions (snake_case for functions/variables, PascalCase for types). Prioritize `Result` for error handling and `Option` for optional values.
- **Type Safety:** Use strong typing to model agent states and tool definitions. Avoid `unwrap()` in production code; use proper error propagation.
- **Async First:** Design APIs to be asynchronous using `tokio` and `futures`. Ensure all I/O-bound operations are non-blocking.
- **Documentation:** All public APIs must be documented with rustdoc comments, including examples where complex usage is expected.

## API Design Principles
- **Builder Pattern:** Use the builder pattern for complex configurations (e.g., `ClaudeAgentOptions`) to maintain readability and extensibility.
- **Composition over Inheritance:** Encourage the composition of agents and tools rather than deep inheritance hierarchies.
- **Explicit Ownership:** Be clear about data ownership. Prefer borrowing (`&T`) where possible to minimize cloning, but use `Arc` for shared state when necessary in async contexts.

## Security & Safety
- **Secure Defaults:** Default configurations should be secure (e.g., `PermissionMode::Prompt` enabled by default).
- **Secret Management:** Use the `secrecy` crate for handling sensitive tokens and keys to prevent accidental logging.
- **Input Validation:** Validate all external inputs, especially those from untrusted sources (like LLM outputs or MCP servers), using `schemars` and strongly typed structs.

## Testing Strategy
- **Unit Tests:** Every module should have unit tests covering core logic.
- **Integration Tests:** Use the `tests/` directory for testing public APIs and component interactions.
- **Mocking:** Provide mock implementations for transports and CLI interactions to facilitate offline testing.

## Git Workflow
- **Conventional Commits:** Use conventional commit messages (e.g., `feat:`, `fix:`, `docs:`) to streamline changelog generation.
- **Atomic Commits:** Keep commits focused on a single logical change to simplify code review and reverts.
