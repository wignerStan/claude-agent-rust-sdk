//! E2E tests for edge cases and error conditions.
//!
//! All tests require: claude CLI + API key.
//! Run: `cargo test -p claude-agent-api --test e2e_edge_cases -- --ignored`

use claude_agent::api::ClaudeAgentClient;
use futures::StreamExt;
use std::time::Duration;

mod e2e_common;

#[tokio::test]
#[ignore]
async fn test_live_unicode_content() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");

    // Send prompt with emoji, CJK, and RTL text
    let prompt = "Reply with exactly: Hello \u{4E16}\u{754C} \u{1F30D} \u{0645}\u{0631}\u{062D}\u{0628}\u{0627}";
    let messages = {
        let mut stream = client.query(prompt).await.expect("query failed");
        e2e_common::collect_until_result(&mut stream, e2e_common::STANDARD_TIMEOUT).await
    };
    assert!(!messages.is_empty(), "Expected response with unicode content");

    client.disconnect().await.ok();
}

#[tokio::test]
#[ignore]
async fn test_live_rapid_connect_disconnect() {
    e2e_common::init_tracing();

    for i in 0..3 {
        let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
        client.connect().await.unwrap_or_else(|e| panic!("connect failed on iteration {i}: {e}"));
        client
            .disconnect()
            .await
            .unwrap_or_else(|e| panic!("disconnect failed on iteration {i}: {e}"));
    }
}

#[tokio::test]
#[ignore]
async fn test_live_query_after_disconnect() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");
    client.disconnect().await.expect("disconnect failed");

    // After disconnect, attempting to query should either error or auto-reconnect
    let result = client.query("hello").await;
    // We don't assert the specific behavior (auto-reconnect vs error),
    // just that it doesn't hang or panic
    match result {
        Ok(mut stream) => {
            // If auto-reconnect, consume the stream
            while stream.next().await.is_some() {}
        },
        Err(e) => {
            // If error, that's acceptable too
            println!("Query after disconnect returned error (expected): {e}");
        },
    }
}

#[tokio::test]
#[ignore]
async fn test_live_large_prompt() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");

    // Generate a ~10KB prompt
    let large_text: String = "The quick brown fox jumps over the lazy dog. ".repeat(200);
    let prompt = format!("Read this text and confirm you received it:\n\n{large_text}");

    let messages = {
        let mut stream = client.query(&prompt).await.expect("query failed");
        e2e_common::collect_until_result(&mut stream, e2e_common::SLOW_TIMEOUT).await
    };
    assert!(!messages.is_empty(), "Expected response for large prompt");

    client.disconnect().await.ok();
}

#[tokio::test]
#[ignore]
async fn test_live_timeout_handling() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");

    // Use a very short timeout (1 second) on a potentially slow query
    let result = {
        let mut stream =
            client.query("Write a 500-word essay about Rust").await.expect("query failed");
        tokio::time::timeout(Duration::from_secs(1), async {
            let mut count = 0u32;
            while stream.next().await.is_some() {
                count += 1;
            }
            count
        })
        .await
    };

    // We just verify it doesn't hang — timeout is acceptable
    match result {
        Ok(count) => println!("Query completed within timeout, {count} messages"),
        Err(_) => println!("Query timed out as expected for short timeout"),
    }

    client.disconnect().await.ok();
}
