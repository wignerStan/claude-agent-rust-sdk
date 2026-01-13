# Streaming Input

The Claude Agent Rust SDK provides a streaming API for receiving real-time responses from the assistant. This is essential for building interactive applications where low perceived latency is important.

## Using the Streaming API

When you call `ClaudeAgentClient::query()`, it returns a `Result<impl Stream<Item = Result<Message, ClaudeAgentError>>, ClaudeAgentError>`.

### Basic Example

```rust
use claude_agent_api::ClaudeAgentClient;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClaudeAgentClient::default();
    client.connect().await?;

    let mut stream = client.query("Write a short story about a robot.").await?;

    while let Some(item) = stream.next().await {
        match item {
            Ok(message) => {
                // Handle different message types (Thinking, Text, etc.)
                println!("Received message: {:?}", message);
            }
            Err(e) => {
                eprintln!("Error in stream: {}", e);
            }
        }
    }

    Ok(())
}
```

## Message Types in Streams

The stream can yield several types of messages:

| Message Type | Description |
|--------------|-------------|
| `Thinking`   | Internal chain-of-thought blocks (visible if enabled). |
| `Assistant`  | Partial or complete assistant text content. |
| `ToolUse`    | Request to execute a tool. |
| `Result`     | Final result metadata for a turn. |

## Partial Messages

By default, the SDK handles the assembly of partial messages. If you need to see partial tokens as they arrive, ensure your `ClaudeAgentOptions` are configured accordingly.

> [!TIP]
> Use `tokio-stream` or `futures` crate to handle the stream asynchronously.
