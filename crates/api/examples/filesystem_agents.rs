//! Filesystem agents example - Working with filesystem tools.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions {
        cwd: Some(std::path::PathBuf::from(".")),
        allowed_tools: vec![
            "Read".to_string(),
            "Write".to_string(),
            "Edit".to_string(),
            "ListDir".to_string(),
        ],
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await?;

    {
        let mut stream = client.query("List the files in the current directory").await?;
        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                println!("{:?}", msg);
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}
