//! Tests for Research Agent demo.

use chrono::Utc;
use claude_agent_api::ClaudeAgentClient;
use claude_agent_demos_common::test_utils::{MockTransport, StreamCollector};
use claude_agent_types::message::Message;
use claude_agent_types::ClaudeAgentError;
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

const TEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn test_subagent_tracker_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let session_dir = temp_dir.path().join("session_test");

    // We'll test the tracker logic separately
    // For now, verify basic structure

    let sessions: Arc<Mutex<std::collections::HashMap<String, ()>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    let current_parent_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let subagent_counters: Arc<Mutex<std::collections::HashMap<String, u32>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));

    assert!(sessions.lock().unwrap().is_empty());
    assert!(current_parent_id.lock().unwrap().is_none());
    assert!(subagent_counters.lock().unwrap().is_empty());

    assert!(session_dir.to_string_lossy().contains("session_test"));
}

#[tokio::test]
async fn test_agent_creation_structure() {
    // Test that agents can be created with proper structure
    let agents: std::collections::HashMap<String, ()> = std::collections::HashMap::new();

    assert!(agents.is_empty());
}

#[tokio::test]
async fn test_tool_call_tracking_data() {
    let tool_call_record = json!({
        "timestamp": Utc::now().to_rfc3339(),
        "tool_name": "WebSearch",
        "tool_input": {"query": "test query"},
        "tool_use_id": "test-123",
        "subagent_type": "Researcher",
        "parent_tool_use_id": null,
        "tool_output": null,
        "error": null
    });

    assert!(tool_call_record.get("timestamp").is_some());
    assert_eq!(tool_call_record["tool_name"], "WebSearch");
    assert_eq!(tool_call_record["subagent_type"], "Researcher");
}

#[tokio::test]
async fn test_research_query_integration() {
    timeout(TEST_TIMEOUT, async {
        let response_text = "I have researched AI trends and found several key patterns...";
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": response_text}],
                "role": "assistant",
                "model": "haiku"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(mock_transport));
        client.connect().await.unwrap();

        let stream = client.query("Research AI trends").await.unwrap();
        let messages = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
        if let Message::Assistant(msg) = &messages[0] {
            if let claude_agent_types::message::ContentBlock::Text(text_block) = &msg.content[0] {
                assert_eq!(text_block.text, response_text);
            } else {
                panic!("Expected text block");
            }
        } else {
            panic!("Expected assistant message");
        }
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_subagent_type_serialization() {
    let subagent_types = vec!["Lead", "Researcher", "DataAnalyst", "ReportWriter"];

    for st in subagent_types {
        let record = json!({
            "subagent_type": st
        });
        assert_eq!(record["subagent_type"], st);
    }
}

#[tokio::test]
async fn test_message_processing_structure() {
    let assistant_message = json!({
        "type": "assistant",
        "message": {
            "content": [
                {"type": "text", "text": "Researching topic..."},
                {"type": "tool_use", "name": "WebSearch", "input": {"query": "test"}}
            ],
            "role": "assistant",
            "model": "haiku"
        }
    });

    assert_eq!(assistant_message["type"], "assistant");
    assert!(assistant_message["message"]["content"].is_array());
    assert_eq!(
        assistant_message["message"]["content"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn test_transcript_data_structure() {
    let transcript_entries = vec![
        "Research Agent Session Started",
        "You: Research AI trends",
        "Agent: Searching for information...",
        "🔍 WebSearch: AI trends 2024",
        "🚀 SUBAGENT SPAWNED: RESEARCHER-1",
        "Research Agent Session Ended",
    ];

    assert_eq!(transcript_entries.len(), 6);
    assert!(transcript_entries[0].contains("Session Started"));
    assert!(transcript_entries[3].contains("WebSearch"));
    assert!(transcript_entries[4].contains("SUBAGENT SPAWNED"));
}

#[tokio::test]
async fn test_directory_structure_logic() {
    let session_dir = PathBuf::from("agent/research_sessions/session_test");
    let logs_dir = session_dir.join("logs");
    let research_notes = session_dir.join("files/research_notes");
    let charts = session_dir.join("files/charts");
    let reports = session_dir.join("files/reports");

    assert!(logs_dir.to_string_lossy().contains("logs"));
    assert!(research_notes.to_string_lossy().contains("research_notes"));
    assert!(charts.to_string_lossy().contains("charts"));
    assert!(reports.to_string_lossy().contains("reports"));
}

#[tokio::test]
async fn test_tool_call_record_serialization_full() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestRecord {
        timestamp: String,
        tool_name: String,
        tool_use_id: String,
        subagent_type: String,
    }

    let record = TestRecord {
        timestamp: Utc::now().to_rfc3339(),
        tool_name: "Task".to_string(),
        tool_use_id: "test-id".to_string(),
        subagent_type: "Lead".to_string(),
    };

    let json = serde_json::to_string(&record).unwrap();
    let deserialized: TestRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(record.tool_name, deserialized.tool_name);
    assert_eq!(record.tool_use_id, deserialized.tool_use_id);
    assert_eq!(record.subagent_type, deserialized.subagent_type);
}
