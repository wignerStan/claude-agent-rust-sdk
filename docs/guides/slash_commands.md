# Slash Commands in the SDK

Slash commands provide a quick way to trigger system-level actions without writing long natural language prompts. Both the primary agent and the CLI support various commands.

## Common Slash Commands

| Command | Description |
|---------|-------------|
| `/compact` | Summarizes history to save on token usage. |
| `/clear` | Clears the current conversation history. |
| `/reset` | Resets the agent and tools to their initial state. |
| `/config` | Displays or modifies current configuration. |

## Using Commands in the SDK

You can send slash commands just like any other prompt.

```rust
let mut stream = client.query("/compact").await?;
```

## Discovery

You can discover available commands by inspecting the initialization data returned during the connection phase.

```rust
// Access the underlying agent's initialization result (placeholder for future API)
let info = client.get_server_info().await;
if let Some(data) = info {
    println!("Available commands: {:?}", data["commands"]);
}
```

## Creating Custom Commands

Custom slash commands are typically defined at the CLI level or via plugins. The SDK allows you to interact with these commands once they are registered in the environment.

> [!NOTE]
> Some slash commands may terminate the current stream and start a new turn.
