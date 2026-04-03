//! Interactive client for bidirectional conversations.

use futures::stream::BoxStream;

use crate::core::{ClaudeAgent, ControlResponse};
use crate::types::{ClaudeAgentError, ClaudeAgentOptions, Message};

/// Client for bidirectional, interactive conversations with Claude Code.
///
/// This client provides full control over the conversation flow with support
/// for interrupts, permission mode changes, and model switching.
///
/// # Example
///
/// ```rust,no_run
/// use claude_agent::api::ClaudeAgentClient;
/// use claude_agent::types::ClaudeAgentOptions;
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
    pub fn set_transport(&mut self, transport: Box<dyn crate::transport::Transport>) {
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

    /// Wrap a serializable value into a successful `ControlResponse`.
    fn wrap_success(data: serde_json::Value) -> ControlResponse {
        ControlResponse {
            request_id: String::new(),
            success: true,
            response: Some(data),
            error: None,
        }
    }

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
        user_message_id: &str,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        self.agent.rewind_files(user_message_id).await
    }

    /// Stop a running task.
    ///
    /// Sends a signal to stop the currently executing task identified by the
    /// given task ID.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The ID of the task to stop.
    pub async fn stop_task(&self, task_id: &str) -> Result<ControlResponse, ClaudeAgentError> {
        self.agent.stop_task(task_id).await
    }

    /// Get the status of all configured MCP servers.
    ///
    /// Returns information about connected, disconnected, and errored MCP servers.
    pub async fn get_mcp_status(&self) -> Result<ControlResponse, ClaudeAgentError> {
        let response = self.agent.get_mcp_status().await?;
        Ok(Self::wrap_success(serde_json::to_value(&response).unwrap_or_default()))
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
        name: &str,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        self.agent.reconnect_mcp_server(name).await
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
        name: &str,
        enabled: bool,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        self.agent.toggle_mcp_server(name, enabled).await
    }

    /// Get current context window usage.
    ///
    /// Returns information about how much of the context window has been
    /// consumed, including token counts and usage percentages.
    pub async fn get_context_usage(&self) -> Result<ControlResponse, ClaudeAgentError> {
        let response = self.agent.get_context_usage().await?;
        Ok(Self::wrap_success(serde_json::to_value(&response).unwrap_or_default()))
    }

    /// Get information about the Claude Code server.
    ///
    /// Returns server version, configuration, and capabilities information.
    /// Returns an error if the agent has not connected or the server has not
    /// sent initialization data.
    pub async fn get_server_info(&self) -> Result<ControlResponse, ClaudeAgentError> {
        match self.agent.get_server_info().await {
            Some(info) => Ok(Self::wrap_success(serde_json::to_value(&info).unwrap_or_default())),
            None => {
                Err(ClaudeAgentError::Transport("No server info available: not connected".into()))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::config::PermissionMode;

    use crate::transport::Transport;
    use async_trait::async_trait;
    use futures::stream::{self, BoxStream};

    struct MockTransport {
        responses: Vec<serde_json::Value>,
    }

    impl MockTransport {
        fn new(responses: Vec<serde_json::Value>) -> Self {
            Self { responses }
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn connect(&mut self) -> Result<(), ClaudeAgentError> {
            Ok(())
        }
        async fn write(&self, _data: &str) -> Result<(), ClaudeAgentError> {
            Ok(())
        }
        async fn read_messages(
            &self,
        ) -> BoxStream<'_, Result<serde_json::Value, ClaudeAgentError>> {
            let responses = self.responses.clone();
            Box::pin(stream::iter(responses.into_iter().map(Ok)))
        }
        async fn close(&mut self) -> Result<(), ClaudeAgentError> {
            Ok(())
        }
    }

    /// A mock transport that auto-replies to control requests.
    /// `response_body` is included as the `"response"` field in every reply.
    struct ControlReplyTransport {
        tx: tokio::sync::broadcast::Sender<Result<serde_json::Value, ClaudeAgentError>>,
        response_body: serde_json::Value,
    }

    #[async_trait]
    impl Transport for ControlReplyTransport {
        async fn connect(&mut self) -> Result<(), ClaudeAgentError> {
            Ok(())
        }
        async fn write(&self, data: &str) -> Result<(), ClaudeAgentError> {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                if val.get("type").and_then(|s| s.as_str()) == Some("control_request") {
                    let req_id = val.get("request_id").and_then(|s| s.as_str()).unwrap_or("");
                    let resp = serde_json::json!({
                        "type": "control_response",
                        "request_id": req_id,
                        "success": true,
                        "response": self.response_body
                    });
                    let _ = self.tx.send(Ok(resp));
                }
            }
            Ok(())
        }
        async fn read_messages(
            &self,
        ) -> BoxStream<'_, Result<serde_json::Value, ClaudeAgentError>> {
            let rx = self.tx.subscribe();
            Box::pin(futures::stream::unfold(rx, |mut rx| async move {
                loop {
                    match rx.recv().await {
                        Ok(res) => return Some((res, rx)),
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                        Err(_) => continue,
                    }
                }
            }))
        }
        async fn close(&mut self) -> Result<(), ClaudeAgentError> {
            Ok(())
        }
    }

    // --- Construction tests ---

    #[test]
    fn new_with_none_options_uses_defaults() {
        let client = ClaudeAgentClient::new(None);
        assert!(client.session_id().is_none());
    }

    #[test]
    fn new_with_some_default_options() {
        let opts = ClaudeAgentOptions::default();
        let client = ClaudeAgentClient::new(Some(opts));
        assert!(client.session_id().is_none());
    }

    #[test]
    fn new_with_custom_options() {
        let opts = ClaudeAgentOptions {
            permission_mode: Some(PermissionMode::Plan),
            max_turns: Some(10),
            model: Some("claude-sonnet-4-5".to_string()),
            ..Default::default()
        };
        let client = ClaudeAgentClient::new(Some(opts));
        assert!(client.session_id().is_none());
    }

    // --- Connect / disconnect tests ---

    #[tokio::test]
    async fn connect_with_mock_transport_sets_session_id() {
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(MockTransport::new(vec![])));
        assert!(client.session_id().is_none());
        client.connect().await.unwrap();
        assert!(client.session_id().is_some());
    }

    #[tokio::test]
    async fn disconnect_after_connect_succeeds() {
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(MockTransport::new(vec![])));
        client.connect().await.unwrap();
        assert!(client.disconnect().await.is_ok());
    }

    #[tokio::test]
    async fn disconnect_without_connect_is_ok() {
        let mut client = ClaudeAgentClient::new(None);
        assert!(client.disconnect().await.is_ok());
    }

    #[tokio::test]
    async fn disconnect_is_idempotent() {
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(MockTransport::new(vec![])));
        client.connect().await.unwrap();
        assert!(client.disconnect().await.is_ok());
        assert!(client.disconnect().await.is_ok());
    }

    // --- Query tests ---

    #[tokio::test]
    async fn query_with_mock_transport_yields_messages() {
        use futures::StreamExt;
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "role": "assistant",
                "content": [{"type": "text", "text": "hello"}],
                "model": "test"
            }
        });
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(MockTransport::new(vec![response])));
        let mut stream = client.query("hi").await.unwrap();
        let mut count = 0;
        while let Some(msg) = stream.next().await {
            assert!(msg.is_ok());
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn query_empty_stream() {
        use futures::StreamExt;
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(MockTransport::new(vec![])));
        let mut stream = client.query("hi").await.unwrap();
        let mut count = 0;
        while let Some(msg) = stream.next().await {
            assert!(msg.is_ok());
            count += 1;
        }
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn query_auto_connects_when_not_connected() {
        use futures::StreamExt;
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "role": "assistant",
                "content": [{"type": "text", "text": "auto"}],
                "model": "test"
            }
        });
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(MockTransport::new(vec![response])));
        // Not explicitly connected -- query should auto-connect
        let mut stream = client.query("auto").await.unwrap();
        let mut count = 0;
        while let Some(msg) = stream.next().await {
            assert!(msg.is_ok());
            count += 1;
        }
        assert_eq!(count, 1);
    }

    // --- Control method tests ---

    #[tokio::test]
    async fn control_methods_fail_without_connected_transport() {
        // Control protocol is always initialized, but the background task
        // that handles responses only runs after connect(). Without it,
        // the oneshot response channel is never fulfilled — we test with
        // a mock transport that auto-replies to control requests.
        let (tx, _) = tokio::sync::broadcast::channel(100);
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(ControlReplyTransport {
            tx,
            response_body: serde_json::json!({}),
        }));
        client.connect().await.unwrap();

        // These should succeed with the auto-replying transport
        assert!(client.interrupt().await.is_ok());
        assert!(client.set_permission_mode("plan").await.is_ok());
        assert!(client.set_model(Some("claude-sonnet-4-5")).await.is_ok());
    }

    #[tokio::test]
    async fn agent_methods_work_with_connected_control_reply_transport() {
        // Use a transport that auto-replies to control requests to test
        // the agent methods that delegate to the core agent.
        let (tx, _) = tokio::sync::broadcast::channel(100);
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(ControlReplyTransport {
            tx,
            response_body: serde_json::json!({}),
        }));
        client.connect().await.unwrap();

        // These delegate to the core agent's control protocol
        assert!(client.rewind_files("msg-id").await.is_ok());
        assert!(client.stop_task("task-id").await.is_ok());
        assert!(client.reconnect_mcp_server("srv").await.is_ok());
        assert!(client.toggle_mcp_server("srv", true).await.is_ok());
    }

    #[tokio::test]
    async fn mcp_status_and_context_with_connected_transport() {
        let (tx, _) = tokio::sync::broadcast::channel(100);
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(ControlReplyTransport {
            tx,
            response_body: serde_json::json!({"mcpServers": []}),
        }));
        client.connect().await.unwrap();

        // get_mcp_status wraps the agent result in a ControlResponse
        assert!(client.get_mcp_status().await.is_ok());

        // get_context_usage wraps the agent result in a ControlResponse
        assert!(client.get_context_usage().await.is_ok());
    }

    #[tokio::test]
    async fn get_server_info_returns_error_when_not_connected() {
        let client = ClaudeAgentClient::new(None);
        let err = client.get_server_info().await.unwrap_err();
        assert!(err.to_string().contains("No server info available"));
    }
}
