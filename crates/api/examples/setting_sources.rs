//! Setting sources example - Loading config from environment/file.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load options from environment
    let model = env::var("CLAUDE_MODEL").ok();
    let cwd = env::var("CLAUDE_CWD").ok().map(std::path::PathBuf::from);

    let options = ClaudeAgentOptions {
        model,
        cwd,
        ..Default::default()
    };

    println!("Using options: {:?}", options);

    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await?;

    {
        let mut stream = client.query("What model are you?").await?;
        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                println!("{:?}", msg);
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}
