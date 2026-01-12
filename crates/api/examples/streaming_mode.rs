//! Streaming mode example - Iterating over message stream.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::message::{ContentBlock, Message};
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));
    client.connect().await?;

    {
        let mut stream = client.query("Count from 1 to 5").await?;

        println!("Streaming response:");
        while let Some(result) = stream.next().await {
            match result {
                Ok(Message::Assistant(msg)) => {
                    for block in &msg.content {
                        if let ContentBlock::Text(t) = block {
                            print!("{}", t.text);
                        }
                    }
                }
                Ok(_) => {} // Handle other message types
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        println!();
    }

    client.disconnect().await?;
    Ok(())
}
