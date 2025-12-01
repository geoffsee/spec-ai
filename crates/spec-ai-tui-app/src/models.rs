use chrono::{DateTime, Local, Utc};
use spec_ai_core::types::{Message, MessageRole};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
    Agent(String),
}

impl ChatRole {
    pub fn label(&self) -> String {
        match self {
            ChatRole::User => "User".to_string(),
            ChatRole::Assistant => "Assistant".to_string(),
            ChatRole::System => "System".to_string(),
            ChatRole::Agent(id) => format!("Agent {id}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    pub timestamp: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
            timestamp: Local::now().format("%H:%M:%S").to_string(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            timestamp: Local::now().format("%H:%M:%S").to_string(),
        }
    }

    pub fn from_backend(message: &Message) -> Self {
        let role = match &message.role {
            MessageRole::System => ChatRole::System,
            MessageRole::Assistant => ChatRole::Assistant,
            MessageRole::User => ChatRole::User,
            MessageRole::Agent(id) => ChatRole::Agent(id.clone()),
        };

        Self {
            role,
            content: message.content.clone(),
            timestamp: format_timestamp(message.created_at),
        }
    }
}

fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp
        .with_timezone(&Local)
        .format("%H:%M:%S")
        .to_string()
}
