//! Hook system implementation.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use claude_agent_types::hooks::HookEvent;
use claude_agent_types::ClaudeAgentError;

/// Type alias for hook callback functions.
pub type HookCallback = Arc<
    dyn Fn(
            HookInput,
            Option<String>,
            HookContext,
        ) -> Pin<Box<dyn Future<Output = Result<HookOutput, ClaudeAgentError>> + Send>>
        + Send
        + Sync,
>;

/// Input for hook callbacks.
#[derive(Debug, Clone)]
pub struct HookInput {
    pub event_name: HookEvent,
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
    pub permission_mode: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_response: Option<serde_json::Value>,
    pub prompt: Option<String>,
}

/// Context for hook callbacks.
#[derive(Debug, Clone, Default)]
pub struct HookContext {
    // Future: abort signal support
}

/// Output from hook callbacks.
#[derive(Debug, Clone, Default)]
pub struct HookOutput {
    pub continue_execution: bool,
    pub suppress_output: bool,
    pub stop_reason: Option<String>,
    pub decision: Option<String>,
    pub system_message: Option<String>,
    pub reason: Option<String>,
    pub hook_specific_output: Option<serde_json::Value>,
}

/// Hook registry for managing hook callbacks.
pub struct HookRegistry {
    hooks: HashMap<HookEvent, Vec<RegisteredHook>>,
}

/// A registered hook with its matcher and callback.
pub struct RegisteredHook {
    pub matcher: Option<String>,
    pub callback: HookCallback,
    pub timeout: Option<f64>,
}

impl HookRegistry {
    /// Create a new hook registry.
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// Register a hook callback.
    pub fn register(
        &mut self,
        event: HookEvent,
        matcher: Option<String>,
        callback: HookCallback,
        timeout: Option<f64>,
    ) {
        let hook = RegisteredHook {
            matcher,
            callback,
            timeout,
        };
        self.hooks.entry(event).or_default().push(hook);
    }

    /// Get hooks for an event.
    pub fn get_hooks(&self, event: &HookEvent) -> Option<&Vec<RegisteredHook>> {
        self.hooks.get(event)
    }

    /// Execute hooks for an event.
    pub async fn execute_hooks(
        &self,
        event: &HookEvent,
        input: HookInput,
        tool_use_id: Option<String>,
    ) -> Result<Vec<HookOutput>, ClaudeAgentError> {
        let mut outputs = Vec::new();

        if let Some(hooks) = self.hooks.get(event) {
            for hook in hooks {
                // Check matcher
                if let Some(ref matcher) = hook.matcher {
                    if let Some(ref tool_name) = input.tool_name {
                        if !matches_tool(matcher, tool_name) {
                            continue;
                        }
                    }
                }

                let context = HookContext::default();
                let output = (hook.callback)(input.clone(), tool_use_id.clone(), context).await?;
                outputs.push(output);
            }
        }

        Ok(outputs)
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a tool name matches a matcher pattern.
fn matches_tool(matcher: &str, tool_name: &str) -> bool {
    // Simple pattern matching: support | for OR
    matcher
        .split('|')
        .any(|pattern| pattern.trim() == tool_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_tool() {
        assert!(matches_tool("Bash", "Bash"));
        assert!(matches_tool("Write|Edit", "Write"));
        assert!(matches_tool("Write|Edit", "Edit"));
        assert!(!matches_tool("Write|Edit", "Read"));
    }
}
