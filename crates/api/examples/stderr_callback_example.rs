//! Stderr callback example - Handling stderr output.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // In a full implementation, ClaudeAgentOptions would support
    // a stderr callback for capturing CLI stderr output.
    //
    // Example:
    // options.stderr_callback = Some(Arc::new(|line: &str| {
    //     eprintln!("[CLI STDERR] {}", line);
    // }));

    let options = ClaudeAgentOptions::default();

    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await?;

    println!("Note: stderr callbacks are captured during transport read");

    {
        let mut stream = client.query("Run a command that might produce stderr").await?;
        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                println!("{:?}", msg);
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}
