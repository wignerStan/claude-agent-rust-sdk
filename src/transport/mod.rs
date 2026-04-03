//! Transport layer for Claude Agent SDK.

pub mod parser;
pub mod reader;
pub mod subprocess;

use crate::types::ClaudeAgentError;
use async_trait::async_trait;
use futures::stream::BoxStream;

pub use subprocess::SubprocessTransport;

/// Transport trait for communication with Claude Code.
#[async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&mut self) -> Result<(), ClaudeAgentError>;
    async fn write(&self, data: &str) -> Result<(), ClaudeAgentError>;
    async fn read_messages(&self) -> BoxStream<'_, Result<serde_json::Value, ClaudeAgentError>>;
    async fn close(&mut self) -> Result<(), ClaudeAgentError>;
}
