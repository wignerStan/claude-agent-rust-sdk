# Subagents in the SDK

Subagents allow you to delegate complex tasks to specialized agents with their own prompts, tools, and constraints. This modular approach improves performance and reliability for multi-step workflows.

## Defining Subagents

Subagents are defined in `ClaudeAgentOptions` using the `agents` field.

```rust
use claude_agent_types::{ClaudeAgentOptions, AgentDefinition};
use std::collections::HashMap;

let mut agents = HashMap::new();
agents.insert(
    "code-reviewer".to_string(),
    AgentDefinition {
        description: "Reviews code for best practices".to_string(),
        prompt: "You are a senior developer. Review code for bugs and style.".to_string(),
        tools: Some(vec!["Read".to_string(), "Grep".to_string()]),
        model: Some("sonnet".to_string()),
    },
);

let options = ClaudeAgentOptions {
    agents: Some(agents),
    ..Default::default()
};
```

## Using Subagents

Once registered, the primary Claude agent can "call" these subagents as if they were tools.

**Prompt Example:**
> "Use the code-reviewer agent to analyze the changes in `src/main.rs`."

## Filesystem-Based Agents

The SDK also supports loading agents defined in `.claude/agents/*.md` files. This is enabled by setting `setting_sources` to include `Project`.

```rust
let options = ClaudeAgentOptions {
    setting_sources: Some(vec![SettingSource::Project]),
    cwd: Some(std::path::PathBuf::from(".")),
    ..Default::default()
};
```

## Monitoring Subagents

When a subagent is active:
1. It has its own conversation history.
2. Messages received in the stream may have a `parent_tool_use_id` corresponding to the subagent call.
3. You can intercept subagent activity using hooks.
