# Release Process

This document outlines the release process for the Claude Agent SDK (Rust).

## Versioning

We follow [Semantic Versioning 2.0.0](https://semver.org/).

## Pre-Release Checks

1.  **Test**: Ensure all tests pass.
    ```bash
    cargo test --workspace --all-features
    ./run_e2e.sh
    ```
2.  **Lint**: Ensure zero warnings.
    ```bash
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo fmt --all -- --check
    ```
3.  **Doc**: Verify documentation builds.
    ```bash
    cargo doc --workspace --no-deps
    ```

## Publishing

Currently, the crates are not published to crates.io. To release a new version locally or via git tags:

1.  Update version numbers in `Cargo.toml` files (workspace and crates).
2.  Update `CHANGELOG.md` (if maintained).
3.  Commit changes: `git commit -am "chore: release v0.1.0"`
4.  Tag release: `git tag v0.1.0`
5.  Push tags: `git push origin v0.1.0`
