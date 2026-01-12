//! Agents example - Working with agent configurations.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create multiple agent configurations
    let coding_agent = ClaudeAgentOptions {
        system_prompt: Some(claude_agent_types::config::SystemPromptConfig::Text(
            "You are an expert Rust programmer.".to_string(),
        )),
        allowed_tools: vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()],
        ..Default::default()
    };

    let review_agent = ClaudeAgentOptions {
        system_prompt: Some(claude_agent_types::config::SystemPromptConfig::Text(
            "You are a code reviewer. Be thorough and constructive.".to_string(),
        )),
        allowed_tools: vec!["Read".to_string()],
        ..Default::default()
    };

    println!("Coding agent config: {:?}", coding_agent);
    println!("Review agent config: {:?}", review_agent);

    // Use the coding agent
    let mut client = ClaudeAgentClient::new(Some(coding_agent));
    client.connect().await?;

    {
        let mut stream = client.query("What tools do you have access to?").await?;
        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                println!("{:?}", msg);
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}
