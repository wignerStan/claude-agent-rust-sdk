# Testing Guide

This guide explains the testing infrastructure and methodologies used in the Claude Agent Rust SDK.

## Overview

The SDK follows a multi-layered testing strategy to ensure reliability, performance, and security:
1. **Unit Tests**: Testing individual modules and functions in isolation.
2. **Integration Tests**: Testing the interaction between different crates and external systems.
3. **Property-Based Testing**: Validating code against a wide range of automatically generated inputs.
4. **Benchmarks**: Measuring and tracking performance of critical paths.
5. **End-to-End (E2E) Tests**: Validating the full system flow against a live Claude CLI instance.

## Unit Testing

Unit tests are located within each crate, typically in the same file as the code or in a `tests` module.

```bash
# Run all unit tests in the workspace
cargo test --workspace
```

We use `MockTransport` in `crates/core` to test agent logic without spawning a real subprocess.

## Property-Based Testing

We use the `proptest` crate to perform property-based testing, which is particularly useful for verifying JSON parsing and state machine transitions.

See `crates/mcp/tests/proptest_mcp.rs` for examples.

```bash
# Run property tests
cargo test -p claude-agent-mcp --test proptest_mcp
```

## Performance Benchmarking

We use `criterion` for high-precision benchmarking. Benchmarks are located in `crates/mcp/benches/`.

```bash
# Run benchmarks
cargo bench -p claude-agent-mcp
```

## End-to-End (E2E) Testing

E2E tests validate the SDK's ability to communicate with a real Claude Code CLI. These tests require a valid `ANTHROPIC_AUTH_TOKEN`.

### Running E2E Tests

Use the provided script:
```bash
./run_e2e.sh
```

The script configures the environment (base URL, model, etc.) and runs the `e2e_live` test suite.

## Documentation (Doc) Tests

We ensure that all examples in the documentation are correct and compile.

```bash
# Run all doc tests
cargo test --doc --workspace
```

Note: Most doc tests are marked as `no_run` because they require an active connection or environment setup, but they are still checked for compilation errors.

## CI/CD and Security

Every pull request runs the full test suite, including:
- Unit and integration tests
- `cargo clippy` for linting
- `cargo fmt` for formatting
- `cargo-deny` for security audits (licenses, vulnerabilities, bans)

### Manual Security Audit
```bash
# Run security audit locally
cargo-deny check
```
