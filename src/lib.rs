//! Claude Agent SDK for Rust.
//!
//! A Rust SDK for interacting with Claude Code via subprocess transport.

pub mod api;
pub mod core;
#[cfg(feature = "mcp")]
pub mod mcp;
pub mod transport;
pub mod types;

// Convenience re-exports
pub use api::client::ClaudeAgentClient;
pub use api::query::query;
pub use api::sessions::{find_claude_cli, SessionInfo};
pub use core::agent::ClaudeAgent;
pub use types::config::ClaudeAgentOptions;
pub use types::error::ClaudeAgentError;
pub use types::message::Message;
