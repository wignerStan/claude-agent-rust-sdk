//! Tests for Email Agent demo.

use chrono::{DateTime, Utc};
use claude_agent_api::ClaudeAgentClient;
use claude_agent_demos_common::test_utils::{MockTransport, StreamCollector};
use claude_agent_types::message::{ContentBlock, Message};
use claude_agent_types::ClaudeAgentError;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

const TEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn test_email_structure() {
    let email = Email {
        id: "test-123".to_string(),
        subject: "Test Subject".to_string(),
        from: "test@example.com".to_string(),
        to: "user@example.com".to_string(),
        body: "Test body content".to_string(),
        date: Utc::now(),
        unread: true,
    };

    assert_eq!(email.id, "test-123");
    assert_eq!(email.subject, "Test Subject");
    assert_eq!(email.from, "test@example.com");
    assert!(email.unread);
}

#[tokio::test]
async fn test_email_fetcher_creation() {
    let fetcher = EmailFetcher::new();
    let emails = fetcher.list_emails();

    assert!(!emails.is_empty());
    assert_eq!(emails.len(), 3); // We have 3 mock emails
}

#[tokio::test]
async fn test_email_search_by_subject() {
    let fetcher = EmailFetcher::new();

    let results = fetcher.search_emails("Meeting");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].subject, "Meeting Tomorrow");
}

#[tokio::test]
async fn test_email_search_by_sender() {
    let fetcher = EmailFetcher::new();

    let results = fetcher.search_emails("alice");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].from, "alice@example.com");
}

#[tokio::test]
async fn test_email_search_by_body() {
    let fetcher = EmailFetcher::new();

    let results = fetcher.search_emails("project");
    assert_eq!(results.len(), 2); // "Meeting Tomorrow" and "Project Update"
}

#[tokio::test]
async fn test_email_search_case_insensitive() {
    let fetcher = EmailFetcher::new();

    let results_lower = fetcher.search_emails("meeting");
    let results_upper = fetcher.search_emails("MEETING");
    let results_mixed = fetcher.search_emails("MeEtInG");

    assert_eq!(results_lower.len(), results_upper.len());
    assert_eq!(results_upper.len(), results_mixed.len());
    assert_eq!(results_lower.len(), 1);
}

#[tokio::test]
async fn test_email_search_no_results() {
    let fetcher = EmailFetcher::new();

    let results = fetcher.search_emails("nonexistent query");
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_get_email_by_id() {
    let fetcher = EmailFetcher::new();

    let email = fetcher.get_email("1");
    assert!(email.is_some());
    assert_eq!(email.unwrap().id, "1");

    let non_existent = fetcher.get_email("999");
    assert!(non_existent.is_none());
}

#[tokio::test]
async fn test_email_serialization() {
    let email = Email {
        id: "test-123".to_string(),
        subject: "Test".to_string(),
        from: "test@example.com".to_string(),
        to: "user@example.com".to_string(),
        body: "Body".to_string(),
        date: Utc::now(),
        unread: false,
    };

    let json = serde_json::to_string(&email).unwrap();
    let deserialized: Email = serde_json::from_str(&json).unwrap();

    assert_eq!(email.id, deserialized.id);
    assert_eq!(email.subject, deserialized.subject);
    assert_eq!(email.from, deserialized.from);
    assert_eq!(email.body, deserialized.body);
}

#[tokio::test]
async fn test_email_agent_draft_response() {
    timeout(TEST_TIMEOUT, async {
        let response_text = "Dear Alice, I'd be happy to meet tomorrow at 2 PM.";
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": response_text}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(mock_transport));
        client.connect().await.unwrap();

        let email = Email {
            id: "1".to_string(),
            subject: "Meeting Tomorrow".to_string(),
            from: "alice@example.com".to_string(),
            to: "user@example.com".to_string(),
            body: "Hi, let's meet tomorrow at 2 PM to discuss the project.".to_string(),
            date: Utc::now(),
            unread: true,
        };

        let prompt = format!(
            "Draft a professional response to this email:\n\nFrom: {}\nSubject: {}\n\nBody:\n{}\n\nWrite a concise, professional response.",
            email.from, email.subject, email.body
        );

        let stream = client.query(&prompt).await.unwrap();
        let messages = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
        if let Message::Assistant(msg) = &messages[0] {
            if let ContentBlock::Text(text_block) = &msg.content[0] {
                assert_eq!(text_block.text, response_text);
            } else {
                panic!("Expected text block");
            }
        } else {
            panic!("Expected assistant message");
        }
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_email_agent_summarize() {
    timeout(TEST_TIMEOUT, async {
        let summary_text = "Alice wants to meet tomorrow at 2 PM to discuss the project.";
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": summary_text}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(mock_transport));
        client.connect().await.unwrap();

        let email = Email {
            id: "1".to_string(),
            subject: "Meeting Tomorrow".to_string(),
            from: "alice@example.com".to_string(),
            to: "user@example.com".to_string(),
            body: "Hi, let's meet tomorrow at 2 PM to discuss the project.".to_string(),
            date: Utc::now(),
            unread: true,
        };

        let prompt = format!(
            "Summarize this email in 2-3 sentences:\n\nFrom: {}\nSubject: {}\n\nBody:\n{}",
            email.from, email.subject, email.body
        );

        let stream = client.query(&prompt).await.unwrap();
        let messages = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
        if let Message::Assistant(msg) = &messages[0] {
            if let ContentBlock::Text(text_block) = &msg.content[0] {
                assert_eq!(text_block.text, summary_text);
            } else {
                panic!("Expected text block");
            }
        } else {
            panic!("Expected assistant message");
        }
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_email_agent_creation_logic() {
    // Verify unread status count from mock data
    let fetcher = EmailFetcher::new();
    let emails = fetcher.list_emails();
    let unread_count = emails.iter().filter(|e| e.unread).count();
    assert_eq!(unread_count, 2);
}

#[tokio::test]
async fn test_email_listing_all_fields() {
    let fetcher = EmailFetcher::new();
    let emails = fetcher.list_emails();

    for email in &emails {
        assert!(!email.id.is_empty());
        assert!(!email.subject.is_empty());
        assert!(!email.from.is_empty());
        assert!(!email.to.is_empty());
        assert!(!email.body.is_empty());
    }
}

#[tokio::test]
async fn test_email_fetcher_clone() {
    let fetcher1 = EmailFetcher::new();
    let fetcher2 = fetcher1.clone();

    let emails1 = fetcher1.list_emails();
    let emails2 = fetcher2.list_emails();

    assert_eq!(emails1.len(), emails2.len());
}

// Simplified structures for testing (matching main.rs)
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

#[derive(Debug, Clone)]
pub struct EmailFetcher {
    emails: Arc<Mutex<Vec<Email>>>,
}

impl EmailFetcher {
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

    pub fn list_emails(&self) -> Vec<Email> {
        self.emails.lock().unwrap().clone()
    }

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

    pub fn get_email(&self, id: &str) -> Option<Email> {
        self.emails
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.id == id)
            .cloned()
    }
}
