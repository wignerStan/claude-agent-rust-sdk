# File Checkpointing and Rewind

File Checkpointing is a powerful safety feature that tracks every modification made by the agent. If the agent makes a mistake, you can programmatically "rewind" files to a known good state.

## Enabling Checkpointing

Checkpointing must be explicitly enabled in `ClaudeAgentOptions`.

```rust
let options = ClaudeAgentOptions {
    enable_file_checkpointing: true,
    ..Default::default()
};
```

## How it works

1. **Tracking**: The CLI saves a snapshot of files before modifications.
2. **Identifying**: Each `UserMessage` in the stream is assigned a unique `uuid`. This ID serves as the checkpoint identifier.
3. **Rewinding**: Call `rewind_files(uuid)` to revert all changes made *after* that message.

## Example: Implement and Rewind

```rust
async fn run_and_maybe_undo() {
    let mut client = ClaudeAgentClient::new(Some(options));
    client.connect().await.unwrap();

    // 1. Get a checkpoint ID
    let mut checkpoint_id = String::new();
    client.query("Refactor main.rs").await.unwrap();

    // In practice, capture the uuid from the UserMessage in the stream
    // checkpoint_id = ...

    // 2. If the refactor is bad, undo it
    client.rewind_files(&checkpoint_id).await.unwrap();
}
```

## Best Practices

- **Frequent Checkpoints**: If doing autonomous work, save checkpoint IDs after every successful sub-task.
- **Verification**: Always verify the state of your files after a rewind operation.
- **Git Interaction**: Checkpointing works alongside git but is independent. It's often safer than `git reset` for surgical undos of agent actions.
