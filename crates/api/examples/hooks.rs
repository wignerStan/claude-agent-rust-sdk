//! Hooks example - Implementing hook callbacks.

use claude_agent_core::hooks::{HookCallback, HookInput, HookOutput, HookRegistry};
use claude_agent_types::hooks::HookEvent;
use std::sync::Arc;

fn main() {
    // Create hook registry
    let mut registry = HookRegistry::new();

    // Register a tool execution hook
    let tool_hook: HookCallback = Arc::new(|input: HookInput, _id, _ctx| {
        Box::pin(async move {
            println!("Tool execution hook triggered!");
            println!("  Tool: {:?}", input.tool_name);
            println!("  Input: {:?}", input.tool_input);

            Ok(HookOutput {
                continue_execution: true,
                ..Default::default()
            })
        })
    });

    registry.register(HookEvent::PreToolUse, None, tool_hook, None);

    // Register a hook that only triggers for specific tools
    let write_hook: HookCallback = Arc::new(|input: HookInput, _id, _ctx| {
        Box::pin(async move {
            println!("Write/Edit operation detected: {:?}", input.tool_name);
            Ok(HookOutput {
                continue_execution: true,
                reason: Some("Logging write operation".to_string()),
                ..Default::default()
            })
        })
    });

    registry.register(
        HookEvent::PreToolUse,
        Some("Write|Edit".to_string()),
        write_hook,
        None,
    );

    println!("Hook registry configured with 2 hooks");
    println!("These would be triggered during ClaudeAgent query execution");
}
