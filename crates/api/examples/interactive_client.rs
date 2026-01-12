//! Interactive client example.
//!
//! Run with: cargo run --example interactive_client

use claude_agent_api::{ClaudeAgentClient, ClaudeAgentOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting interactive session...");

    let options = ClaudeAgentOptions {
        cwd: Some(std::env::current_dir()?),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));

    // Connect to Claude
    client.connect().await?;
    println!("Connected! Session ID: {:?}", client.session_id());

    // Send a query (Turn 1)
    println!("Sending turn 1...");
    {
        let mut stream = client.query("Hello! Can you tell me what 2+2 is?").await?;

        while let Some(msg) = stream.next().await {
            println!("Turn 1 response: {:?}", msg);
        }
    }

    // Send another query (Turn 2) - Context is maintained
    println!("Sending turn 2...");
    {
        let mut stream = client.query("Now, can you subtract 1 from that?").await?;

        while let Some(msg) = stream.next().await {
            println!("Turn 2 response: {:?}", msg);
        }
    }

    // Disconnect
    client.disconnect().await?;
    println!("\nDisconnected!");

    Ok(())
}
