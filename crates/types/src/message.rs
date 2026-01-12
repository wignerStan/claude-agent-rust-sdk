use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text(TextBlock),
    #[serde(rename = "thinking")]
    Thinking(ThinkingBlock),
    #[serde(rename = "tool_use")]
    ToolUse(ToolUseBlock),
    #[serde(rename = "tool_result")]
    ToolResult(ToolResultBlock),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    pub thinking: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlock {
    #[serde(rename = "tool_use_id")]
    pub tool_use_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolResultContent>,
    #[serde(rename = "is_error", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<serde_json::Value>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Message {
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
    #[serde(rename = "system")]
    System(SystemMessage),
    #[serde(rename = "result")]
    Result(ResultMessage),
    #[serde(rename = "stream_event")]
    StreamEvent(StreamEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "WireUserMessage", into = "WireUserMessage")]
pub struct UserMessage {
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(rename = "parent_tool_use_id", skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WireUserMessage {
    #[serde(default)]
    message: UserMessageBody,
    #[serde(default)]
    uuid: Option<String>,
    #[serde(default)]
    parent_tool_use_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct UserMessageBody {
    content: MessageContent,
}

impl From<WireUserMessage> for UserMessage {
    fn from(wire: WireUserMessage) -> Self {
        Self {
            content: wire.message.content,
            uuid: wire.uuid,
            parent_tool_use_id: wire.parent_tool_use_id,
        }
    }
}

impl From<UserMessage> for WireUserMessage {
    fn from(msg: UserMessage) -> Self {
        Self {
            message: UserMessageBody {
                content: msg.content,
            },
            uuid: msg.uuid,
            parent_tool_use_id: msg.parent_tool_use_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

impl Default for MessageContent {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "WireAssistantMessage", into = "WireAssistantMessage")]
pub struct AssistantMessage {
    pub content: Vec<ContentBlock>,
    pub model: String,
    #[serde(rename = "parent_tool_use_id", skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AssistantMessageError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WireAssistantMessage {
    #[serde(default)]
    message: AssistantMessageBody,
    #[serde(default)]
    parent_tool_use_id: Option<String>,
    #[serde(default)]
    error: Option<AssistantMessageError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AssistantMessageBody {
    #[serde(default)]
    content: Vec<ContentBlock>,
    #[serde(default)]
    model: String,
}

impl From<WireAssistantMessage> for AssistantMessage {
    fn from(wire: WireAssistantMessage) -> Self {
        Self {
            content: wire.message.content,
            model: wire.message.model,
            parent_tool_use_id: wire.parent_tool_use_id,
            error: wire.error,
        }
    }
}

impl From<AssistantMessage> for WireAssistantMessage {
    fn from(msg: AssistantMessage) -> Self {
        Self {
            message: AssistantMessageBody {
                content: msg.content,
                model: msg.model,
            },
            parent_tool_use_id: msg.parent_tool_use_id,
            error: msg.error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssistantMessageError {
    AuthenticationFailed,
    BillingError,
    RateLimit,
    InvalidRequest,
    ServerError,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMessage {
    pub subtype: String,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub is_error: bool,
    pub num_turns: u32,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_output: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub uuid: String,
    pub session_id: String,
    pub event: serde_json::Value,
    #[serde(rename = "parent_tool_use_id", skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}
