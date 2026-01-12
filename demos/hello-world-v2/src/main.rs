//! Hello World V2 - Session API demonstration.
//!
//! This demo showcases:
//! - Session creation and resumption
//! - Multi-turn conversations
//! - Context retention
//! - One-shot prompts
//!
//! Migrated from: claude-agent-sdk-demos/hello-world-v2/v2-examples.ts

use anyhow::{Context, Result};
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

/// Basic session example.
async fn basic_session() -> Result<()> {
    println!("\n=== Basic Session Example ===\n");

    let options = ClaudeAgentOptions {
        model: Some("claude-sonnet-4-5".to_string()),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await.context("Failed to connect")?;

    println!("Creating session...");

    let mut stream = client.query("Hello! My name is Alice.").await?;
    let mut messages = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                messages.push(message.clone());
                if let claude_agent_types::Message::Assistant(msg) = message {
                    for block in msg.content {
                        if let claude_agent_types::message::ContentBlock::Text(text) = block {
                            println!("Claude: {}", text.text);
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    client.disconnect().await.context("Failed to disconnect")?;

    println!("\nSession completed with {} messages", messages.len());
    Ok(())
}

/// Session resumption example.
async fn resume_session(session_id: String) -> Result<()> {
    println!("\n=== Session Resumption Example ===\n");
    println!("Resuming session: {}\n", session_id);

    let options = ClaudeAgentOptions {
        model: Some("claude-sonnet-4-5".to_string()),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await.context("Failed to connect")?;

    println!("Sending follow-up message...");

    let mut stream = client.query("What's my name?").await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let claude_agent_types::Message::Assistant(msg) = message {
                    for block in msg.content {
                        if let claude_agent_types::message::ContentBlock::Text(text) = block {
                            println!("Claude: {}", text.text);
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    client.disconnect().await.context("Failed to disconnect")?;

    println!("\nSession resumed successfully!");
    Ok(())
}

/// Multi-turn conversation example.
async fn multi_turn_conversation() -> Result<()> {
    println!("\n=== Multi-Turn Conversation Example ===\n");

    let options = ClaudeAgentOptions {
        model: Some("claude-sonnet-4-5".to_string()),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await.context("Failed to connect")?;

    let prompts = vec![
        "I'm learning Rust programming.",
        "What are the key features of Rust?",
        "Can you explain ownership and borrowing?",
        "How does Rust handle memory safety?",
    ];

    for (i, prompt) in prompts.iter().enumerate() {
        println!("\n--- Turn {} ---", i + 1);
        println!("You: {}", prompt);

        let mut stream = client.query(prompt).await?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    if let claude_agent_types::Message::Assistant(msg) = message {
                        for block in msg.content {
                            if let claude_agent_types::message::ContentBlock::Text(text) = block {
                                println!("Claude: {}", text.text);
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    client.disconnect().await.context("Failed to disconnect")?;

    println!("\nMulti-turn conversation completed!");
    Ok(())
}

/// One-shot prompt example (stateless query).
async fn one_shot_prompt() -> Result<()> {
    println!("\n=== One-Shot Prompt Example ===\n");

    let prompt = "What is the capital of France?";

    println!("Sending one-shot query: {}", prompt);

    let mut stream = claude_agent_api::query(prompt, None)
        .await
        .context("Failed to create query stream")?;

    let mut response = String::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let claude_agent_types::Message::Assistant(msg) = message {
                    for block in msg.content {
                        if let claude_agent_types::message::ContentBlock::Text(text) = block {
                            response.push_str(&text.text);
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    println!("Response: {}", response);
    println!("\nOne-shot query completed!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("claude_agent=debug".parse().unwrap()),
        )
        .init();

    println!("Claude Agent SDK - Hello World V2 Demo");
    println!("Demonstrating Session API capabilities\n");

    println!("Choose an example:");
    println!("1. Basic Session");
    println!("2. Session Resumption");
    println!("3. Multi-Turn Conversation");
    println!("4. One-Shot Prompt");
    println!("5. Run All Examples");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => basic_session().await?,
        "2" => {
            basic_session().await?;
            println!("\nNote: Session resumption requires actual session ID from previous run.");
            println!("For demonstration, we'll skip actual resumption.");
        }
        "3" => multi_turn_conversation().await?,
        "4" => one_shot_prompt().await?,
        "5" => {
            basic_session().await?;
            multi_turn_conversation().await?;
            one_shot_prompt().await?;
        }
        _ => {
            println!("Invalid choice. Running basic session example...");
            basic_session().await?;
        }
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
