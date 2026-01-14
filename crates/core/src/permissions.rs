//! Permission system implementation.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use claude_agent_types::hooks::{PermissionResult, PermissionUpdate, ToolPermissionContext};
use claude_agent_types::ClaudeAgentError;

/// Type alias for permission callback functions.
pub type PermissionCallback = Arc<
    dyn Fn(
            String,
            serde_json::Value,
            ToolPermissionContext,
        )
            -> Pin<Box<dyn Future<Output = Result<PermissionResult, ClaudeAgentError>> + Send>>
        + Send
        + Sync,
>;

/// Permission handler for tool execution.
pub struct PermissionHandler {
    callback: Option<PermissionCallback>,
}

impl PermissionHandler {
    /// Create a new permission handler.
    pub fn new() -> Self {
        Self { callback: None }
    }

    /// Set the permission callback.
    pub fn set_callback(&mut self, callback: PermissionCallback) {
        self.callback = Some(callback);
    }

    /// Check if a tool can be used.
    pub async fn can_use_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
        suggestions: Vec<PermissionUpdate>,
    ) -> Result<PermissionResult, ClaudeAgentError> {
        match &self.callback {
            Some(callback) => {
                let context = ToolPermissionContext { suggestions };
                callback(tool_name.to_string(), input, context).await
            },
            None => {
                // No callback set, allow by default
                Ok(PermissionResult::Allow { updated_input: None, updated_permissions: None })
            },
        }
    }

    /// Check if a permission callback is set.
    pub fn has_callback(&self) -> bool {
        self.callback.is_some()
    }
}

impl Default for PermissionHandler {
    fn default() -> Self {
        Self::new()
    }
}
