use claude_agent_types::{
    config::{ClaudeAgentOptions, PermissionMode, SystemPromptConfig, SystemPromptPreset},
    message::{
        AssistantMessage, ContentBlock, MessageContent, ResultMessage, TextBlock, ThinkingBlock,
        ToolResultBlock, ToolUseBlock, UserMessage,
    },
};

#[test]
fn test_user_message_creation() {
    let msg = UserMessage {
        content: MessageContent::Text("Hello, Claude!".to_string()),
        uuid: None,
        parent_tool_use_id: None,
    };

    match msg.content {
        MessageContent::Text(text) => assert_eq!(text, "Hello, Claude!"),
        _ => panic!("Expected text content"),
    }
}

#[test]
fn test_assistant_message_with_text() {
    let text_block = ContentBlock::Text(TextBlock { text: "Hello, human!".to_string() });

    let msg = AssistantMessage {
        content: vec![text_block],
        model: "claude-opus-4-1-20250805".to_string(),
        parent_tool_use_id: None,
        error: None,
    };

    assert_eq!(msg.content.len(), 1);
    if let ContentBlock::Text(block) = &msg.content[0] {
        assert_eq!(block.text, "Hello, human!");
    } else {
        panic!("Expected text block");
    }
}

#[test]
fn test_assistant_message_with_thinking() {
    let thinking_block = ContentBlock::Thinking(ThinkingBlock {
        thinking: "I'm thinking...".to_string(),
        signature: "sig-123".to_string(),
    });

    let msg = AssistantMessage {
        content: vec![thinking_block],
        model: "claude-opus-4-1-20250805".to_string(),
        parent_tool_use_id: None,
        error: None,
    };

    assert_eq!(msg.content.len(), 1);
    if let ContentBlock::Thinking(block) = &msg.content[0] {
        assert_eq!(block.thinking, "I'm thinking...");
        assert_eq!(block.signature, "sig-123");
    } else {
        panic!("Expected thinking block");
    }
}

#[test]
fn test_tool_use_block() {
    let mut input = serde_json::Map::new();
    input.insert("file_path".to_string(), serde_json::json!("/test.txt"));

    let block = ToolUseBlock {
        id: "tool-123".to_string(),
        name: "Read".to_string(),
        input: serde_json::Value::Object(input),
    };

    assert_eq!(block.id, "tool-123");
    assert_eq!(block.name, "Read");
    assert_eq!(block.input["file_path"], "/test.txt");
}

#[test]
fn test_tool_result_block() {
    use claude_agent_types::message::ToolResultContent;

    let block = ToolResultBlock {
        tool_use_id: "tool-123".to_string(),
        content: Some(ToolResultContent::Text("File contents here".to_string())),
        is_error: Some(false),
    };

    assert_eq!(block.tool_use_id, "tool-123");
    if let Some(ToolResultContent::Text(content)) = &block.content {
        assert_eq!(content, "File contents here");
    } else {
        panic!("Expected text content");
    }
    assert_eq!(block.is_error, Some(false));
}

#[test]
fn test_result_message() {
    let msg = ResultMessage {
        subtype: "success".to_string(),
        duration_ms: 1500,
        duration_api_ms: 1200,
        is_error: false,
        num_turns: 1,
        session_id: "session-123".to_string(),
        total_cost_usd: Some(0.01),
        usage: None,
        result: None,
        structured_output: None,
    };

    assert_eq!(msg.subtype, "success");
    assert_eq!(msg.total_cost_usd, Some(0.01));
    assert_eq!(msg.session_id, "session-123");
}

#[test]
fn test_default_options() {
    let options = ClaudeAgentOptions::default();
    assert!(options.allowed_tools.is_empty());
    assert!(options.system_prompt.is_none());
    assert!(options.permission_mode.is_none());
    assert!(!options.continue_conversation);
    assert!(options.disallowed_tools.is_empty());
}

#[test]
fn test_claude_code_options_with_tools() {
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string(), "Edit".to_string()],
        disallowed_tools: vec!["Bash".to_string()],
        ..Default::default()
    };

    assert_eq!(options.allowed_tools, vec!["Read", "Write", "Edit"]);
    assert_eq!(options.disallowed_tools, vec!["Bash"]);
}

#[test]
fn test_claude_code_options_with_permission_mode() {
    let options = ClaudeAgentOptions {
        permission_mode: Some(PermissionMode::BypassPermissions),
        ..Default::default()
    };
    assert_eq!(options.permission_mode, Some(PermissionMode::BypassPermissions));

    let options_plan =
        ClaudeAgentOptions { permission_mode: Some(PermissionMode::Plan), ..Default::default() };
    assert_eq!(options_plan.permission_mode, Some(PermissionMode::Plan));

    let options_default =
        ClaudeAgentOptions { permission_mode: Some(PermissionMode::Default), ..Default::default() };
    assert_eq!(options_default.permission_mode, Some(PermissionMode::Default));

    let options_accept = ClaudeAgentOptions {
        permission_mode: Some(PermissionMode::AcceptEdits),
        ..Default::default()
    };
    assert_eq!(options_accept.permission_mode, Some(PermissionMode::AcceptEdits));
}

#[test]
fn test_claude_code_options_with_system_prompt_string() {
    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPromptConfig::Text("You are a helpful assistant.".to_string())),
        ..Default::default()
    };

    match options.system_prompt {
        Some(SystemPromptConfig::Text(text)) => assert_eq!(text, "You are a helpful assistant."),
        _ => panic!("Expected text system prompt"),
    }
}

#[test]
fn test_claude_code_options_with_system_prompt_preset() {
    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPromptConfig::Preset(SystemPromptPreset::Preset {
            preset: "claude_code".to_string(),
            append: None,
        })),
        ..Default::default()
    };

    match options.system_prompt {
        Some(SystemPromptConfig::Preset(SystemPromptPreset::Preset { preset, append })) => {
            assert_eq!(preset, "claude_code");
            assert!(append.is_none());
        },
        _ => panic!("Expected preset system prompt"),
    }
}

#[test]
fn test_claude_code_options_with_system_prompt_preset_and_append() {
    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPromptConfig::Preset(SystemPromptPreset::Preset {
            preset: "claude_code".to_string(),
            append: Some("Be concise.".to_string()),
        })),
        ..Default::default()
    };

    match options.system_prompt {
        Some(SystemPromptConfig::Preset(SystemPromptPreset::Preset { preset, append })) => {
            assert_eq!(preset, "claude_code");
            assert_eq!(append, Some("Be concise.".to_string()));
        },
        _ => panic!("Expected preset system prompt"),
    }
}

#[test]
fn test_claude_code_options_with_session_continuation() {
    let options = ClaudeAgentOptions {
        continue_conversation: true,
        resume: Some("session-123".to_string()),
        ..Default::default()
    };

    assert!(options.continue_conversation);
    assert_eq!(options.resume, Some("session-123".to_string()));
}

#[test]
fn test_claude_code_options_with_model_specification() {
    let options = ClaudeAgentOptions {
        model: Some("claude-sonnet-4-5".to_string()),
        permission_prompt_tool_name: Some("CustomTool".to_string()),
        ..Default::default()
    };

    assert_eq!(options.model, Some("claude-sonnet-4-5".to_string()));
    assert_eq!(options.permission_prompt_tool_name, Some("CustomTool".to_string()));
}
