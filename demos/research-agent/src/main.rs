//! Research Agent - Multi-agent research system demonstration.
//!
//! This demo showcases:
//! - Multi-agent orchestration (lead, researcher, data-analyst, report-writer)
//! - Hook system for subagent tracking
//! - File-based coordination (research_notes/, charts/, reports/)
//! - Transcript and tool call logging
//!
//! Migrated from: claude-agent-sdk-demos/research-agent/research_agent/agent.py

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::{
    config::{AgentDefinition, PermissionMode, SystemPromptConfig},
    message::{ContentBlock, Message},
    ClaudeAgentOptions,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};

/// Agent type for tracking subagent calls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SubagentType {
    Lead,
    Researcher,
    DataAnalyst,
    ReportWriter,
}

/// Record of a tool call for tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub timestamp: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_use_id: String,
    pub subagent_type: SubagentType,
    pub parent_tool_use_id: Option<String>,
    pub tool_output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Session for tracking subagent activities.
#[derive(Debug, Clone)]
pub struct SubagentSession {
    pub subagent_id: String,
    pub subagent_type: SubagentType,
    pub parent_tool_use_id: Option<String>,
    pub start_time: DateTime<Utc>,
}

/// Tracks subagent activities and logs tool calls.
pub struct SubagentTracker {
    pub sessions: Arc<Mutex<HashMap<String, SubagentSession>>>,
    pub current_parent_id: Arc<Mutex<Option<String>>>,
    pub subagent_counters: Arc<Mutex<HashMap<String, u32>>>,
    pub tool_log_file: Arc<Mutex<File>>,
    pub transcript_file: Arc<Mutex<File>>,
    pub session_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl SubagentTracker {
    /// Create a new tracker with session and logs directories.
    pub fn new(session_dir: PathBuf) -> Result<Self> {
        let logs_dir = session_dir.join("logs");
        std::fs::create_dir_all(&logs_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let tool_log_path = logs_dir.join(format!("tool_calls_{}.jsonl", timestamp));
        let transcript_path = logs_dir.join(format!("transcript_{}.txt", timestamp));

        let tool_log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&tool_log_path)
            .context("Failed to create tool log file")?;

        let transcript_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&transcript_path)
            .context("Failed to create transcript file")?;

        Ok(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            current_parent_id: Arc::new(Mutex::new(None)),
            subagent_counters: Arc::new(Mutex::new(HashMap::new())),
            tool_log_file: Arc::new(Mutex::new(tool_log_file)),
            transcript_file: Arc::new(Mutex::new(transcript_file)),
            session_dir,
            logs_dir,
        })
    }

    /// Log a tool call to the JSONL file.
    #[allow(dead_code)]
    fn log_tool_call(&self, record: &ToolCallRecord) {
        if let Ok(json) = serde_json::to_string(record) {
            if let Ok(mut file) = self.tool_log_file.lock() {
                let _ = writeln!(file, "{}", json);
            }
        }
    }

    /// Write to the transcript file.
    fn write_transcript(&self, text: &str) {
        if let Ok(mut file) = self.transcript_file.lock() {
            let _ = writeln!(file, "{}", text);
        }
    }

    /// Get the current subagent type based on parent tool use.
    fn get_current_subagent_type(&self) -> SubagentType {
        let parent_id = self.current_parent_id.lock().unwrap().clone();

        if parent_id.is_none() {
            return SubagentType::Lead;
        }

        let parent_tool_use_id = parent_id.unwrap();
        let sessions = self.sessions.lock().unwrap();

        if let Some(session) = sessions.get(&parent_tool_use_id) {
            match session.subagent_type {
                SubagentType::Lead => SubagentType::Researcher,
                SubagentType::Researcher => SubagentType::DataAnalyst,
                SubagentType::DataAnalyst => SubagentType::ReportWriter,
                SubagentType::ReportWriter => SubagentType::ReportWriter,
            }
        } else {
            SubagentType::Lead
        }
    }

    /// Log a subagent spawn event.
    pub fn log_subagent_spawn(&self, subagent_id: &str, tool_name: &str) {
        let current_subagent = self.get_current_subagent_type();
        let _subagent_type_str = format!("{:?}", current_subagent).to_uppercase();
        let counter_key = format!("{:?}", current_subagent);
        let mut counters = self.subagent_counters.lock().unwrap();
        let counter = counters.entry(counter_key.clone()).or_insert(0);
        *counter += 1;

        self.write_transcript(&format!("🚀 SUBAGENT SPAWNED: {}", subagent_id));
        self.write_transcript(&format!("📝 Tool Use: {} -> {}", subagent_id, tool_name));
    }

    /// Log a subagent completion event.
    #[allow(dead_code)]
    pub fn log_subagent_complete(&self, subagent_id: &str) {
        self.write_transcript(&format!("🎉 SUBAGENT COMPLETED: {}", subagent_id));
    }

    /// Close the tracker and cleanup.
    pub fn close(&mut self) {
        if let Ok(mut file) = self.tool_log_file.lock() {
            let _ = file.flush();
        }
        if let Ok(mut file) = self.transcript_file.lock() {
            let _ = file.flush();
        }
    }
}

/// Agent definitions for specialized research tasks.
pub fn create_agents() -> HashMap<String, AgentDefinition> {
    let mut agents = HashMap::new();

    agents.insert(
        "researcher".to_string(),
        AgentDefinition {
            description: "Gather research information using web search".to_string(),
            prompt: include_str!("../prompts/researcher.txt").to_string(),
            tools: Some(vec![
                "WebSearch".to_string(),
                "WebFetch".to_string(),
                "Write".to_string(),
                "Read".to_string(),
            ]),
            model: std::env::var("ANTHROPIC_MODEL").ok(),
        },
    );

    agents.insert(
        "data-analyst".to_string(),
        AgentDefinition {
            description: "Generate charts and data analysis".to_string(),
            prompt: include_str!("../prompts/data_analyst.txt").to_string(),
            tools: Some(vec![
                "Glob".to_string(),
                "Read".to_string(),
                "Bash".to_string(),
                "Write".to_string(),
            ]),
            model: std::env::var("ANTHROPIC_MODEL").ok(),
        },
    );

    agents.insert(
        "report-writer".to_string(),
        AgentDefinition {
            description: "Create PDF reports".to_string(),
            prompt: include_str!("../prompts/report_writer.txt").to_string(),
            tools: Some(vec![
                "Skill".to_string(),
                "Write".to_string(),
                "Glob".to_string(),
                "Read".to_string(),
                "Bash".to_string(),
            ]),
            model: std::env::var("ANTHROPIC_MODEL").ok(),
        },
    );

    agents
}

/// Process assistant messages and log to transcript.
pub fn process_assistant_message(
    message: &Message,
    tracker: &SubagentTracker,
    _transcript: &mut impl Write,
) -> Result<()> {
    if let Message::Assistant(msg) = message {
        for block in &msg.content {
            match block {
                ContentBlock::ToolUse(tool_use) => {
                    if tool_use.name == "Task" {
                        let task_input = tool_use.input.get("input");
                        if let Some(input) = task_input {
                            let input_str = input.as_str().unwrap_or("");
                            if input_str.starts_with("Use the ") && input_str.contains(" agent") {
                                let agent_name = input_str
                                    .split_whitespace()
                                    .nth(2)
                                    .unwrap_or("")
                                    .to_string();
                                tracker.write_transcript(&format!(
                                    "\n📋 AGENT CALLED: {}",
                                    agent_name.to_uppercase()
                                ));
                            }
                        }
                    } else if tool_use.name == "Bash" {
                        let command = tool_use.input.get("command");
                        if let Some(cmd) = command {
                            tracker.write_transcript(&format!("🔧 Bash: {}", cmd));
                        }
                    } else if tool_use.name == "WebSearch" {
                        let query = tool_use.input.get("query");
                        if let Some(q) = query {
                            tracker.write_transcript(&format!("🔍 WebSearch: {}", q));
                        }
                    } else {
                        tracker.write_transcript(&format!("🛠️  Tool: {}", tool_use.name));
                    }
                }
                ContentBlock::Text(text_block) => {
                    tracker.write_transcript(&text_block.text);
                }
                ContentBlock::Thinking(_thinking) => {
                    // Optionally log thinking blocks
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Main chat function for the research agent.
async fn chat() -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let session_dir = PathBuf::from(format!("agent/research_sessions/session_{}", timestamp));
    std::fs::create_dir_all(&session_dir)?;

    let mut tracker = SubagentTracker::new(session_dir.clone())?;

    tracker.write_transcript(&format!("Research Agent Session Started: {}", timestamp));
    tracker.write_transcript(&format!("Session Directory: {}", session_dir.display()));

    let agents = create_agents();

    let options = ClaudeAgentOptions {
        permission_mode: Some(PermissionMode::BypassPermissions),
        system_prompt: Some(SystemPromptConfig::Text(
            include_str!("../prompts/lead_agent.txt").to_string(),
        )),
        allowed_tools: vec!["Task".to_string()],
        agents: Some(agents),
        model: Some("haiku".to_string()),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client
        .connect()
        .await
        .context("Failed to connect to Claude Code")?;

    println!("Research Agent started! Type 'exit' to quit.");
    println!("Session ID: {}", timestamp);

    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut input = String::new();

    loop {
        print!("\nYou: ");
        std::io::stdout().flush()?;

        input.clear();
        match stdin.read_line(&mut input).await {
            Ok(0) => break,
            Ok(_) => {
                let user_input = input.trim();
                if user_input.is_empty() {
                    continue;
                }
                if user_input == "exit" {
                    break;
                }

                tracker.write_transcript(&format!("\nYou: {}", user_input));

                let mut stream = client.query(user_input).await?;

                tracker.write_transcript("\nAgent: ");

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(message) => {
                            process_assistant_message(&message, &tracker, &mut std::io::stdout())?;
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            tracker.write_transcript(&format!("ERROR: {}", e));
                        }
                    }
                }

                tracker.write_transcript("\n");
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    tracker.write_transcript("\nResearch Agent Session Ended");
    tracker.close();

    println!(
        "\nSession completed! Logs saved to: {}",
        session_dir.join("logs").display()
    );

    client.disconnect().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("claude_agent=debug".parse().unwrap()),
        )
        .init();

    println!("Claude Agent SDK - Research Agent Demo");
    println!("Demonstrating multi-agent orchestration\n");

    chat().await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}
