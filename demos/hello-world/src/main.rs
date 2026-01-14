//! Hello World - Basic Claude Agent SDK demonstration.
//!
//! This demo showcases:
//! - Basic SDK usage with `ClaudeAgentClient`
//! - Message streaming and content extraction
//! - Tool whitelisting
//! - Custom working directory
//!
//! Migrated from: claude-agent-sdk-demos/hello-world/hello-world.ts

use anyhow::{Context, Result};
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("claude_agent=debug".parse().unwrap()),
        )
        .init();

    let agent_dir = std::env::current_dir()
        .context("Failed to get current directory")?
        .join("agent");

    let options = ClaudeAgentOptions {
        max_turns: Some(100),
        cwd: Some(agent_dir.clone()),
        model: std::env::var("ANTHROPIC_MODEL").ok(),
        allowed_tools: vec![
            "Task".to_string(),
            "Bash".to_string(),
            "Glob".to_string(),
            "Grep".to_string(),
            "LS".to_string(),
            "ExitPlanMode".to_string(),
            "Read".to_string(),
            "Edit".to_string(),
            "MultiEdit".to_string(),
            "Write".to_string(),
            "NotebookEdit".to_string(),
            "WebFetch".to_string(),
            "TodoWrite".to_string(),
            "WebSearch".to_string(),
            "BashOutput".to_string(),
            "KillBash".to_string(),
        ],
        ..Default::default()
    };

    let prompt = "Hello, Claude! Please introduce yourself in one sentence.";

    println!("Sending query: {}", prompt);
    println!("{}", "-".repeat(50));

    let mut client = ClaudeAgentClient::new(Some(options));
    client
        .connect()
        .await
        .context("Failed to connect to Claude Code")?;

    {
        let mut stream = client
            .query(prompt)
            .await
            .context("Failed to create query stream")?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    if let claude_agent_types::Message::Assistant(msg) = message {
                        for block in msg.content {
                            if let claude_agent_types::message::ContentBlock::Text(text_block) =
                                block
                            {
                                println!("Claude says: {}", text_block.text);
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    client
        .disconnect()
        .await
        .context("Failed to disconnect from Claude Code")?;

    println!("{}", "-".repeat(50));
    println!("Query completed.");

    Ok(())
}
