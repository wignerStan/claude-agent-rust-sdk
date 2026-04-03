//! Tests for hook system: HookRegistry, HookInput/Output.

use claude_agent::core::hooks::{HookInput, HookOutput, HookRegistry};
use claude_agent::types::hooks::HookEvent;
use claude_agent::types::ClaudeAgentError;
use std::sync::Arc;

#[test]
fn registry_new_empty() {
    let registry = HookRegistry::new();
    assert!(registry.get_hooks(&HookEvent::PreToolUse).is_none());
}

#[test]
fn registry_default() {
    let registry = HookRegistry::default();
    assert!(registry.get_hooks(&HookEvent::Stop).is_none());
}

#[test]
fn registry_register_single() {
    let mut registry = HookRegistry::new();
    let cb: claude_agent::core::hooks::HookCallback =
        Arc::new(|_input, _id, _ctx| Box::pin(async { Ok(HookOutput::default()) }));
    registry.register(HookEvent::PreToolUse, None, cb, None);
    assert_eq!(registry.get_hooks(&HookEvent::PreToolUse).unwrap().len(), 1);
}

#[test]
fn registry_register_multiple_same_event() {
    let mut registry = HookRegistry::new();
    for _ in 0..3 {
        let cb: claude_agent::core::hooks::HookCallback =
            Arc::new(|_input, _id, _ctx| Box::pin(async { Ok(HookOutput::default()) }));
        registry.register(HookEvent::PostToolUse, None, cb, None);
    }
    assert_eq!(registry.get_hooks(&HookEvent::PostToolUse).unwrap().len(), 3);
}

#[test]
fn registry_register_different_events() {
    let mut registry = HookRegistry::new();
    let cb1: claude_agent::core::hooks::HookCallback =
        Arc::new(|_input, _id, _ctx| Box::pin(async { Ok(HookOutput::default()) }));
    let cb2: claude_agent::core::hooks::HookCallback =
        Arc::new(|_input, _id, _ctx| Box::pin(async { Ok(HookOutput::default()) }));
    registry.register(HookEvent::PreToolUse, None, cb1, None);
    registry.register(HookEvent::Stop, None, cb2, None);
    assert!(registry.get_hooks(&HookEvent::PreToolUse).is_some());
    assert!(registry.get_hooks(&HookEvent::Stop).is_some());
    assert!(registry.get_hooks(&HookEvent::UserPromptSubmit).is_none());
}

#[test]
fn registry_register_with_matcher() {
    let mut registry = HookRegistry::new();
    let cb: claude_agent::core::hooks::HookCallback =
        Arc::new(|_input, _id, _ctx| Box::pin(async { Ok(HookOutput::default()) }));
    registry.register(HookEvent::PreToolUse, Some("Bash".to_string()), cb, Some(5.0));
    let hooks = registry.get_hooks(&HookEvent::PreToolUse).unwrap();
    assert_eq!(hooks[0].matcher.as_deref(), Some("Bash"));
    assert_eq!(hooks[0].timeout, Some(5.0));
}

#[test]
fn hook_output_default() {
    let output = HookOutput::default();
    assert!(!output.continue_execution);
    assert!(!output.suppress_output);
    assert!(output.stop_reason.is_none());
    assert!(output.decision.is_none());
}

#[test]
fn hook_output_clone() {
    let output = HookOutput {
        continue_execution: true,
        system_message: Some("msg".to_string()),
        ..Default::default()
    };
    let cloned = output.clone();
    assert!(cloned.continue_execution);
    assert_eq!(cloned.system_message.as_deref(), Some("msg"));
}

#[test]
fn hook_input_clone_and_debug() {
    let input = HookInput {
        event_name: HookEvent::Stop,
        session_id: "s1".to_string(),
        transcript_path: "/t".to_string(),
        cwd: "/w".to_string(),
        permission_mode: Some("plan".to_string()),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({"cmd": "ls"})),
        tool_response: None,
        prompt: None,
    };
    let cloned = input.clone();
    assert_eq!(cloned.session_id, "s1");
    assert!(format!("{:?}", input).contains("s1"));
}

fn make_hook_input(tool_name: Option<&str>) -> HookInput {
    HookInput {
        event_name: HookEvent::PreToolUse,
        session_id: "test".to_string(),
        transcript_path: "/tmp".to_string(),
        cwd: "/home".to_string(),
        permission_mode: None,
        tool_name: tool_name.map(|s| s.to_string()),
        tool_input: None,
        tool_response: None,
        prompt: None,
    }
}

#[tokio::test]
async fn execute_no_hooks_returns_empty() {
    let registry = HookRegistry::new();
    let results =
        registry.execute_hooks(&HookEvent::PreToolUse, make_hook_input(None), None).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn execute_runs_callback() {
    let mut registry = HookRegistry::new();
    let cb: claude_agent::core::hooks::HookCallback = Arc::new(|input, _id, _ctx| {
        Box::pin(async move {
            Ok(HookOutput {
                continue_execution: true,
                reason: Some(format!("tool={}", input.tool_name.as_deref().unwrap_or("none"))),
                ..Default::default()
            })
        })
    });
    registry.register(HookEvent::PreToolUse, None, cb, None);
    let results = registry
        .execute_hooks(&HookEvent::PreToolUse, make_hook_input(Some("Bash")), None)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].continue_execution);
    assert_eq!(results[0].reason.as_deref(), Some("tool=Bash"));
}

#[tokio::test]
async fn execute_matcher_filters() {
    let mut registry = HookRegistry::new();
    let cb: claude_agent::core::hooks::HookCallback = Arc::new(|_input, _id, _ctx| {
        Box::pin(async {
            Ok(HookOutput { decision: Some("matched".to_string()), ..Default::default() })
        })
    });
    registry.register(HookEvent::PreToolUse, Some("Write".to_string()), cb, None);

    // Match
    let results = registry
        .execute_hooks(&HookEvent::PreToolUse, make_hook_input(Some("Write")), None)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);

    // No match
    let results = registry
        .execute_hooks(&HookEvent::PreToolUse, make_hook_input(Some("Read")), None)
        .await
        .unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn execute_no_tool_name_skips_matcher() {
    let mut registry = HookRegistry::new();
    let cb: claude_agent::core::hooks::HookCallback = Arc::new(|_input, _id, _ctx| {
        Box::pin(async {
            Ok(HookOutput { decision: Some("ran".to_string()), ..Default::default() })
        })
    });
    registry.register(HookEvent::PreToolUse, Some("Write".to_string()), cb, None);
    let results =
        registry.execute_hooks(&HookEvent::PreToolUse, make_hook_input(None), None).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn execute_callback_error_propagates() {
    let mut registry = HookRegistry::new();
    let cb: claude_agent::core::hooks::HookCallback = Arc::new(|_input, _id, _ctx| {
        Box::pin(async { Err(ClaudeAgentError::Transport("hook denied".to_string())) })
    });
    registry.register(HookEvent::PreToolUse, None, cb, None);
    let result = registry.execute_hooks(&HookEvent::PreToolUse, make_hook_input(None), None).await;
    assert!(result.is_err());
}

#[test]
fn get_hooks_unregistered_events() {
    let registry = HookRegistry::new();
    assert!(registry.get_hooks(&HookEvent::SubagentStop).is_none());
    assert!(registry.get_hooks(&HookEvent::PreCompact).is_none());
}
