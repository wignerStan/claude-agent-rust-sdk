//! Core logic for Claude Agent SDK.

pub mod agent;
pub mod control;
pub mod hooks;
pub mod permissions;
pub mod server_info;
pub mod session;
pub mod streaming;

pub use agent::ClaudeAgent;
pub use control::{ControlProtocol, ControlRequest, ControlRequestType, ControlResponse};
pub use hooks::{HookCallback, HookContext, HookInput, HookOutput, HookRegistry};
pub use permissions::{PermissionCallback, PermissionHandler};
pub use server_info::{
    ContextUsageCategory, ContextUsageResponse, McpConnectionStatus, McpServerStatus,
    McpStatusResponse, McpToolInfo, ServerInfo,
};
pub use session::{Session, SessionManager};
pub use streaming::{message_channel, MessageReceiver, MessageSender};
