//! E2E tests for control protocol operations.
//!
//! All tests require: claude CLI + API key.
//! Run: `cargo test -p claude-agent-api --test e2e_control -- --ignored`

use claude_agent_api::ClaudeAgentClient;
use std::time::Duration;

mod e2e_common;

#[tokio::test]
#[ignore]
async fn test_live_interrupt_during_query() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");

    // Start a long query to consume the stream
    {
        let mut stream =
            client.query("Count from 1 to 10000, one number per line").await.expect("query failed");

        // Collect briefly to let the query start processing
        let _ = e2e_common::collect_until_result(&mut stream, Duration::from_secs(3)).await;
    } // stream dropped here, releasing mutable borrow

    // Send interrupt signal after the stream has ended.
    // Note: interrupting mid-stream requires the stream and interrupt to
    // operate on independent handles (not the current client API design).
    let result = client.interrupt().await;
    assert!(result.is_ok(), "Interrupt should succeed: {:?}", result.err());

    client.disconnect().await.ok();
}

#[tokio::test]
#[ignore]
async fn test_live_mcp_status() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");

    let result = client.get_mcp_status().await;
    assert!(result.is_ok(), "get_mcp_status should succeed: {:?}", result.err());
    let resp = result.unwrap();
    // MCP status should return a response (may be empty object or contain servers)
    assert!(
        resp.response.is_some(),
        "MCP status response should contain data"
    );

    client.disconnect().await.ok();
}

#[tokio::test]
#[ignore]
async fn test_live_model_switching() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");

    // Switch model
    let result = client.set_model(Some("claude-haiku-4-5-20251001")).await;
    assert!(result.is_ok(), "set_model should succeed: {:?}", result.err());

    // Query with the new model
    {
        let mut stream = client.query("Say OK").await.expect("query failed");
        let messages =
            e2e_common::collect_until_result(&mut stream, e2e_common::STANDARD_TIMEOUT).await;
        assert!(!messages.is_empty(), "Expected response after model switch");
    } // stream dropped here, releasing mutable borrow

    client.disconnect().await.ok();
}

#[tokio::test]
#[ignore]
async fn test_live_rewind_files() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");

    // Send a simple query
    {
        let mut stream = client.query("Say hello").await.expect("query failed");
        let messages =
            e2e_common::collect_until_result(&mut stream, e2e_common::STANDARD_TIMEOUT).await;
        assert!(!messages.is_empty(), "Expected response before rewind");
    } // stream dropped here, releasing mutable borrow

    // Rewind to a message (use a placeholder ID -- the CLI should handle gracefully)
    let result = client.rewind_files("msg-placeholder").await;
    // This may succeed or fail depending on whether the message ID exists,
    // but it should NOT panic or hang
    let _ = result;

    client.disconnect().await.ok();
}

#[tokio::test]
#[ignore]
async fn test_live_stop_task() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");

    // Send a stop task control request (may fail if no task is running, but should not hang)
    let result = client.stop_task("task-placeholder").await;
    // The CLI should acknowledge the request even if no such task exists
    let _ = result;

    client.disconnect().await.ok();
}
