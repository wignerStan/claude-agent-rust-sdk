# Contributing to Claude Agent Rust SDK

We welcome contributions from the community! This document provides guidelines for contributing to the Claude Agent Rust SDK project.

## Code of Conduct

Please be respectful, inclusive, and constructive in all interactions. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for our community standards.

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates.

1. Search [existing issues](../../issues) for similar problems
2. Create a new issue with a clear, descriptive title
3. Provide detailed information about the bug:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Environment details (Rust version, OS, etc.)
4. Include relevant code snippets or error messages
5. Use appropriate labels (e.g., `bug`, `transport`, `mcp`)

### Suggesting Enhancements

We appreciate suggestions for improving the SDK:

1. Check [ROADMAP.md](ROADMAP.md) to see if your idea fits our plans
2. Open a discussion or issue with the `enhancement` label
3. Provide a clear description of the proposed feature
4. Explain the use case and benefits
5. Consider implementation complexity and trade-offs

### Pull Requests

We welcome pull requests for:

- **Bug fixes** with clear descriptions and tests
- **New features** that align with project goals
- **Documentation improvements**
- **Code refactoring** for better maintainability
- **Performance optimizations**

#### PR Guidelines

1. **Keep PRs focused and small** - Large PRs are harder to review
2. **Write clear commit messages** - Follow [conventional commits](https://www.conventionalcommits.org/)
3. **Add tests for new features** - Ensure test coverage doesn't decrease
4. **Update documentation** - If changing public APIs, update relevant docs
5. **Follow Rust best practices** - See [CONFIGURATION.md](CONFIGURATION.md)
6. **Pass CI checks** - Ensure `cargo test`, `cargo clippy`, and `cargo fmt` pass

### Development Setup

1. Fork the repository
2. Create a new branch for your feature: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run linter: `cargo clippy --all-targets`
6. Format code: `cargo fmt`
7. Commit your changes with a clear message
8. Push to your fork: `git push origin feature/my-feature`
9. Create a pull request on GitHub

### Testing

All contributions must include tests:

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test -p claude-agent-transport -- test_name
```

### Code Style

Follow the project's coding standards:

- Use `cargo fmt` to format code
- Use `cargo clippy` to check for issues
- Follow Rust naming conventions
- Add documentation for public APIs
- Write meaningful variable and function names
- Keep functions focused and small

### Commit Messages

Use [conventional commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no-op)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Scopes:**
- `transport`: Transport layer changes
- `mcp`: MCP server changes
- `core`: Core agent logic
- `api`: API client changes
- `types`: Type definitions
- `docs`: Documentation

**Examples:**
- `feat(transport): add timeout handling for connections`
- `fix(mcp): resolve resource leak in subprocess transport`
- `docs(core): add documentation for agent lifecycle`
- `refactor(api): improve error handling in HTTP client`

## Project Structure

Understanding the project structure is important for making effective contributions:

```
claude-agent-rust/
├── crates/
│   ├── types/           # Type definitions
│   ├── transport/       # Transport layer
│   ├── mcp/            # MCP server management
│   ├── core/           # Core agent logic
│   └── api/            # API client
├── demos/              # Example applications
├── CHANGELOG.md        # Change history
├── ROADMAP.md          # Development roadmap
├── CONFIGURATION.md    # Configuration documentation
├── LICENSE-MIT        # MIT license
├── LICENSE-APACHE     # Apache 2.0 license
├── CONTRIBUTING.md     # This file
└── CODE_OF_CONDUCT.md  # Community standards
```

## Getting Help

If you need help:

1. Check [existing issues](../../issues) - your question may already be answered
2. Read [documentation](README.md) - comprehensive project overview
3. Read [ROADMAP.md](ROADMAP.md) - planned features and milestones
4. Start a [discussion](../../discussions) - for questions and ideas

## License

By contributing to this project, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).

---

Thank you for contributing to Claude Agent Rust SDK! 🚀
