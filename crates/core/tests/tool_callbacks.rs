//! Tool callbacks tests.

use claude_agent_core::hooks::{HookCallback, HookContext, HookInput, HookOutput, HookRegistry};
use claude_agent_types::hooks::HookEvent;
use std::sync::{Arc, Mutex};

fn make_test_callback(counter: Arc<Mutex<i32>>) -> HookCallback {
    Arc::new(
        move |_input: HookInput, _id: Option<String>, _ctx: HookContext| {
            let c = counter.clone();
            Box::pin(async move {
                let mut guard = c.lock().unwrap();
                *guard += 1;
                Ok(HookOutput {
                    continue_execution: true,
                    ..Default::default()
                })
            })
        },
    )
}

#[tokio::test]
async fn test_tool_execution_callback() {
    let counter = Arc::new(Mutex::new(0));
    let mut registry = HookRegistry::new();

    registry.register(
        HookEvent::PreToolUse,
        None,
        make_test_callback(counter.clone()),
        None,
    );

    let input = HookInput {
        event_name: HookEvent::PreToolUse,
        session_id: "test".to_string(),
        transcript_path: "/tmp/test".to_string(),
        cwd: ".".to_string(),
        permission_mode: None,
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({"command": "echo hello"})),
        tool_response: None,
        prompt: None,
    };

    let outputs = registry
        .execute_hooks(&HookEvent::PreToolUse, input, None)
        .await
        .unwrap();

    assert_eq!(outputs.len(), 1);
    assert!(outputs[0].continue_execution);
    assert_eq!(*counter.lock().unwrap(), 1);
}

#[tokio::test]
async fn test_tool_matcher_filter() {
    let counter = Arc::new(Mutex::new(0));
    let mut registry = HookRegistry::new();

    registry.register(
        HookEvent::PreToolUse,
        Some("Write|Edit".to_string()),
        make_test_callback(counter.clone()),
        None,
    );

    let input = HookInput {
        event_name: HookEvent::PreToolUse,
        session_id: "test".to_string(),
        transcript_path: "/tmp/test".to_string(),
        cwd: ".".to_string(),
        permission_mode: None,
        tool_name: Some("Bash".to_string()), // Won't match
        tool_input: None,
        tool_response: None,
        prompt: None,
    };

    let outputs = registry
        .execute_hooks(&HookEvent::PreToolUse, input, None)
        .await
        .unwrap();

    // Should not trigger because Bash doesn't match Write|Edit
    assert_eq!(outputs.len(), 0);
    assert_eq!(*counter.lock().unwrap(), 0);
}
