//! Quick start example - Basic query demonstration.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default options
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));

    // Connect to Claude Code CLI
    client.connect().await?;

    // Send a query and stream results
    {
        let mut stream = client.query("What is 2 + 2?").await?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => println!("{:?}", message),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
