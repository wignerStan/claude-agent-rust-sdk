# Roadmap

This document outlines the planned development direction for the Claude Agent Rust SDK.

## Overview

The Claude Agent Rust SDK provides a comprehensive Rust implementation for interacting with Claude Code CLI. This roadmap outlines our priorities for future development, focusing on stability, performance, security, and developer experience.

## Current Status: v0.1.0 (2026-01-13)

The SDK is in early development with core functionality implemented:
- ✅ Core agent implementation
- ✅ Transport layer (subprocess)
- ✅ MCP integration (stdio transport only)
- ✅ Session management
- ✅ Hook system
- ✅ Permission handling
- ✅ Control protocol
- ✅ Streaming support
- ✅ Comprehensive documentation
- ✅ Example applications

## Short-term Priorities (Q1 2026)

### 1. Complete MCP Transport Implementations

**Priority**: High | **Status**: In Progress

The SDK currently only supports stdio-based MCP servers. We need to complete the other transport options:

- [ ] Implement `HttpMcpServer` for HTTP-based MCP servers
- [ ] Implement `SseMcpServer` for Server-Sent Events transport
- [ ] Add transport selection logic based on configuration
- [ ] Document transport-specific use cases

**Rationale**: HTTP and SSE transports are essential for:
- Cloud-based MCP services
- Web-based integrations
- Serverless deployments
- Multi-tenant architectures

### 2. Security Enhancements

**Priority**: High | **Status**: Pending

Implement security best practices for production deployments:

- [ ] Integrate `secrecy` crate for sensitive data handling
- [ ] Add `cargo-deny` to CI/CD pipeline
- [ ] Implement input validation and schema checking for JSON parsing
- [ ] Add constant-time comparison for password/token handling
- [ ] Security audit of subprocess command execution
- [ ] Add rate limiting for MCP tool calls

**Rationale**: Security is critical for production use, especially when handling:
- API keys and tokens
- User credentials
- File system operations
- External process execution

### 3. Testing Infrastructure

**Priority**: High | **Status**: Pending

Improve test coverage and add specialized testing:

- [ ] Add test coverage metrics (e.g., `cargo-tarpaulin`)
- [ ] Implement performance benchmarks
- [ ] Add concurrency and race condition tests
- [ ] Create end-to-end integration tests with real CLI
- [ ] Add fuzzing for JSON parsing
- [ ] Implement property-based testing (e.g., `proptest`)

**Rationale**: Comprehensive testing ensures:
- Code quality and reliability
- Performance regression detection
- Thread safety guarantees
- Production readiness

### 4. Documentation Completion

**Priority**: Medium | **Status**: Pending

Complete missing documentation:

- [ ] Finish `MIGRATION_PROGRESS.md` guide
- [ ] Create architecture diagrams
- [ ] Add troubleshooting guide
- [ ] Document performance characteristics
- [ ] Add API reference documentation
- [ ] Create video tutorials

**Rationale**: Good documentation improves:
- Developer onboarding
- Adoption rates
- Support burden reduction
- Community contributions

## Medium-term Priorities (Q2-Q3 2026)

### 5. Performance Optimization

**Priority**: Medium | **Status**: Pending

Optimize hot paths for better performance:

- [ ] Reduce excessive cloning in session management
- [ ] Implement zero-copy message passing where possible
- [ ] Optimize JSON parsing with `serde_json::from_slice`
- [ ] Add connection pooling for MCP servers
- [ ] Implement backpressure handling for channels
- [ ] Profile and optimize memory usage

**Rationale**: Performance improvements enable:
- Higher throughput
- Lower latency
- Reduced resource usage
- Better scalability

### 6. Error Handling Improvements

**Priority**: Medium | **Status**: Pending

Enhance error handling throughout the codebase:

- [ ] Add structured error context with `anyhow`
- [ ] Implement error recovery strategies
- [ ] Add retry logic for transient failures
- [ ] Create error classification (retryable vs fatal)
- [ ] Add error metrics and monitoring

**Rationale**: Better error handling provides:
- Improved debugging experience
- Better resilience
- Clearer error messages
- Easier troubleshooting

### 7. Advanced Features

**Priority**: Medium | **Status**: Pending

Add advanced capabilities:

- [ ] Implement session persistence and recovery
- [ ] Add distributed session support
- [ ] Implement tool caching and memoization
- [ ] Add streaming tool execution
- [ ] Support for custom message formats
- [ ] Implement plugin system for extensions

**Rationale**: Advanced features enable:
- More sophisticated use cases
- Better performance through caching
- Extensibility without core changes
- Enterprise-grade capabilities

### 8. Developer Experience

**Priority**: Medium | **Status**: Pending

Improve developer experience:

- [ ] Add `derive` macros for common patterns
- [ ] Create CLI tool for SDK management
- [ ] Add IDE integration (VS Code, IntelliJ)
- [ ] Implement hot-reload for development
- [ ] Add interactive examples
- [ ] Create SDK generator for custom agents

**Rationale**: Better DX leads to:
- Faster development cycles
- Higher adoption
- More contributions
- Better community engagement

## Long-term Vision (Q4 2026+)

### 9. Production Readiness

**Priority**: High | **Status**: Pending

Ensure SDK is production-ready:

- [ ] SLA guarantees and benchmarks
- [ ] Comprehensive monitoring and observability
- [ ] Disaster recovery procedures
- [ ] Multi-region deployment support
- [ ] Blue-green deployment strategies
- [ ] Automated rollback capabilities

### 10. Ecosystem Integration

**Priority**: Medium | **Status**: Pending

Integrate with broader ecosystem:

- [ ] WASM support for browser environments
- [ ] Python bindings via PyO3
- [ ] JavaScript/TypeScript bindings via wasm-bindgen
- [ ] GraphQL API layer
- [ ] gRPC transport option
- [ ] Kubernetes operator for deployment

### 11. AI/ML Enhancements

**Priority**: Low | **Status**: Pending

Explore AI/ML capabilities:

- [ ] Add token usage tracking and optimization
- [ ] Implement intelligent caching strategies
- [ ] Add A/B testing for prompts
- [ ] Implement prompt engineering tools
- [ ] Add model selection heuristics
- [ ] Support for fine-tuned models

### 12. Community and Governance

**Priority**: Low | **Status**: Pending

Build a sustainable community:

- [ ] Establish RFC process for major changes
- [ ] Create technical steering committee
- [ ] Implement contribution recognition program
- [ ] Add security bounty program
- [ ] Create mentorship program
- [ ] Establish regular release cadence

## Deprecated Features

The following features are deprecated and will be removed in future releases:

- `HttpMcpServer` stub (will be fully implemented or removed in v0.2.0)
- `SseMcpServer` stub (will be fully implemented or removed in v0.2.0)
- Direct CLI path manipulation (use `ClaudeAgentOptions` instead)

## Versioning Strategy

We follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html):

- **Major version (X.0.0)**: Breaking changes
- **Minor version (0.X.0)**: New features, backwards compatible
- **Patch version (0.0.X)**: Bug fixes, backwards compatible

## Release Cadence

Target release schedule:
- **Major releases**: Every 6 months
- **Minor releases**: Every 2-3 months
- **Patch releases**: As needed (bug fixes, security updates)

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

For roadmap discussions:
- Open an issue with the `roadmap` label
- Join our Discord/Slack community
- Attend our weekly community calls

## Questions?

If you have questions about the roadmap or want to discuss priorities:
- Open a GitHub issue
- Start a discussion in GitHub Discussions
- Contact the maintainers

---

*Last updated: 2026-01-13*
