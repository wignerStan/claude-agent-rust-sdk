use claude_agent_core::{ClaudeAgent, ClaudeAgentOptions};
use claude_agent_types::Message;
use futures::StreamExt;
use serde_json::json;

mod common;
use common::MockTransport;

#[tokio::test]
async fn test_agent_initialization() {
    let options = ClaudeAgentOptions::default();
    let agent = ClaudeAgent::new(options);
    assert!(
        agent.current_session().is_none(),
        "Session should not be initialized before connect"
    );
}

#[tokio::test]
async fn test_agent_connects_and_queries() {
    let mut agent = ClaudeAgent::new(ClaudeAgentOptions::default());
    let transport = MockTransport::new();
    let transport_clone = transport.clone();

    agent.set_transport(Box::new(transport));

    // Connect
    agent.connect(None).await.expect("Connect failed");

    // Check initialization connection logic if any

    // Perform Query
    let prompt = "Hello World";

    // We need to simulate the response from the server for the query
    // The query writes a message, then waits for a stream of responses.

    // Spawn a task to feed responses once the query is sent
    let t_clone = transport_clone.clone();
    tokio::spawn(async move {
        // Give a bit of time for the agent to write the request
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        {
            let sent_msgs = t_clone.sent_messages.lock().unwrap();
            assert!(!sent_msgs.is_empty(), "Agent should have sent a message");
            // Verify the last message contains the prompt
            assert!(sent_msgs.last().unwrap().contains("Hello World"));
        }

        // Push response: Start turn
        t_clone
            .push_incoming(json!({
                "type": "message_start",
                "message": {
                    "id": "msg_123",
                    "role": "assistant",
                    "content": [],
                    "model": "claude-3-5-sonnet-20241022",
                    "type": "message",
                    "usage": {"input_tokens": 10, "output_tokens": 5}
                }
            }))
            .await;

        // Push content
        t_clone
            .push_incoming(json!({
                "type": "content_block_start",
                "index": 0,
                "content_block": {"type": "text", "text": ""}
            }))
            .await;

        t_clone
            .push_incoming(json!({
                "type": "content_block_delta",
                "index": 0,
                "delta": {"type": "text_delta", "text": "Hello! How can I help?"}
            }))
            .await;

        t_clone
            .push_incoming(json!({
                "type": "content_block_stop",
                "index": 0
            }))
            .await;

        t_clone
            .push_incoming(json!({
                "type": "message_delta",
                "delta": {"stop_reason": "end_turn", "stop_sequence": null},
                "usage": {"output_tokens": 10}
            }))
            .await;

        t_clone
            .push_incoming(json!({
                "type": "message_stop"
            }))
            .await;
    });

    let mut stream = agent.query(prompt).await.expect("Query failed");

    let mut collected_text = String::new();
    while let Some(msg_result) = stream.next().await {
        let msg = msg_result.expect("Message error");
        println!("Received: {:?}", msg);

        if let Message::ContentBlockDelta(delta) = &msg {
            if let claude_agent_types::message::Delta::TextDelta { text } = &delta.delta {
                collected_text.push_str(text);
            }
        }

        if matches!(msg, Message::MessageStop(_) | Message::Result(_)) {
            break;
        }
    }

    assert!(
        collected_text.contains("Hello! How can I help?"),
        "Should receive the assistant response"
    );
}
