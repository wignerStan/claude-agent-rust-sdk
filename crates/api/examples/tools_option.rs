//! Tools option example - Configuring allowed tools.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions {
        // Restrict to specific tools
        allowed_tools: vec!["Read".to_string(), "Bash".to_string()],
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await?;

    {
        let mut stream = client.query("List files in the current directory").await?;
        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                println!("{:?}", msg);
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}
