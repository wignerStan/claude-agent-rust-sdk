use claude_agent_types::hooks::*;
use std::collections::HashMap;

#[test]
fn hook_event_all_variants_serde_roundtrip() {
    let variants = vec![
        HookEvent::PreToolUse,
        HookEvent::PostToolUse,
        HookEvent::UserPromptSubmit,
        HookEvent::Stop,
        HookEvent::SubagentStop,
        HookEvent::PreCompact,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

#[test]
fn hook_event_serde_values() {
    assert_eq!(serde_json::to_string(&HookEvent::PreToolUse).unwrap(), r#""PreToolUse""#);
    assert_eq!(serde_json::to_string(&HookEvent::PostToolUse).unwrap(), r#""PostToolUse""#);
    assert_eq!(
        serde_json::to_string(&HookEvent::UserPromptSubmit).unwrap(),
        r#""UserPromptSubmit""#
    );
    assert_eq!(serde_json::to_string(&HookEvent::Stop).unwrap(), r#""Stop""#);
    assert_eq!(serde_json::to_string(&HookEvent::SubagentStop).unwrap(), r#""SubagentStop""#);
    assert_eq!(serde_json::to_string(&HookEvent::PreCompact).unwrap(), r#""PreCompact""#);
}

#[test]
fn hook_event_equality() {
    assert_eq!(HookEvent::PreToolUse, HookEvent::PreToolUse);
    assert_ne!(HookEvent::PreToolUse, HookEvent::PostToolUse);
}

#[test]
fn hook_event_hashable() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(HookEvent::PreToolUse);
    set.insert(HookEvent::PostToolUse);
    set.insert(HookEvent::PreToolUse);
    assert_eq!(set.len(), 2);
}

#[test]
fn hook_matcher_full() {
    let m = HookMatcher { matcher: Some("Read".to_string()), timeout: Some(30.0) };
    let json = serde_json::to_string(&m).unwrap();
    let back: HookMatcher = serde_json::from_str(&json).unwrap();
    assert_eq!(back.matcher, Some("Read".to_string()));
    assert_eq!(back.timeout, Some(30.0));
}

#[test]
fn hook_matcher_minimal() {
    let m = HookMatcher { matcher: None, timeout: None };
    let json = serde_json::to_string(&m).unwrap();
    assert_eq!(json, "{}");
    let back: HookMatcher = serde_json::from_str(&json).unwrap();
    assert!(back.matcher.is_none());
    assert!(back.timeout.is_none());
}

#[test]
fn permission_update_type_all_variants() {
    let variants = vec![
        PermissionUpdateType::AddRules,
        PermissionUpdateType::ReplaceRules,
        PermissionUpdateType::RemoveRules,
        PermissionUpdateType::SetMode,
        PermissionUpdateType::AddDirectories,
        PermissionUpdateType::RemoveDirectories,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let _back: PermissionUpdateType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn permission_update_type_serde_values() {
    assert_eq!(serde_json::to_string(&PermissionUpdateType::AddRules).unwrap(), r#""addRules""#);
    assert_eq!(
        serde_json::to_string(&PermissionUpdateType::ReplaceRules).unwrap(),
        r#""replaceRules""#
    );
    assert_eq!(
        serde_json::to_string(&PermissionUpdateType::RemoveRules).unwrap(),
        r#""removeRules""#
    );
    assert_eq!(serde_json::to_string(&PermissionUpdateType::SetMode).unwrap(), r#""setMode""#);
    assert_eq!(
        serde_json::to_string(&PermissionUpdateType::AddDirectories).unwrap(),
        r#""addDirectories""#
    );
    assert_eq!(
        serde_json::to_string(&PermissionUpdateType::RemoveDirectories).unwrap(),
        r#""removeDirectories""#
    );
}

#[test]
fn permission_behavior_all_variants() {
    let variants =
        vec![PermissionBehavior::Allow, PermissionBehavior::Deny, PermissionBehavior::Ask];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let _back: PermissionBehavior = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn permission_behavior_serde_values() {
    assert_eq!(serde_json::to_string(&PermissionBehavior::Allow).unwrap(), r#""allow""#);
    assert_eq!(serde_json::to_string(&PermissionBehavior::Deny).unwrap(), r#""deny""#);
    assert_eq!(serde_json::to_string(&PermissionBehavior::Ask).unwrap(), r#""ask""#);
}

#[test]
fn permission_rule_value_full() {
    let r = PermissionRuleValue {
        tool_name: "Read".to_string(),
        rule_content: Some("/tmp/**".to_string()),
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: PermissionRuleValue = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tool_name, "Read");
    assert_eq!(back.rule_content, Some("/tmp/**".to_string()));
}

#[test]
fn permission_rule_value_no_content() {
    let r = PermissionRuleValue { tool_name: "Bash".to_string(), rule_content: None };
    let json = serde_json::to_string(&r).unwrap();
    assert!(!json.contains("ruleContent"));
    let back: PermissionRuleValue = serde_json::from_str(&json).unwrap();
    assert!(back.rule_content.is_none());
}

#[test]
fn permission_update_destination_all_variants() {
    let variants = vec![
        PermissionUpdateDestination::UserSettings,
        PermissionUpdateDestination::ProjectSettings,
        PermissionUpdateDestination::LocalSettings,
        PermissionUpdateDestination::Session,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let _back: PermissionUpdateDestination = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn permission_update_full() {
    let u = PermissionUpdate {
        update_type: PermissionUpdateType::AddRules,
        rules: Some(vec![PermissionRuleValue {
            tool_name: "Read".to_string(),
            rule_content: Some("/tmp/**".to_string()),
        }]),
        behavior: Some(PermissionBehavior::Allow),
        mode: Some("acceptEdits".to_string()),
        directories: Some(vec!["/workspace".to_string()]),
        destination: Some(PermissionUpdateDestination::ProjectSettings),
    };
    let json = serde_json::to_string(&u).unwrap();
    let back: PermissionUpdate = serde_json::from_str(&json).unwrap();
    match back.update_type {
        PermissionUpdateType::AddRules => {},
        _ => panic!(),
    }
    assert!(back.rules.is_some());
    match back.behavior {
        Some(PermissionBehavior::Allow) => {},
        _ => panic!(),
    }
    assert_eq!(back.mode, Some("acceptEdits".to_string()));
    assert_eq!(back.directories, Some(vec!["/workspace".to_string()]));
    match back.destination {
        Some(PermissionUpdateDestination::ProjectSettings) => {},
        _ => panic!(),
    }
}

#[test]
fn permission_update_minimal() {
    let u = PermissionUpdate {
        update_type: PermissionUpdateType::SetMode,
        rules: None,
        behavior: None,
        mode: Some("plan".to_string()),
        directories: None,
        destination: None,
    };
    let json = serde_json::to_string(&u).unwrap();
    let back: PermissionUpdate = serde_json::from_str(&json).unwrap();
    match back.update_type {
        PermissionUpdateType::SetMode => {},
        _ => panic!(),
    }
    assert!(back.rules.is_none());
    assert!(back.destination.is_none());
}

#[test]
fn permission_update_empty_rules() {
    let u = PermissionUpdate {
        update_type: PermissionUpdateType::RemoveRules,
        rules: Some(vec![]),
        behavior: None,
        mode: None,
        directories: None,
        destination: None,
    };
    let json = serde_json::to_string(&u).unwrap();
    let back: PermissionUpdate = serde_json::from_str(&json).unwrap();
    assert!(back.rules.as_ref().unwrap().is_empty());
}

#[test]
fn permission_update_add_directories() {
    let u = PermissionUpdate {
        update_type: PermissionUpdateType::AddDirectories,
        rules: None,
        behavior: None,
        mode: None,
        directories: Some(vec!["/d1".to_string(), "/d2".to_string()]),
        destination: Some(PermissionUpdateDestination::UserSettings),
    };
    let json = serde_json::to_string(&u).unwrap();
    let back: PermissionUpdate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.directories, Some(vec!["/d1".to_string(), "/d2".to_string()]));
}

#[test]
fn tool_permission_context_default() {
    let ctx = ToolPermissionContext { suggestions: vec![] };
    let json = serde_json::to_string(&ctx).unwrap();
    let back: ToolPermissionContext = serde_json::from_str(&json).unwrap();
    assert!(back.suggestions.is_empty());
}

#[test]
fn tool_permission_context_with_suggestions() {
    let ctx = ToolPermissionContext {
        suggestions: vec![PermissionUpdate {
            update_type: PermissionUpdateType::AddRules,
            rules: Some(vec![PermissionRuleValue {
                tool_name: "Read".to_string(),
                rule_content: None,
            }]),
            behavior: Some(PermissionBehavior::Allow),
            mode: None,
            directories: None,
            destination: None,
        }],
    };
    let json = serde_json::to_string(&ctx).unwrap();
    let back: ToolPermissionContext = serde_json::from_str(&json).unwrap();
    assert_eq!(back.suggestions.len(), 1);
}

#[test]
fn permission_result_allow_with_updates() {
    let result = PermissionResult::Allow {
        updated_input: Some({
            let mut m = HashMap::new();
            m.insert("cmd".to_string(), serde_json::json!("ls"));
            m
        }),
        updated_permissions: Some(vec![PermissionUpdate {
            update_type: PermissionUpdateType::AddRules,
            rules: Some(vec![PermissionRuleValue {
                tool_name: "Bash".to_string(),
                rule_content: None,
            }]),
            behavior: Some(PermissionBehavior::Allow),
            mode: None,
            directories: None,
            destination: None,
        }]),
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains(r#""behavior":"allow""#));
    let back: PermissionResult = serde_json::from_str(&json).unwrap();
    match back {
        PermissionResult::Allow { updated_input, updated_permissions } => {
            assert!(updated_input.is_some());
            assert!(updated_permissions.is_some());
        },
        _ => panic!(),
    }
}

#[test]
fn permission_result_allow_minimal() {
    let result = PermissionResult::Allow { updated_input: None, updated_permissions: None };
    let json = serde_json::to_string(&result).unwrap();
    assert!(!json.contains("updatedInput"));
    let back: PermissionResult = serde_json::from_str(&json).unwrap();
    match back {
        PermissionResult::Allow { updated_input, updated_permissions } => {
            assert!(updated_input.is_none());
            assert!(updated_permissions.is_none());
        },
        _ => panic!(),
    }
}

#[test]
fn permission_result_deny() {
    let result = PermissionResult::Deny { message: "Not allowed.".to_string(), interrupt: true };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains(r#""behavior":"deny""#));
    let back: PermissionResult = serde_json::from_str(&json).unwrap();
    match back {
        PermissionResult::Deny { message, interrupt } => {
            assert_eq!(message, "Not allowed.");
            assert!(interrupt);
        },
        _ => panic!(),
    }
}

#[test]
fn permission_result_deny_defaults() {
    let result = PermissionResult::Deny { message: String::new(), interrupt: false };
    let json = serde_json::to_string(&result).unwrap();
    let back: PermissionResult = serde_json::from_str(&json).unwrap();
    match back {
        PermissionResult::Deny { message, interrupt } => {
            assert!(message.is_empty());
            assert!(!interrupt);
        },
        _ => panic!(),
    }
}
