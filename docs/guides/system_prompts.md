# Modifying System Prompts

System prompts are the primary way to define the agent's persona, expertise, and constraints. The Rust SDK offers several ways to configure these prompts.

## System Prompt Config

Configuration is done via the `SystemPromptConfig` enum within `ClaudeAgentOptions`.

### Using a Custom String

This is the most common way to define a prompt.

```rust
use claude_agent_types::{ClaudeAgentOptions, SystemPromptConfig};

let options = ClaudeAgentOptions {
    system_prompt: Some(SystemPromptConfig::Text("You are a specialized code reviewer.".to_string())),
    ..Default::default()
};
```

### Using Upstream Presets

The `claude` CLI often comes with built-in presets (like 'claude' or 'agent'). You can reference these directly.

```rust
let options = ClaudeAgentOptions {
    system_prompt: Some(SystemPromptConfig::Preset("code-bot".to_string())),
    ..Default::default()
};
```

### Appending to the Existing Prompt

If you want to keep the default persona but add specific instructions:

```rust
let options = ClaudeAgentOptions {
    system_prompt: Some(SystemPromptConfig::Append(" Always use async/await for I/O.".to_string())),
    ..Default::default()
};
```

## Effective Prompting Tips

| Requirement | Recommended Approach |
|-------------|----------------------|
| **Formatting** | Explicitly describe preferred output formats (e.g., "Always return JSON"). |
| **Safety** | Include negative constraints (e.g., "Do not delete files without asking"). |
| **Context** | Provide relevant background about the project or user environment. |

> [!TIP]
> Changes to the system prompt are most effective when starting a new session.
