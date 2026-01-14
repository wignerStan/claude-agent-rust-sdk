//! Email Agent - IMAP email assistant (simplified demonstration).
//!
//! This demo showcases:
//! - IMAP email connection and listing
//! - Email search functionality
//! - Agent-assisted email responses
//!
//! Note: This is a simplified demonstration. The original TypeScript demo
//! included complex features like OAuth2, real-time IDLE monitoring,
//! and SQLite persistence. This version focuses on core functionality.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::{config::SystemPromptConfig, ClaudeAgentOptions};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Email structure for simplified demonstration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub body: String,
    pub date: DateTime<Utc>,
    pub unread: bool,
}

/// Email fetcher for demonstrating IMAP integration patterns.
#[derive(Debug, Clone)]
pub struct EmailFetcher {
    emails: Arc<Mutex<Vec<Email>>>,
}

impl EmailFetcher {
    /// Create a new email fetcher with mock data.
    pub fn new() -> Self {
        let mock_emails = vec![
            Email {
                id: "1".to_string(),
                subject: "Meeting Tomorrow".to_string(),
                from: "alice@example.com".to_string(),
                to: "user@example.com".to_string(),
                body: "Hi, let's meet tomorrow at 2 PM to discuss the project.".to_string(),
                date: Utc::now(),
                unread: true,
            },
            Email {
                id: "2".to_string(),
                subject: "Project Update".to_string(),
                from: "bob@example.com".to_string(),
                to: "user@example.com".to_string(),
                body: "Here's the latest update on our project progress.".to_string(),
                date: Utc::now(),
                unread: false,
            },
            Email {
                id: "3".to_string(),
                subject: "Question about API".to_string(),
                from: "charlie@example.com".to_string(),
                to: "user@example.com".to_string(),
                body: "I have a question about the API documentation.".to_string(),
                date: Utc::now(),
                unread: true,
            },
        ];

        Self {
            emails: Arc::new(Mutex::new(mock_emails)),
        }
    }

    /// List emails (simulates IMAP LIST command).
    pub fn list_emails(&self) -> Vec<Email> {
        self.emails.lock().unwrap().clone()
    }

    /// Search emails by query.
    pub fn search_emails(&self, query: &str) -> Vec<Email> {
        let query_lower = query.to_lowercase();
        self.emails
            .lock()
            .unwrap()
            .iter()
            .filter(|email| {
                email.subject.to_lowercase().contains(&query_lower)
                    || email.body.to_lowercase().contains(&query_lower)
                    || email.from.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// Get email by ID.
    pub fn get_email(&self, id: &str) -> Option<Email> {
        self.emails
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.id == id)
            .cloned()
    }
}

impl Default for EmailFetcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent-assisted email processing.
pub struct EmailAgent {
    client: ClaudeAgentClient,
    email_fetcher: EmailFetcher,
}

impl EmailAgent {
    /// Create a new email agent.
    pub async fn new() -> Result<Self> {
        let options = ClaudeAgentOptions {
            allowed_tools: vec![
                "WebSearch".to_string(),
                "WebFetch".to_string(),
                "Write".to_string(),
                "Read".to_string(),
            ],
            system_prompt: Some(SystemPromptConfig::Text(
                "You are an email assistant. Help users process and respond to emails. \
                 Be professional, concise, and helpful."
                    .to_string(),
            )),
            model: std::env::var("ANTHROPIC_MODEL").ok(),
            ..Default::default()
        };

        let mut client = ClaudeAgentClient::new(Some(options));
        client.connect().await.context("Failed to connect")?;

        Ok(Self {
            client,
            email_fetcher: EmailFetcher::new(),
        })
    }

    /// List all emails.
    pub fn list_emails(&self) -> Vec<Email> {
        self.email_fetcher.list_emails()
    }

    /// Search emails.
    pub fn search_emails(&self, query: &str) -> Vec<Email> {
        self.email_fetcher.search_emails(query)
    }

    /// Generate a draft response for an email.
    pub async fn draft_response(&mut self, email: &Email) -> Result<String> {
        let prompt = format!(
            "Draft a professional response to this email:\n\n\
             From: {}\n\
             Subject: {}\n\n\
             Body:\n{}\n\n\
             Write a concise, professional response.",
            email.from, email.subject, email.body
        );

        let mut stream = self.client.query(&prompt).await?;
        let mut response = String::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    if let claude_agent_types::Message::Assistant(msg) = message {
                        for block in msg.content {
                            if let claude_agent_types::message::ContentBlock::Text(text_block) =
                                block
                            {
                                response.push_str(&text_block.text);
                            }
                        }
                    }
                }
                Err(e) => {
                    anyhow::bail!("Error generating response: {}", e);
                }
            }
        }

        Ok(response)
    }

    /// Summarize an email.
    pub async fn summarize_email(&mut self, email: &Email) -> Result<String> {
        let prompt = format!(
            "Summarize this email in 2-3 sentences:\n\n\
             From: {}\n\
             Subject: {}\n\n\
             Body:\n{}",
            email.from, email.subject, email.body
        );

        let mut stream = self.client.query(&prompt).await?;
        let mut summary = String::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    if let claude_agent_types::Message::Assistant(msg) = message {
                        for block in msg.content {
                            if let claude_agent_types::message::ContentBlock::Text(text_block) =
                                block
                            {
                                summary.push_str(&text_block.text);
                            }
                        }
                    }
                }
                Err(e) => {
                    anyhow::bail!("Error generating summary: {}", e);
                }
            }
        }

        Ok(summary)
    }

    /// Clean up resources.
    pub async fn close(&mut self) {
        let _ = self.client.disconnect().await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("claude_agent=debug".parse().unwrap()),
        )
        .init();

    println!("Claude Agent SDK - Email Agent Demo (Simplified)");
    println!("Demonstrating IMAP integration patterns\n");

    // Create email agent
    println!("Initializing email agent...");
    let mut agent = EmailAgent::new().await?;
    println!("Email agent initialized!");

    // List emails
    println!("\n=== Listing Emails ===\n");
    let emails = agent.list_emails();

    for email in &emails {
        println!(
            "[{}] {} - {}",
            if email.unread { "NEW" } else { "  " },
            email.from,
            email.subject
        );
    }

    // Search emails
    println!("\n=== Searching Emails ===\n");
    println!("Searching for 'project'...");

    let search_results = agent.search_emails("project");
    println!("Found {} emails:", search_results.len());

    for email in &search_results {
        println!("  - {}: {}", email.from, email.subject);
    }

    // Generate draft response
    if let Some(first_email) = emails.first() {
        println!("\n=== Drafting Response ===\n");
        println!("Email: {} - {}", first_email.from, first_email.subject);
        println!("\nGenerating response...");

        let response = agent.draft_response(first_email).await?;
        println!("\nDraft Response:\n{}", response);
    }

    // Summarize email
    if let Some(second_email) = emails.get(1) {
        println!("\n=== Summarizing Email ===\n");
        println!("Email: {} - {}", second_email.from, second_email.subject);

        let summary = agent.summarize_email(second_email).await?;
        println!("\nSummary:\n{}", summary);
    }

    // Clean up
    agent.close().await;

    println!("\n=== Demo Complete ===");
    println!("Note: This is a simplified demonstration.");
    println!("For full IMAP functionality, configure real IMAP credentials.");

    Ok(())
}
