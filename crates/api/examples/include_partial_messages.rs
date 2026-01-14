//! Include partial messages example - Handling complex message structures.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::message::{ContentBlock, Message};
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions {
        // Enable partial message streaming
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await?;

    {
        let mut stream = client.query("Write a haiku about Rust").await?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(Message::Assistant(msg)) => {
                    println!("Assistant message with {} content blocks:", msg.content.len());
                    for (i, block) in msg.content.iter().enumerate() {
                        match block {
                            ContentBlock::Text(t) => println!("  [{}] Text: {}", i, t.text),
                            ContentBlock::ToolUse(tu) => println!("  [{}] ToolUse: {}", i, tu.name),
                            ContentBlock::ToolResult(tr) => {
                                println!("  [{}] ToolResult: {}", i, tr.tool_use_id)
                            },
                            _ => {},
                        }
                    }
                },
                Ok(Message::User(msg)) => {
                    println!("User message: {:?}", msg);
                },
                Ok(Message::Result(r)) => {
                    println!("Result: {:?}", r);
                },
                Ok(Message::System(s)) => {
                    println!("System: {:?}", s);
                },
                Err(e) => eprintln!("Error: {}", e),
                _ => {},
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}
