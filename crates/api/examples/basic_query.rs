//! Basic query example.
//!
//! Run with: cargo run --example basic_query

use claude_agent_api::{query, ClaudeAgentOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sending query to Claude...");

    let options = ClaudeAgentOptions { cwd: Some(std::env::current_dir()?), ..Default::default() };

    let mut stream = query("What is 2+2? Please give a brief answer.", Some(options)).await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                println!("Received: {:?}", message);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            },
        }
    }

    println!("Done!");
    Ok(())
}
