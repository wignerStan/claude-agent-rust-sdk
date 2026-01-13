//! Interactive client for bidirectional conversations.

use futures::stream::BoxStream;

use claude_agent_core::{ClaudeAgent, ControlResponse};
use claude_agent_types::{ClaudeAgentError, ClaudeAgentOptions, Message};

/// Client for bidirectional, interactive conversations with Claude Code.
///
/// This client provides full control over the conversation flow with support
/// for interrupts, permission mode changes, and model switching.
///
/// # Example
///
/// ```rust,no_run
/// use claude_agent_api::ClaudeAgentClient;
/// use claude_agent_types::ClaudeAgentOptions;
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut client = ClaudeAgentClient::new(None);
///     client.connect().await.unwrap();
///
///     let mut stream = client.query("Hello, Claude!").await.unwrap();
///     while let Some(result) = stream.next().await {
///         println!("{:?}", result);
///     }
///
///     // Explicitly drop stream to release mutable borrow on client
///     drop(stream);
///     client.disconnect().await?;
///     Ok(())
/// }
/// ```
pub struct ClaudeAgentClient {
    agent: ClaudeAgent,
}

impl ClaudeAgentClient {
    /// Create a new Claude Agent client.
    pub fn new(options: Option<ClaudeAgentOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            agent: ClaudeAgent::new(opts),
        }
    }

    /// Set the transport implementation.
    ///
    /// Useful for testing with mock transports or using custom transport implementations.
    pub fn set_transport(&mut self, transport: Box<dyn claude_agent_transport::Transport>) {
        self.agent.set_transport(transport);
    }

    /// Connect to Claude Code.
    pub async fn connect(&mut self) -> Result<(), ClaudeAgentError> {
        self.agent.connect(None).await
    }

    /// Send a query and receive a stream of messages.
    pub async fn query(
        &mut self,
        prompt: &str,
    ) -> Result<BoxStream<'_, Result<Message, ClaudeAgentError>>, ClaudeAgentError> {
        self.agent.query(prompt).await
    }

    /// Send interrupt signal.
    pub async fn interrupt(&self) -> Result<ControlResponse, ClaudeAgentError> {
        self.agent.interrupt().await
    }

    /// Set permission mode.
    ///
    /// Valid modes:
    /// - "default": CLI prompts for dangerous tools
    /// - "acceptEdits": Auto-accept file edits
    /// - "plan": Planning mode
    /// - "bypassPermissions": Allow all tools (use with caution)
    pub async fn set_permission_mode(
        &self,
        mode: &str,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        self.agent.set_permission_mode(mode).await
    }

    /// Set the AI model.
    ///
    /// Examples:
    /// - "claude-sonnet-4-5"
    /// - "claude-opus-4-1-20250805"
    pub async fn set_model(
        &self,
        model: Option<&str>,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        self.agent.set_model(model).await
    }

    /// Disconnect from Claude Code.
    pub async fn disconnect(&mut self) -> Result<(), ClaudeAgentError> {
        self.agent.disconnect().await
    }

    /// Get the current session ID.
    pub fn session_id(&self) -> Option<&str> {
        self.agent.current_session().map(|s| s.id.as_str())
    }
}
