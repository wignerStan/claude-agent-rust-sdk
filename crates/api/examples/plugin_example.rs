//! Plugin example - Extending agent functionality.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;

/// A simple plugin trait for extending agent functionality.
trait Plugin {
    fn name(&self) -> &str;
    fn on_query(&self, prompt: &str) -> String;
}

/// Example plugin that adds a prefix to queries.
struct PrefixPlugin {
    prefix: String,
}

impl Plugin for PrefixPlugin {
    fn name(&self) -> &str {
        "PrefixPlugin"
    }

    fn on_query(&self, prompt: &str) -> String {
        format!("{} {}", self.prefix, prompt)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a plugin
    let plugin = PrefixPlugin { prefix: "[IMPORTANT]".to_string() };

    println!("Plugin '{}' loaded", plugin.name());

    let original_query = "Write a hello world program";
    let modified_query = plugin.on_query(original_query);

    println!("Original: {}", original_query);
    println!("Modified: {}", modified_query);

    // Use with client
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));
    client.connect().await?;

    // The plugin would modify queries before sending
    // client.query(&modified_query).await?;

    client.disconnect().await?;
    Ok(())
}
