//! System prompt example - Using custom system prompts.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions {
        system_prompt: Some(claude_agent_types::config::SystemPromptConfig::Text(
            "You are a helpful coding assistant. Be concise.".to_string(),
        )),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await?;

    {
        let mut stream = client.query("Explain Rust ownership in one sentence.").await?;

        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                println!("{:?}", msg);
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}
