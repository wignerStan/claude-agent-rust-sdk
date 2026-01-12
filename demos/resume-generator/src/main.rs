//! Resume Generator - Document generation demonstration.
//!
//! This demo showcases:
//! - Web search integration for research
//! - Document generation workflow
//! - Template-based formatting
//! - File output verification
//!
//! Migrated from: claude-agent-sdk-demos/resume-generator/resume-generator.ts

use anyhow::{Context, Result};
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;

/// Generate a professional resume for a given person.
///
/// This function demonstrates:
/// 1. Web search integration for gathering information
/// 2. AI-assisted document generation
/// 3. Professional resume formatting
///
/// # Arguments
///
/// * `person_name` - The name of the person to research
///
/// # Returns
///
/// Result indicating success or failure
async fn generate_resume(person_name: &str) -> Result<()> {
    let options = ClaudeAgentOptions {
        allowed_tools: vec![
            "WebSearch".to_string(),
            "WebFetch".to_string(),
            "Write".to_string(),
            "Read".to_string(),
            "Glob".to_string(),
        ],
        system_prompt: Some(
            "You are a professional resume writer. Research the person and create a 1-page resume in .docx format. \
             Include sections for: Professional Summary, Experience, Education, Skills, and Projects. \
             Use professional formatting with clear headings and bullet points. \
             Focus on achievements and quantifiable results. \
             Keep it concise and impactful.".to_string()
        ),
        max_turns: Some(30),
        model: Some("claude-sonnet-4-5".to_string()),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client
        .connect()
        .await
        .context("Failed to connect to Claude Code")?;

    let prompt = format!(
        "Research '{}' and create a professional 1-page resume in .docx format. \
         Include: Professional Summary, Experience, Education, Skills, and Projects sections. \
         Use professional formatting with clear headings and bullet points. \
         Focus on achievements and quantifiable results.",
        person_name
    );

    println!("Sending query: {}", prompt);
    println!("{}", "-".repeat(50));

    let mut stream = client
        .query(&prompt)
        .await
        .context("Failed to create query stream")?;

    let mut response_text = String::new();
    let mut tool_uses = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let claude_agent_types::Message::Assistant(msg) = message {
                    for block in msg.content {
                        if let claude_agent_types::message::ContentBlock::ToolUse(tool_use) = block {
                            println!("Tool used: {}", tool_use.name);
                            tool_uses.push(tool_use.name.clone());
                        }
                        if let claude_agent_types::message::ContentBlock::Text(text_block) = block {
                            response_text.push_str(&text_block.text);
                            print!("{}", text_block.text);
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    client
        .disconnect()
        .await
        .context("Failed to disconnect from Claude Code")?;

    println!("{}", "-".repeat(50));
    println!("Resume generation completed!");
    println!("Tools used: {}", tool_uses.join(", "));
    println!("Response length: {} characters", response_text.len());

    Ok(())
}

/// Generate a resume with specific focus areas.
async fn generate_resume_with_focus(
    person_name: &str,
    focus_areas: Vec<&str>,
) -> Result<()> {
    let focus_prompt = format!(
        "Research '{}' and create a professional resume focusing on: {}. \
         Include relevant achievements and quantifiable results in these areas.",
        person_name,
        focus_areas.join(", ")
    );

    let options = ClaudeAgentOptions {
        allowed_tools: vec![
            "WebSearch".to_string(),
            "WebFetch".to_string(),
            "Write".to_string(),
            "Read".to_string(),
        ],
        system_prompt: Some(
            "You are a professional resume writer. Create a 1-page resume in .docx format. \
             Tailor the content to highlight the specified focus areas. \
             Use strong action verbs and measurable achievements.".to_string()
        ),
        max_turns: Some(25),
        model: Some("claude-sonnet-4-5".to_string()),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client
        .connect()
        .await
        .context("Failed to connect to Claude Code")?;

    println!("Generating resume with focus on: {}", focus_areas.join(", "));
    println!("{}", "-".repeat(50));

    let mut stream = client
        .query(&focus_prompt)
        .await
        .context("Failed to create query stream")?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let claude_agent_types::Message::Assistant(msg) = message {
                    for block in msg.content {
                        if let claude_agent_types::message::ContentBlock::Text(text_block) = block {
                            println!("{}", text_block.text);
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    client
        .disconnect()
        .await
        .context("Failed to disconnect from Claude Code")?;

    println!("{}", "-".repeat(50));
    println!("Resume generation completed!");

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

    println!("Claude Agent SDK - Resume Generator Demo");
    println!("Demonstrating web search and document generation\n");

    let args: Vec<String> = std::env::args().collect();

    if args.is_empty() {
        println!("\nUsage: cargo run -- <person_name>");
        println!("\nExamples:");
        println!("  cargo run -- \"Jane Doe\"");
        println!("  cargo run -- \"John Smith\" --focus leadership,management");
        println!("\nOptions:");
        println!("  --focus <area1>,<area2>  Focus on specific resume sections");
        println!("\nThis will research the person and generate a professional 1-page resume.");
        std::process::exit(1);
    }

    let person_name = &args[0];

    let focus_areas = if args.len() > 2 {
        args[2..].split(',').map(|s| s.trim()).collect()
    } else {
        Vec::new()
    };

    if focus_areas.is_empty() {
        generate_resume(person_name).await?;
    } else {
        generate_resume_with_focus(person_name, focus_areas).await?;
    }

    Ok(())
}
