//! Public API for Claude Agent SDK.
//!
//! This crate provides the high-level public interface for interacting with Claude Code.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use claude_agent_api::{query, ClaudeAgentClient};
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Simple one-shot query
//!     let mut stream = query("What is 2+2?", None).await.unwrap();
//!     while let Some(msg) = stream.next().await {
//!         println!("{:?}", msg);
//!     }
//!
//!     // Interactive client
//!     let mut client = ClaudeAgentClient::new(None);
//!     client.connect().await.unwrap();
//!     // ... interact with client
//!     client.disconnect().await.unwrap();
//! }
//! ```

pub mod client;
pub mod query;

// Re-export public types
pub use claude_agent_types as types;
pub use claude_agent_types::{ClaudeAgentError, ClaudeAgentOptions, Message};

// Re-export public API
pub use client::ClaudeAgentClient;
pub use query::query;
