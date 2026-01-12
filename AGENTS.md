# Project Rules for AI and Humans

## General
- Do **not** introduce `unsafe` code unless explicitly asked and justified in comments.
- Avoid `.unwrap()` / `.expect()` in production paths.
- Prefer borrowing (`&T`) over cloning; only `.clone()` when necessary.
- Use `?` operator for error propagation instead of explicit match statements where appropriate.
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types).

## Error Handling
- Use `thiserror` for library error types with proper error variants.
- Use `anyhow::Result` only at the outermost application layer.
- Provide meaningful error messages with context using `.context()` from anyhow.
- Create custom error types for domain-specific errors.

## Async and Concurrency
- All async handlers must be `Send + Sync` where appropriate.
- Use `Arc` + `Mutex/RwLock` sparingly; prefer message passing or `&` references.
- Use `tokio` for async runtime.
- Be aware of blocking operations in async contexts; use `tokio::task::spawn_blocking` for CPU-bound work.
- Prefer `tokio::sync` primitives over `std::sync` in async contexts.

## Testing
- Every new non-trivial function must have unit tests.
- For public APIs, add at least one integration test per endpoint.
- Use `cargo test` to run tests.
- Consider using `cargo nextest` for faster test runs in large projects.
- Write tests that are deterministic and don't rely on external state.
- Use `#[tokio::test]` for async tests.

## Performance
- Prefer `Cow<str>` over `String` when borrowing is possible.
- Use `&[u8]` instead of `Vec<u8>` for read-only byte slices.
- Avoid unnecessary allocations in hot paths.
- Use iterators instead of loops when possible.
- Consider using `Box::leak` or `lazy_static`/`once_cell` for static data.

## Code Organization
- Keep modules focused and small (aim for < 300 lines per module).
- Use `pub(crate)` for internal APIs that shouldn't be exposed.
- Organize code by feature/domain, not by type.
- Use workspace dependencies when working with multiple crates.

## Documentation
- Document all public APIs with `///` doc comments.
- Include examples in doc comments where appropriate.
- Use `#[doc(hidden)]` for internal implementation details.
- Keep documentation up-to-date with code changes.

## AI Usage
- You (AI) are a Senior Rust Engineer.
- Always ensure generated code:
  - Compiles without errors
  - Is idiomatic Rust
  - Respects the rules in this file
  - Passes clippy with no warnings
- Before writing code, briefly outline your plan in comments.
- When in doubt, ask for clarification rather than making assumptions.

## Security
- Never log sensitive information (passwords, tokens, secrets).
- Use `secrecy` crate for handling secret values.
- Validate all external input.
- Use constant-time comparison for sensitive data (e.g., passwords).
- Keep dependencies up-to-date and audit them regularly.

## Git Workflow
- Write meaningful commit messages following conventional commits.
- Keep commits atomic and focused.
- Run `cargo fmt`, `cargo clippy`, and `cargo test` before committing. - Ensure CI passes before merging.
