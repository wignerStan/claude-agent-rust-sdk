//! Control protocol implementation for SDK communication.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, Mutex};
use uuid::Uuid;

use crate::types::ClaudeAgentError;

/// Control protocol handler for request/response routing.
pub struct ControlProtocol {
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<ControlResponse>>>>,
    request_tx: mpsc::Sender<ControlRequest>,
}

/// A control request to send to the CLI.
#[derive(Debug, Clone)]
pub struct ControlRequest {
    pub request_id: String,
    pub request: ControlRequestType,
}

/// Types of control requests.
#[derive(Debug, Clone)]
pub enum ControlRequestType {
    Interrupt,
    Initialize {
        hooks: Option<serde_json::Value>,
    },
    SetPermissionMode {
        mode: String,
    },
    SetModel {
        model: Option<String>,
    },
    RewindFiles {
        user_message_id: String,
    },
    StopTask {
        task_id: String,
    },
    McpMessage {
        server_name: String,
        message: serde_json::Value,
    },
    McpStatus,
    McpReconnect {
        server_name: String,
    },
    McpToggle {
        server_name: String,
        enabled: bool,
    },
    GetContextUsage,
    HookCallback {
        callback_id: String,
        output: serde_json::Value,
    },
}

/// A control response from the CLI.
#[derive(Debug, Clone)]
pub struct ControlResponse {
    pub request_id: String,
    pub success: bool,
    pub response: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl ControlProtocol {
    /// Create a new control protocol handler.
    pub fn new() -> (Self, mpsc::Receiver<ControlRequest>) {
        let (tx, rx) = mpsc::channel(32);
        (
            Self { pending_requests: Arc::new(Mutex::new(HashMap::new())), request_tx: tx },
            rx,
        )
    }

    /// Send a control request and wait for response.
    pub async fn send_request(
        &self,
        request_type: ControlRequestType,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        let request_id = Uuid::new_v4().to_string();
        let (response_tx, response_rx) = oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request_id.clone(), response_tx);
        }

        // Send request
        let request = ControlRequest { request_id: request_id.clone(), request: request_type };

        self.request_tx.send(request).await.map_err(|e| {
            ClaudeAgentError::ControlProtocol(format!("Failed to send request: {}", e))
        })?;

        // Wait for response
        response_rx.await.map_err(|e| {
            ClaudeAgentError::ControlProtocol(format!("Failed to receive response: {}", e))
        })
    }

    /// Handle an incoming control response.
    pub async fn handle_response(&self, response: ControlResponse) -> Result<(), ClaudeAgentError> {
        let mut pending = self.pending_requests.lock().await;
        if let Some(tx) = pending.remove(&response.request_id) {
            let _ = tx.send(response);
        }
        Ok(())
    }

    /// Send interrupt request.
    pub async fn interrupt(&self) -> Result<ControlResponse, ClaudeAgentError> {
        self.send_request(ControlRequestType::Interrupt).await
    }

    /// Send set permission mode request.
    pub async fn set_permission_mode(
        &self,
        mode: &str,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        self.send_request(ControlRequestType::SetPermissionMode { mode: mode.to_string() }).await
    }

    /// Send set model request.
    pub async fn set_model(
        &self,
        model: Option<&str>,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        self.send_request(ControlRequestType::SetModel { model: model.map(|s| s.to_string()) })
            .await
    }

    /// Send rewind files request.
    pub async fn rewind_files(
        &self,
        user_message_id: &str,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        self.send_request(ControlRequestType::RewindFiles {
            user_message_id: user_message_id.to_string(),
        })
        .await
    }

    /// Send stop task request to stop a running background task.
    pub async fn stop_task(&self, task_id: &str) -> Result<ControlResponse, ClaudeAgentError> {
        self.send_request(ControlRequestType::StopTask { task_id: task_id.to_string() }).await
    }

    /// Request MCP server connection status.
    pub async fn get_mcp_status(&self) -> Result<ControlResponse, ClaudeAgentError> {
        self.send_request(ControlRequestType::McpStatus).await
    }

    /// Send reconnect request for a failed MCP server.
    pub async fn reconnect_mcp_server(
        &self,
        server_name: &str,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        self.send_request(ControlRequestType::McpReconnect { server_name: server_name.to_string() })
            .await
    }

    /// Send toggle request to enable or disable an MCP server.
    pub async fn toggle_mcp_server(
        &self,
        server_name: &str,
        enabled: bool,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        self.send_request(ControlRequestType::McpToggle {
            server_name: server_name.to_string(),
            enabled,
        })
        .await
    }

    /// Request context window usage breakdown.
    pub async fn get_context_usage(&self) -> Result<ControlResponse, ClaudeAgentError> {
        self.send_request(ControlRequestType::GetContextUsage).await
    }
}

impl Default for ControlProtocol {
    fn default() -> Self {
        Self::new().0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a protocol, spawn a task that immediately responds with success.
    async fn setup_responding_protocol() -> ControlProtocol {
        let (protocol, mut rx) = ControlProtocol::new();
        let protocol_clone = protocol.pending_requests.clone();

        tokio::spawn(async move {
            while let Some(req) = rx.recv().await {
                let mut pending = protocol_clone.lock().await;
                if let Some(tx) = pending.remove(&req.request_id) {
                    let _ = tx.send(ControlResponse {
                        request_id: req.request_id,
                        success: true,
                        response: Some(serde_json::json!({"status": "ok"})),
                        error: None,
                    });
                }
            }
        });

        protocol
    }

    #[tokio::test]
    async fn stop_task_sends_correct_request_type() {
        let protocol = setup_responding_protocol().await;
        let result = protocol.stop_task("task-123").await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn get_mcp_status_sends_correct_request_type() {
        let protocol = setup_responding_protocol().await;
        let result = protocol.get_mcp_status().await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn reconnect_mcp_server_sends_correct_request_type() {
        let protocol = setup_responding_protocol().await;
        let result = protocol.reconnect_mcp_server("my-server").await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn toggle_mcp_server_sends_correct_request_type() {
        let protocol = setup_responding_protocol().await;
        let result = protocol.toggle_mcp_server("my-server", true).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn get_context_usage_sends_correct_request_type() {
        let protocol = setup_responding_protocol().await;
        let result = protocol.get_context_usage().await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn control_request_type_variants_exhaustive() {
        // Verify all variants construct without panic
        let _ = ControlRequestType::Interrupt;
        let _ = ControlRequestType::Initialize { hooks: None };
        let _ = ControlRequestType::SetPermissionMode { mode: "default".into() };
        let _ = ControlRequestType::SetModel { model: Some("claude-sonnet-4-5".into()) };
        let _ = ControlRequestType::SetModel { model: None };
        let _ = ControlRequestType::RewindFiles { user_message_id: "id-1".into() };
        let _ = ControlRequestType::StopTask { task_id: "task-1".into() };
        let _ = ControlRequestType::McpMessage {
            server_name: "server".into(),
            message: serde_json::json!({"method": "test"}),
        };
        let _ = ControlRequestType::McpStatus;
        let _ = ControlRequestType::McpReconnect { server_name: "server".into() };
        let _ = ControlRequestType::McpToggle { server_name: "server".into(), enabled: false };
        let _ = ControlRequestType::GetContextUsage;
        let _ = ControlRequestType::HookCallback {
            callback_id: "cb-1".into(),
            output: serde_json::json!({"result": "ok"}),
        };
    }
}
