use claude_agent_types::message::*;
use std::collections::HashMap;

#[test]
fn content_block_text_serde_roundtrip() {
    let block = ContentBlock::Text(TextBlock { text: "Hello, world!".to_string() });
    let json = serde_json::to_string(&block).unwrap();
    let back: ContentBlock = serde_json::from_str(&json).unwrap();
    match back {
        ContentBlock::Text(t) => assert_eq!(t.text, "Hello, world!"),
        _ => panic!("expected Text variant"),
    }
}

#[test]
fn content_block_thinking_serde_roundtrip() {
    let block = ContentBlock::Thinking(ThinkingBlock {
        thinking: "Let me think...".to_string(),
        signature: "sig123".to_string(),
    });
    let json = serde_json::to_string(&block).unwrap();
    let back: ContentBlock = serde_json::from_str(&json).unwrap();
    match back {
        ContentBlock::Thinking(t) => {
            assert_eq!(t.thinking, "Let me think...");
            assert_eq!(t.signature, "sig123");
        },
        _ => panic!("expected Thinking variant"),
    }
}

#[test]
fn content_block_tool_use_serde_roundtrip() {
    let block = ContentBlock::ToolUse(ToolUseBlock {
        id: "tool-123".to_string(),
        name: "Read".to_string(),
        input: serde_json::json!({"file_path": "/tmp/test"}),
    });
    let json = serde_json::to_string(&block).unwrap();
    let back: ContentBlock = serde_json::from_str(&json).unwrap();
    match back {
        ContentBlock::ToolUse(t) => {
            assert_eq!(t.id, "tool-123");
            assert_eq!(t.name, "Read");
            assert_eq!(t.input["file_path"], "/tmp/test");
        },
        _ => panic!("expected ToolUse variant"),
    }
}

#[test]
fn content_block_tool_result_serde_roundtrip() {
    let block = ContentBlock::ToolResult(ToolResultBlock {
        tool_use_id: "tool-123".to_string(),
        content: Some(ToolResultContent::Text("file contents here".to_string())),
        is_error: None,
    });
    let json = serde_json::to_string(&block).unwrap();
    let back: ContentBlock = serde_json::from_str(&json).unwrap();
    match back {
        ContentBlock::ToolResult(t) => {
            assert_eq!(t.tool_use_id, "tool-123");
            assert!(t.is_error.is_none());
        },
        _ => panic!("expected ToolResult variant"),
    }
}

#[test]
fn content_block_tool_result_with_error() {
    let block = ContentBlock::ToolResult(ToolResultBlock {
        tool_use_id: "tool-456".to_string(),
        content: Some(ToolResultContent::Text("permission denied".to_string())),
        is_error: Some(true),
    });
    let json = serde_json::to_string(&block).unwrap();
    let back: ContentBlock = serde_json::from_str(&json).unwrap();
    match back {
        ContentBlock::ToolResult(t) => {
            assert_eq!(t.tool_use_id, "tool-456");
            assert_eq!(t.is_error, Some(true));
        },
        _ => panic!("expected ToolResult variant"),
    }
}

#[test]
fn content_block_tool_result_skip_none_content() {
    let block = ContentBlock::ToolResult(ToolResultBlock {
        tool_use_id: "tool-789".to_string(),
        content: None,
        is_error: None,
    });
    let json = serde_json::to_string(&block).unwrap();
    assert!(!json.contains("content"));
}

#[test]
fn message_content_text_serde_roundtrip() {
    let content = MessageContent::Text("Hello".to_string());
    let json = serde_json::to_string(&content).unwrap();
    let back: MessageContent = serde_json::from_str(&json).unwrap();
    match back {
        MessageContent::Text(s) => assert_eq!(s, "Hello"),
        MessageContent::Blocks(_) => panic!("expected Text variant"),
    }
}

#[test]
fn message_content_blocks_serde_roundtrip() {
    let content = MessageContent::Blocks(vec![
        ContentBlock::Text(TextBlock { text: "Block 1".to_string() }),
        ContentBlock::Text(TextBlock { text: "Block 2".to_string() }),
    ]);
    let json = serde_json::to_string(&content).unwrap();
    let back: MessageContent = serde_json::from_str(&json).unwrap();
    match back {
        MessageContent::Blocks(blocks) => assert_eq!(blocks.len(), 2),
        MessageContent::Text(_) => panic!("expected Blocks variant"),
    }
}

#[test]
fn message_content_default() {
    let content = MessageContent::default();
    match content {
        MessageContent::Text(s) => assert!(s.is_empty()),
        MessageContent::Blocks(_) => panic!("expected default Text variant"),
    }
}

#[test]
fn tool_result_content_text() {
    let content = ToolResultContent::Text("output".to_string());
    let json = serde_json::to_string(&content).unwrap();
    let back: ToolResultContent = serde_json::from_str(&json).unwrap();
    match back {
        ToolResultContent::Text(s) => assert_eq!(s, "output"),
        ToolResultContent::Blocks(_) => panic!("expected Text variant"),
    }
}

#[test]
fn tool_result_content_blocks() {
    let content = ToolResultContent::Blocks(vec![
        serde_json::json!({"type": "text", "text": "line1"}),
        serde_json::json!({"type": "text", "text": "line2"}),
    ]);
    let json = serde_json::to_string(&content).unwrap();
    let back: ToolResultContent = serde_json::from_str(&json).unwrap();
    match back {
        ToolResultContent::Blocks(blocks) => assert_eq!(blocks.len(), 2),
        ToolResultContent::Text(_) => panic!("expected Blocks variant"),
    }
}

#[test]
fn user_message_text_serde_roundtrip() {
    let msg = UserMessage {
        content: MessageContent::Text("Hello Claude".to_string()),
        uuid: Some("uuid-1".to_string()),
        parent_tool_use_id: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: UserMessage = serde_json::from_str(&json).unwrap();
    match back.content {
        MessageContent::Text(s) => assert_eq!(s, "Hello Claude"),
        MessageContent::Blocks(_) => panic!("expected Text content"),
    }
    assert_eq!(back.uuid, Some("uuid-1".to_string()));
    assert!(back.parent_tool_use_id.is_none());
}

#[test]
fn user_message_block_content() {
    let msg = UserMessage {
        content: MessageContent::Blocks(vec![ContentBlock::Text(TextBlock {
            text: "Block text".to_string(),
        })]),
        uuid: None,
        parent_tool_use_id: Some("parent-tool-1".to_string()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: UserMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.parent_tool_use_id, Some("parent-tool-1".to_string()));
}

#[test]
fn user_message_minimal() {
    let msg = UserMessage {
        content: MessageContent::Text(String::new()),
        uuid: None,
        parent_tool_use_id: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: UserMessage = serde_json::from_str(&json).unwrap();
    assert!(back.uuid.is_none());
    assert!(back.parent_tool_use_id.is_none());
}

#[test]
fn assistant_message_text_serde_roundtrip() {
    let msg = AssistantMessage {
        content: vec![ContentBlock::Text(TextBlock { text: "Response".to_string() })],
        model: "claude-sonnet-4-20250514".to_string(),
        parent_tool_use_id: None,
        error: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: AssistantMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.content.len(), 1);
    assert_eq!(back.model, "claude-sonnet-4-20250514");
    assert!(back.error.is_none());
}

#[test]
fn assistant_message_with_parent_tool_use_id() {
    let msg = AssistantMessage {
        content: vec![],
        model: "claude-sonnet-4-20250514".to_string(),
        parent_tool_use_id: Some("tool-123".to_string()),
        error: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: AssistantMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.parent_tool_use_id, Some("tool-123".to_string()));
}

#[test]
fn assistant_message_error_variants_serde() {
    let variants = vec![
        AssistantMessageError::AuthenticationFailed,
        AssistantMessageError::BillingError,
        AssistantMessageError::RateLimit,
        AssistantMessageError::InvalidRequest,
        AssistantMessageError::ServerError,
        AssistantMessageError::Unknown,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: AssistantMessageError = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&back).unwrap();
        assert_eq!(json, json2);
    }
}

#[test]
fn system_message_serde_roundtrip() {
    let msg =
        SystemMessage { subtype: "init".to_string(), data: serde_json::json!({"key": "value"}) };
    let json = serde_json::to_string(&msg).unwrap();
    let back: SystemMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.subtype, "init");
    assert_eq!(back.data["key"], "value");
}

#[test]
fn result_message_full_serde_roundtrip() {
    let mut usage = HashMap::new();
    usage.insert("input_tokens".to_string(), serde_json::json!(100));
    usage.insert("output_tokens".to_string(), serde_json::json!(200));
    let msg = ResultMessage {
        subtype: "success".to_string(),
        duration_ms: 5000,
        duration_api_ms: 3000,
        is_error: false,
        num_turns: 5,
        session_id: "sess-abc".to_string(),
        total_cost_usd: Some(0.05),
        usage: Some(usage),
        result: Some("Task completed".to_string()),
        structured_output: Some(serde_json::json!({"answer": 42})),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ResultMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.subtype, "success");
    assert_eq!(back.duration_ms, 5000);
    assert_eq!(back.duration_api_ms, 3000);
    assert!(!back.is_error);
    assert_eq!(back.num_turns, 5);
    assert_eq!(back.session_id, "sess-abc");
    assert_eq!(back.total_cost_usd, Some(0.05));
    assert!(back.usage.is_some());
    assert_eq!(back.result, Some("Task completed".to_string()));
    assert!(back.structured_output.is_some());
}

#[test]
fn result_message_minimal() {
    let msg = ResultMessage {
        subtype: "error".to_string(),
        duration_ms: 100,
        duration_api_ms: 50,
        is_error: true,
        num_turns: 1,
        session_id: "sess-err".to_string(),
        total_cost_usd: None,
        usage: None,
        result: None,
        structured_output: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ResultMessage = serde_json::from_str(&json).unwrap();
    assert!(back.is_error);
    assert!(back.total_cost_usd.is_none());
    assert!(back.result.is_none());
}

#[test]
fn stream_event_serde_roundtrip() {
    let event = StreamEvent {
        uuid: "evt-1".to_string(),
        session_id: "sess-1".to_string(),
        event: serde_json::json!({"type": "message_start"}),
        parent_tool_use_id: Some("parent-1".to_string()),
    };
    let json = serde_json::to_string(&event).unwrap();
    let back: StreamEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(back.uuid, "evt-1");
    assert_eq!(back.session_id, "sess-1");
    assert_eq!(back.parent_tool_use_id, Some("parent-1".to_string()));
}

#[test]
fn stream_event_no_parent() {
    let event = StreamEvent {
        uuid: "evt-2".to_string(),
        session_id: "sess-2".to_string(),
        event: serde_json::json!({"type": "ping"}),
        parent_tool_use_id: None,
    };
    let json = serde_json::to_string(&event).unwrap();
    let back: StreamEvent = serde_json::from_str(&json).unwrap();
    assert!(back.parent_tool_use_id.is_none());
}

#[test]
fn message_user_variant() {
    let msg = Message::User(UserMessage {
        content: MessageContent::Text("hi".to_string()),
        uuid: None,
        parent_tool_use_id: None,
    });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::User(_) => {},
        _ => panic!("expected User variant"),
    }
}

#[test]
fn message_assistant_variant() {
    let msg = Message::Assistant(AssistantMessage {
        content: vec![],
        model: "m".to_string(),
        parent_tool_use_id: None,
        error: None,
    });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::Assistant(_) => {},
        _ => panic!("expected Assistant variant"),
    }
}

#[test]
fn message_system_variant() {
    let msg =
        Message::System(SystemMessage { subtype: "init".to_string(), data: serde_json::json!({}) });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::System(_) => {},
        _ => panic!("expected System variant"),
    }
}

#[test]
fn message_result_variant() {
    let msg = Message::Result(ResultMessage {
        subtype: "ok".to_string(),
        duration_ms: 100,
        duration_api_ms: 50,
        is_error: false,
        num_turns: 1,
        session_id: "s".to_string(),
        total_cost_usd: None,
        usage: None,
        result: None,
        structured_output: None,
    });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::Result(_) => {},
        _ => panic!("expected Result variant"),
    }
}

#[test]
fn message_stream_event_variant() {
    let msg = Message::StreamEvent(StreamEvent {
        uuid: "u".to_string(),
        session_id: "s".to_string(),
        event: serde_json::json!({}),
        parent_tool_use_id: None,
    });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::StreamEvent(_) => {},
        _ => panic!("expected StreamEvent variant"),
    }
}

#[test]
fn message_start_variant() {
    let msg = Message::MessageStart(MessageStart {
        message: AssistantMessage {
            content: vec![],
            model: "m".to_string(),
            parent_tool_use_id: None,
            error: None,
        },
    });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::MessageStart(_) => {},
        _ => panic!("expected MessageStart variant"),
    }
}

#[test]
fn message_delta_variant() {
    let msg = Message::MessageDelta(MessageDelta {
        delta: MessageDeltaBody { stop_reason: Some("end_turn".to_string()), stop_sequence: None },
        usage: None,
    });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::MessageDelta(_) => {},
        _ => panic!("expected MessageDelta variant"),
    }
}

#[test]
fn message_stop_variant() {
    let msg = Message::MessageStop(MessageStop);
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::MessageStop(_) => {},
        _ => panic!("expected MessageStop variant"),
    }
}

#[test]
fn ping_variant_deserialize() {
    // Message::Ping has a serialization conflict between Message's tag "type"
    // and Ping's inner "type" field, so test deserialization directly.
    let json = r#"{"type":"ping"}"#;
    let back: Message = serde_json::from_str(json).unwrap();
    match back {
        Message::Ping(p) => assert!(p.event_type.is_none()),
        _ => panic!("expected Ping variant"),
    }
}

#[test]
fn ping_none_event_type() {
    let p = Ping { event_type: None };
    let json = serde_json::to_string(&p).unwrap();
    let back: Ping = serde_json::from_str(&json).unwrap();
    assert!(back.event_type.is_none());
}

#[test]
fn error_variant() {
    let msg = Message::Error(ErrorEvent {
        error: ErrorBody {
            error_type: "server_error".to_string(),
            message: "something went wrong".to_string(),
        },
    });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::Error(e) => {
            assert_eq!(e.error.error_type, "server_error");
            assert_eq!(e.error.message, "something went wrong");
        },
        _ => panic!("expected Error variant"),
    }
}

#[test]
fn content_block_start_variant() {
    let msg = Message::ContentBlockStart(ContentBlockStart {
        index: 0,
        content_block: ContentBlock::Text(TextBlock { text: "partial".to_string() }),
    });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::ContentBlockStart(cbs) => assert_eq!(cbs.index, 0),
        _ => panic!("expected ContentBlockStart variant"),
    }
}

#[test]
fn content_block_delta_text_delta() {
    let msg = Message::ContentBlockDelta(ContentBlockDelta {
        index: 0,
        delta: Delta::TextDelta { text: "more text".to_string() },
    });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::ContentBlockDelta(cbd) => {
            assert_eq!(cbd.index, 0);
            match cbd.delta {
                Delta::TextDelta { text } => assert_eq!(text, "more text"),
                _ => panic!("expected TextDelta"),
            }
        },
        _ => panic!("expected ContentBlockDelta variant"),
    }
}

#[test]
fn content_block_stop_variant() {
    let msg = Message::ContentBlockStop(ContentBlockStop { index: 0 });
    let json = serde_json::to_string(&msg).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    match back {
        Message::ContentBlockStop(cbs) => assert_eq!(cbs.index, 0),
        _ => panic!("expected ContentBlockStop variant"),
    }
}

#[test]
fn delta_text_delta_direct() {
    let d = Delta::TextDelta { text: "hello".to_string() };
    let json = serde_json::to_string(&d).unwrap();
    let back: Delta = serde_json::from_str(&json).unwrap();
    match back {
        Delta::TextDelta { text } => assert_eq!(text, "hello"),
        _ => panic!("expected TextDelta"),
    }
}

#[test]
fn delta_input_json_delta_direct() {
    let d = Delta::InputJsonDelta { partial_json: "{\"key\":".to_string() };
    let json = serde_json::to_string(&d).unwrap();
    let back: Delta = serde_json::from_str(&json).unwrap();
    match back {
        Delta::InputJsonDelta { partial_json } => assert_eq!(partial_json, "{\"key\":"),
        _ => panic!("expected InputJsonDelta"),
    }
}

#[test]
fn delta_tool_use_direct() {
    let d = Delta::ToolUse {
        id: Some("tool-1".to_string()),
        name: Some("Read".to_string()),
        input: Some(serde_json::json!({"file": "/tmp"})),
    };
    let json = serde_json::to_string(&d).unwrap();
    let back: Delta = serde_json::from_str(&json).unwrap();
    match back {
        Delta::ToolUse { id, name, input } => {
            assert_eq!(id, Some("tool-1".to_string()));
            assert_eq!(name, Some("Read".to_string()));
            assert!(input.is_some());
        },
        _ => panic!("expected ToolUse delta"),
    }
}

#[test]
fn usage_full_serde_roundtrip() {
    let usage = Usage { input_tokens: Some(100), output_tokens: 200 };
    let json = serde_json::to_string(&usage).unwrap();
    let back: Usage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.input_tokens, Some(100));
    assert_eq!(back.output_tokens, 200);
}

#[test]
fn usage_none_input() {
    let usage = Usage { input_tokens: None, output_tokens: 50 };
    let json = serde_json::to_string(&usage).unwrap();
    let back: Usage = serde_json::from_str(&json).unwrap();
    assert!(back.input_tokens.is_none());
    assert_eq!(back.output_tokens, 50);
}
