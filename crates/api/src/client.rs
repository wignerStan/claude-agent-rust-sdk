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
        Self { agent: ClaudeAgent::new(opts) }
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

    // TODO: Implement when core agent methods are available

    /// Rewind files to their state before a specific user message.
    ///
    /// This undoes all file modifications made since the given user message
    /// was sent, restoring the filesystem to its prior state.
    ///
    /// # Arguments
    ///
    /// * `user_message_id` - The ID of the user message to rewind to.
    pub async fn rewind_files(
        &self,
        _user_message_id: &str,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        // TODO: Implement when core agent methods are available
        Err(ClaudeAgentError::Transport("Not yet implemented: core support pending".into()))
    }

    /// Stop a running task.
    ///
    /// Sends a signal to stop the currently executing task identified by the
    /// given task ID.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The ID of the task to stop.
    pub async fn stop_task(&self, _task_id: &str) -> Result<ControlResponse, ClaudeAgentError> {
        // TODO: Implement when core agent methods are available
        Err(ClaudeAgentError::Transport("Not yet implemented: core support pending".into()))
    }

    /// Get the status of all configured MCP servers.
    ///
    /// Returns information about connected, disconnected, and errored MCP servers.
    pub async fn get_mcp_status(&self) -> Result<ControlResponse, ClaudeAgentError> {
        // TODO: Implement when core agent methods are available
        Err(ClaudeAgentError::Transport("Not yet implemented: core support pending".into()))
    }

    /// Reconnect a disconnected MCP server.
    ///
    /// Attempts to re-establish the connection to an MCP server that has
    /// been disconnected or has errored.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the MCP server to reconnect.
    pub async fn reconnect_mcp_server(
        &self,
        _name: &str,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        // TODO: Implement when core agent methods are available
        Err(ClaudeAgentError::Transport("Not yet implemented: core support pending".into()))
    }

    /// Enable or disable an MCP server.
    ///
    /// Toggles the enabled state of a named MCP server. When disabled, the
    /// server remains configured but will not be used for tool calls.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the MCP server.
    /// * `enabled` - Whether the server should be enabled.
    pub async fn toggle_mcp_server(
        &self,
        _name: &str,
        _enabled: bool,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        // TODO: Implement when core agent methods are available
        Err(ClaudeAgentError::Transport("Not yet implemented: core support pending".into()))
    }

    /// Get current context window usage.
    ///
    /// Returns information about how much of the context window has been
    /// consumed, including token counts and usage percentages.
    pub async fn get_context_usage(&self) -> Result<ControlResponse, ClaudeAgentError> {
        // TODO: Implement when core agent methods are available
        Err(ClaudeAgentError::Transport("Not yet implemented: core support pending".into()))
    }

    /// Get information about the Claude Code server.
    ///
    /// Returns server version, configuration, and capabilities information.
    pub async fn get_server_info(&self) -> Result<ControlResponse, ClaudeAgentError> {
        // TODO: Implement when core agent methods are available
        Err(ClaudeAgentError::Transport("Not yet implemented: core support pending".into()))
    }
}
