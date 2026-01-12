# Agent Skills in the SDK

"Skills" in the Claude Agent ecosystem represent the set of capabilities available to the agent, primarily implemented as tools and MCP servers.

## Managing Skills

Skills are configured via `ClaudeAgentOptions`. You can enable built-in skills or add external ones.

### Built-in Skills (Claude Code)

By default, Claude Code provides its own high-performance tools for:
- Filesystem access (Read, Write, Edit)
- Search (Grep, Glob)
- Environment (Bash, Ls)

You can restrict these using `allowed_tools` or `disallowed_tools`.

```rust
let options = ClaudeAgentOptions {
    allowed_tools: vec!["Read".to_string(), "Bash".to_string()],
    ..Default::default()
};
```

### Expanding Skills with MCP

The Model Context Protocol (MCP) is the standard way to add new "skills" to your agent.

- [MCP Guide](mcp.md): Learn how to connect external servers.
- [Custom Tools Guide](custom_tools.md): Learn how to write your own skills in Rust.

## Skill Discovery

Claude automatically discovers available skills during initialization. When building an interactive UI, you may want to display these skills to the user to indicate what the agent can do.

## Permissions as Guards

Since many skills involve sensitive actions (like running shell commands), they are guarded by the [Permission System](handling_permissions.md).
