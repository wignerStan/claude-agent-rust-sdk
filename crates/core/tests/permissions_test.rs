//! Tests for permission system: PermissionHandler.

use claude_agent_core::permissions::{PermissionCallback, PermissionHandler};
use claude_agent_types::hooks::{
    PermissionBehavior, PermissionResult, PermissionRuleValue, PermissionUpdate,
    PermissionUpdateDestination, PermissionUpdateType,
};
use claude_agent_types::ClaudeAgentError;
use std::sync::Arc;

#[test]
fn new_handler_no_callback() {
    let handler = PermissionHandler::new();
    assert!(!handler.has_callback());
}

#[test]
fn default_handler_no_callback() {
    let handler = PermissionHandler::default();
    assert!(!handler.has_callback());
}

#[tokio::test]
async fn no_callback_allows_by_default() {
    let handler = PermissionHandler::new();
    let result =
        handler.can_use_tool("Bash", serde_json::json!({"cmd": "ls"}), vec![]).await.unwrap();
    assert!(matches!(
        result,
        PermissionResult::Allow { updated_input: None, updated_permissions: None }
    ));
}

#[tokio::test]
async fn no_callback_passes_suggestions_through() {
    let handler = PermissionHandler::new();
    let suggestions = vec![PermissionUpdate {
        update_type: PermissionUpdateType::AddRules,
        rules: Some(vec![PermissionRuleValue {
            tool_name: "Bash".to_string(),
            rule_content: Some("ls".to_string()),
        }]),
        behavior: Some(PermissionBehavior::Allow),
        mode: None,
        directories: None,
        destination: Some(PermissionUpdateDestination::Session),
    }];
    let result = handler.can_use_tool("Bash", serde_json::json!({}), suggestions).await.unwrap();
    if let PermissionResult::Allow { updated_input, updated_permissions } = result {
        assert!(updated_input.is_none());
        assert!(updated_permissions.is_none());
    } else {
        panic!("Expected Allow");
    }
}

#[tokio::test]
async fn callback_allow() {
    let mut handler = PermissionHandler::new();
    let cb: PermissionCallback = Arc::new(|tool, _input, _ctx| {
        Box::pin(async move {
            if tool == "Read" {
                Ok(PermissionResult::Allow { updated_input: None, updated_permissions: None })
            } else {
                Ok(PermissionResult::Deny { message: "blocked".to_string(), interrupt: false })
            }
        })
    });
    handler.set_callback(cb);
    assert!(handler.has_callback());

    let result = handler.can_use_tool("Read", serde_json::json!({}), vec![]).await.unwrap();
    assert!(matches!(result, PermissionResult::Allow { .. }));

    let result = handler.can_use_tool("Bash", serde_json::json!({}), vec![]).await.unwrap();
    assert!(matches!(result, PermissionResult::Deny { .. }));
}

#[tokio::test]
async fn callback_receives_tool_and_input() {
    let mut handler = PermissionHandler::new();
    let cb: PermissionCallback = Arc::new(|tool, input, _ctx| {
        Box::pin(async move {
            assert_eq!(tool, "Write");
            assert_eq!(input["path"], "test.rs");
            Ok(PermissionResult::Allow { updated_input: None, updated_permissions: None })
        })
    });
    handler.set_callback(cb);
    handler.can_use_tool("Write", serde_json::json!({"path": "test.rs"}), vec![]).await.unwrap();
}

#[tokio::test]
async fn callback_receives_suggestions() {
    let mut handler = PermissionHandler::new();
    let suggestions = vec![PermissionUpdate {
        update_type: PermissionUpdateType::SetMode,
        rules: None,
        behavior: None,
        mode: Some("bypassPermissions".to_string()),
        directories: None,
        destination: None,
    }];
    let suggestions_clone = suggestions.clone();
    let cb: PermissionCallback = Arc::new(move |_tool, _input, ctx| {
        let _expected = suggestions_clone.clone();
        Box::pin(async move {
            assert_eq!(ctx.suggestions.len(), 1);
            assert_eq!(ctx.suggestions[0].mode.as_deref(), Some("bypassPermissions"));
            Ok(PermissionResult::Allow { updated_input: None, updated_permissions: None })
        })
    });
    handler.set_callback(cb);
    handler.can_use_tool("Bash", serde_json::json!({}), suggestions).await.unwrap();
}

#[tokio::test]
async fn callback_returns_updated_input() {
    let mut handler = PermissionHandler::new();
    let cb: PermissionCallback = Arc::new(|_tool, _input, _ctx| {
        Box::pin(async move {
            let mut map = std::collections::HashMap::new();
            map.insert("modified".to_string(), serde_json::json!(true));
            Ok(PermissionResult::Allow { updated_input: Some(map), updated_permissions: None })
        })
    });
    handler.set_callback(cb);
    let result = handler.can_use_tool("Edit", serde_json::json!({}), vec![]).await.unwrap();
    if let PermissionResult::Allow { updated_input, .. } = result {
        assert_eq!(updated_input.unwrap()["modified"], serde_json::json!(true));
    } else {
        panic!("Expected Allow with updated_input");
    }
}

#[tokio::test]
async fn callback_returns_updated_permissions() {
    let mut handler = PermissionHandler::new();
    let perm = PermissionUpdate {
        update_type: PermissionUpdateType::AddRules,
        rules: Some(vec![PermissionRuleValue {
            tool_name: "Bash".to_string(),
            rule_content: Some("npm test".to_string()),
        }]),
        behavior: Some(PermissionBehavior::Allow),
        mode: None,
        directories: None,
        destination: Some(PermissionUpdateDestination::Session),
    };
    let cb: PermissionCallback = Arc::new(move |_tool, _input, _ctx| {
        let update = perm.clone();
        Box::pin(async move {
            Ok(PermissionResult::Allow {
                updated_input: None,
                updated_permissions: Some(vec![update]),
            })
        })
    });
    handler.set_callback(cb);
    let result =
        handler.can_use_tool("Bash", serde_json::json!({"cmd": "npm test"}), vec![]).await.unwrap();
    if let PermissionResult::Allow { updated_permissions, .. } = result {
        let perms = updated_permissions.unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0].rules.as_ref().unwrap()[0].tool_name, "Bash");
    } else {
        panic!("Expected Allow with updated_permissions");
    }
}

#[tokio::test]
async fn callback_error_propagates() {
    let mut handler = PermissionHandler::new();
    let cb: PermissionCallback = Arc::new(|_tool, _input, _ctx| {
        Box::pin(async { Err(ClaudeAgentError::Transport("denied by policy".to_string())) })
    });
    handler.set_callback(cb);
    let result = handler.can_use_tool("Bash", serde_json::json!({}), vec![]).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("denied by policy"));
}
