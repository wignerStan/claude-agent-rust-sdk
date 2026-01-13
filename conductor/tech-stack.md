# Technology Stack - Claude Agent Rust SDK

## Core Language & Runtime
- **Language:** Rust (Edition 2021)
- **Async Runtime:** `tokio` (v1.35, full features)
- **Async Utilities:** `futures`, `async-trait`, `async-stream`, `tokio-util`, `tokio-stream`, and `pin-project-lite`.

## Data Handling & Protocols
- **Serialization/Deserialization:** `serde` (with derive) and `serde_json`.
- **MCP Implementation:** `rmcp` (v0.8.0) for Model Context Protocol server/client logic.
- **Schema Generation:** `schemars` (v0.8) for JSON Schema integration.
- **Protocols:** JSON-RPC 2.0 (via `rmcp` and custom transport).

## Infrastructure & Utilities
- **Error Handling:** `thiserror` for library-level errors and `anyhow` for application-level context.
- **Observability:** `tracing` and `tracing-subscriber` (env-filter, fmt).
- **ID Generation:** `uuid` (v1.6, v4 with serde support).
- **Environment:** `which` for locating system binaries.
- **Traffic Control:** `governor` for rate limiting.
- **Networking:** `reqwest` (v0.11 with JSON support) for HTTP/SSE transports.

## Security
- **Secret Handling:** `secrecy` (v0.8 with serde support).
- **Cryptography:** `subtle` (v2.5) for constant-time comparisons.

## Testing & Quality Assurance
- **Property-Based Testing:** `proptest`.
- **Benchmarking:** `criterion`.
- **Linting & Formatting:** `clippy` and `rustfmt`.
- **Security Auditing:** `cargo-deny`.
