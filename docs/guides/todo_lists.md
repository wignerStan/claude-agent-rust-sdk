# Todo Lists

The Claude Agent can maintain a task list (to-do list) to track progress during long-running or complex operations.

## How the Agent Uses Todo Lists

When the agent identifies multiple steps required to solve a problem, it may create a todo list. This helps keep the context focused and ensures no steps are missed.

### Standard Todo Tools

If Enabled, the agent has access to tools like:
- `add_todo`: Add a new task.
- `update_todo`: Mark a task as complete or in progress.
- `list_todos`: Show the current status of all tasks.

## Interacting with Todos in the SDK

Todo updates appear in the message stream as `ToolUse` and `ToolResult` blocks. You can also see the summarized state in the `StructuredOutput` of a `ResultMessage`.

## Persistence

Todo lists are typically scoped to the current `session_id`. If you continue a session, the previous todo list will be available to the agent.

> [!TIP]
> Use the [Hook System](hooks.md) to intercept todo updates and display a progress bar in your UI.
