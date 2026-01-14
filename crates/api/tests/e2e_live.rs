//! End-to-End Live Tests.
//!
//! These tests run against a real Claude Code CLI installation and the configured API.
//! They are ignored by default and must be run explicitly with `cargo test --ignored`.
//!
//! Requires:
//! - `claude` CLI installed and in PATH.
//! - Valid ANTHROPIC_API_KEY or compatible auth token in environment.

use futures::StreamExt;
use std::env;
use std::time::Duration;
use tokio::time::timeout;

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::message::Message;
use claude_agent_types::ClaudeAgentOptions;

// Helper to get options with current environment
fn get_live_options() -> ClaudeAgentOptions {
    let mut options = ClaudeAgentOptions::default();

    // Pass through critical env vars if they exist in current process
    let vars = vec![
        "ANTHROPIC_AUTH_TOKEN",
        "ANTHROPIC_BASE_URL",
        "ANTHROPIC_MODEL",
        "ANTHROPIC_DEFAULT_HAIKU_MODEL",
        "ANTHROPIC_DEFAULT_SONNET_MODEL",
        "ANTHROPIC_DEFAULT_OPUS_MODEL",
    ];

    for var in vars {
        if let Ok(val) = env::var(var) {
            options.env.insert(var.to_string(), val);
        }
    }

    // We can also set a custom CLI path if provided
    if let Ok(path) = env::var("CLAUDE_CLI_PATH") {
        options.cli_path = Some(std::path::PathBuf::from(path));
    }

    options
}

#[tokio::test]
#[ignore]
async fn test_live_connectivity() {
    // Initialize tracing if possible (ignores error if already init)
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    // 1. Setup
    let options = get_live_options();
    println!("Connecting with options keys: {:?}", options.env.keys());

    let mut client = ClaudeAgentClient::new(Some(options));

    match client.connect().await {
        Ok(_) => println!("Successfully connected to Claude process"),
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            // Verify if cli exists
            if let Ok(path) = which::which("claude") {
                eprintln!("Claude CLI found at: {:?}", path);
            } else {
                eprintln!("Claude CLI NOT found in PATH");
            }
            panic!("Connection failed");
        },
    }

    // 3. Simple Query
    let prompt = "Reply with exactly the word 'PONG'";
    println!("Sending query: '{}'", prompt);

    {
        // Scope for the query stream
        // Increase timeout to 300s (5 minutes) for reliability with slow tools
        let result = timeout(Duration::from_secs(300), client.query(prompt)).await;

        match result {
            Ok(query_res) => {
                match query_res {
                    Ok(mut stream) => {
                        let mut found_pong = false;
                        println!("Stream started, waiting for messages...");

                        while let Some(msg_res) = stream.next().await {
                            match msg_res {
                                Ok(Message::Assistant(m)) => {
                                    println!("Assistant: {:?}", m);
                                    found_pong = true;
                                },
                                Ok(Message::Result(r)) => {
                                    if !found_pong {
                                        if let Some(res_text) = r.result {
                                            if res_text.to_lowercase().contains("pong") {
                                                found_pong = true;
                                                println!("Found PONG in Result message");
                                            }
                                        }
                                    }
                                    // Result message indicates end of turn
                                    break;
                                },
                                Ok(other) => println!("Received: {:?}", other),
                                Err(e) => println!("Stream error: {}", e),
                            }
                        }

                        // We check found_pong
                        assert!(found_pong, "Did not receive PONG response from live CLI");
                    },
                    Err(e) => panic!("Query failed to start: {}", e),
                }
            },
            Err(_) => panic!("Timed out waiting for response. Check auth/CLI status."),
        }
    } // stream dropped here

    // 4. Cleanup
    let _ = client.disconnect().await;
}

#[tokio::test]
#[ignore]
async fn test_live_conversation_memory() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let options = get_live_options();
    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await.expect("Connect failed");

    // Turn 1
    let prompt1 = "My name is IntegrationTestUser. Remember this.";
    println!("Turn 1: {}", prompt1);
    {
        let mut stream1 = client.query(prompt1).await.expect("Query 1 failed");
        while let Some(res) = stream1.next().await {
            match res {
                Ok(m) => {
                    println!("Turn 1 response: {:?}", m);
                    if let Message::Result(_) = m {
                        break;
                    }
                },
                Err(e) => eprintln!("Turn 1 error: {}", e),
            }
        }
    } // stream1 dropped

    // Turn 2
    let prompt2 = "What is my name? Reply with just the name.";
    println!("Turn 2: {}", prompt2);
    {
        let mut stream2 = client.query(prompt2).await.expect("Query 2 failed");
        let mut buffer = String::new();
        while let Some(res) = stream2.next().await {
            match res {
                Ok(Message::Assistant(m)) => {
                    use claude_agent_types::message::ContentBlock;
                    for c in m.content {
                        if let ContentBlock::Text(t) = c {
                            buffer.push_str(&t.text);
                        }
                    }
                },
                Ok(Message::Result(r)) => {
                    if let Some(res) = r.result {
                        buffer.push_str(&res);
                    }
                    break;
                },
                _ => {},
            }
        }
        println!("Memory Test Response: {}", buffer);
        assert!(buffer.contains("IntegrationTestUser"), "Agent failed to remember context");
    } // stream2 dropped

    client.disconnect().await.expect("Disconnect failed");
}
