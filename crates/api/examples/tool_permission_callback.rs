//! Tool permission callback example - Custom permission logic.

use claude_agent_core::permissions::{PermissionCallback, PermissionHandler};
use claude_agent_types::hooks::PermissionResult;
use std::sync::Arc;

fn main() {
    let mut handler = PermissionHandler::new();

    // Create a custom permission callback
    let callback: PermissionCallback = Arc::new(|tool_name, input, _ctx| {
        Box::pin(async move {
            println!("Permission check for tool: {}", tool_name);
            println!("Input: {:?}", input);

            // Example: deny destructive operations
            if tool_name.contains("Delete") || tool_name.contains("Remove") {
                return Ok(PermissionResult::Deny {
                    message: "Destructive operations are not allowed".to_string(),
                    interrupt: false,
                });
            }

            // Allow all other operations
            Ok(PermissionResult::Allow { updated_input: None, updated_permissions: None })
        })
    });

    handler.set_callback(callback);

    println!("Permission handler configured");
    println!("Destructive tools (Delete, Remove) will be denied");
    println!("All other tools will be allowed");
}
