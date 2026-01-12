# Justfile for Claude Agent Rust SDK

# A comprehensive task runner for common development operations.

## Development Tasks

### Build
build:
    cargo build --workspace

### Test
test:
    cargo test --workspace

### Test with Output
test-verbose:
    cargo test --workspace -- --nocapture

### Test Specific Crate
test-core:
    cargo test -p claude-agent-core

test-transport:
    cargo test -p claude-agent-transport

test-mcp:
    cargo test -p claude-agent-mcp

### Format
fmt:
    cargo fmt --all

### Lint
lint:
    cargo clippy --workspace --all-targets

### Check
check:
    cargo check --workspace

### Clean
clean:
    cargo clean --workspace

### Clean All
clean-all:
    cargo clean --workspace && rm -rf target

### Doc
doc:
    cargo doc --workspace --no-deps

### Doc Open
doc-open:
    cargo doc --workspace --open

### Run
run:
    cargo run --bin <name>

### Release
release-patch:
    cargo release --dry-run

release-minor:
    cargo release --dry-run --level minor

release-major:
    cargo release --dry-run --level major

## Quality Tasks

### All Quality Checks
qa:
    fmt && lint && check && test

### Format Check
qa-fmt:
    cargo fmt --all -- --check

### Lint Check
qa-lint:
    cargo clippy --workspace --all-targets -- -D warnings

### Test Check
qa-test:
    cargo test --workspace

## Workspace Operations

### List Members
members:
    cargo tree --workspace

### Update Dependencies
update:
    cargo update --workspace

### Outdated Dependencies
outdated:
    cargo outdated --workspace

### Audit Dependencies
audit:
    cargo audit --workspace

### Check Security
check-security:
    cargo-deny check bans sources licenses

## CI/CD Tasks

### CI Build
ci-build:
    cargo build --workspace --release

### CI Test
ci-test:
    cargo test --workspace

### CI Lint
ci-lint:
    cargo clippy --workspace --all-targets -- -D warnings

### CI Format
ci-fmt:
    cargo fmt --all -- --check

## Release Tasks

### Prepare Release
release-prepare:
    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo doc --workspace --no-deps

### Release
release:
    cargo release

## Utility Tasks

### Install Tools
install-tools:
    cargo install cargo-watch cargo-edit cargo-outdated

### Generate Lockfile
lockfile:
    cargo generate-lockfile

### Update Lockfile
update-lockfile:
    cargo update --workspace

## Documentation Tasks

### Serve Documentation
doc-serve:
    cargo doc --workspace --open --no-deps

### Examples
example-hello:
    cargo run --example hello_world

example-chat:
    cargo run --example simple_chat

## Environment Variables

### Set Development Environment
dev-env:
    export RUST_LOG=debug
    export RUST_BACKTRACE=1

### Set Production Environment
prod-env:
    export RUST_LOG=info
    unset RUST_BACKTRACE

### Show Environment
env:
    echo "RUST_LOG=${RUST_LOG:-not set}"
    echo "RUST_BACKTRACE=${RUST_BACKTRACE:-not set}"

## Help
help:
    @just --list

## Default Task
default:
    @just --list

## Notes

# All tasks are workspace-aware and operate on the entire workspace
# Use `just <task> --help` to see task-specific help
# Tasks can be chained: `just fmt && lint && test`
# For more information, see: https://just.systems/man/en/
