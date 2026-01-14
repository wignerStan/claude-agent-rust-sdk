//! Control protocol implementation for SDK communication.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, Mutex};
use uuid::Uuid;

use claude_agent_types::ClaudeAgentError;

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
    McpMessage {
        server_name: String,
        message: serde_json::Value,
    },
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
}

impl Default for ControlProtocol {
    fn default() -> Self {
        Self::new().0
    }
}
