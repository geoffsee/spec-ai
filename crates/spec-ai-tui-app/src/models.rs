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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_role_label_user() {
        assert_eq!(ChatRole::User.label(), "User");
    }

    #[test]
    fn chat_role_label_assistant() {
        assert_eq!(ChatRole::Assistant.label(), "Assistant");
    }

    #[test]
    fn chat_role_label_system() {
        assert_eq!(ChatRole::System.label(), "System");
    }

    #[test]
    fn chat_role_label_agent() {
        let agent = ChatRole::Agent("test-agent".to_string());
        assert_eq!(agent.label(), "Agent test-agent");
    }

    #[test]
    fn chat_role_label_agent_with_spaces() {
        let agent = ChatRole::Agent("my agent 1".to_string());
        assert_eq!(agent.label(), "Agent my agent 1");
    }

    #[test]
    fn chat_message_system_sets_correct_role() {
        let msg = ChatMessage::system("Hello");
        assert_eq!(msg.role, ChatRole::System);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn chat_message_system_accepts_string() {
        let msg = ChatMessage::system(String::from("Test message"));
        assert_eq!(msg.content, "Test message");
    }

    #[test]
    fn chat_message_user_sets_correct_role() {
        let msg = ChatMessage::user("User input");
        assert_eq!(msg.role, ChatRole::User);
        assert_eq!(msg.content, "User input");
    }

    #[test]
    fn chat_message_user_accepts_string() {
        let msg = ChatMessage::user(String::from("Another message"));
        assert_eq!(msg.content, "Another message");
    }

    #[test]
    fn chat_message_timestamp_is_valid_format() {
        let msg = ChatMessage::system("test");
        // Timestamp should be in HH:MM:SS format
        assert_eq!(msg.timestamp.len(), 8);
        assert!(msg.timestamp.chars().nth(2) == Some(':'));
        assert!(msg.timestamp.chars().nth(5) == Some(':'));
    }

    fn make_test_message(role: MessageRole, content: &str) -> Message {
        Message {
            id: 0,
            session_id: "test-session".to_string(),
            role,
            content: content.to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn chat_message_from_backend_user_role() {
        let backend_msg = make_test_message(MessageRole::User, "User content");
        let chat_msg = ChatMessage::from_backend(&backend_msg);
        assert_eq!(chat_msg.role, ChatRole::User);
        assert_eq!(chat_msg.content, "User content");
    }

    #[test]
    fn chat_message_from_backend_assistant_role() {
        let backend_msg = make_test_message(MessageRole::Assistant, "Assistant response");
        let chat_msg = ChatMessage::from_backend(&backend_msg);
        assert_eq!(chat_msg.role, ChatRole::Assistant);
        assert_eq!(chat_msg.content, "Assistant response");
    }

    #[test]
    fn chat_message_from_backend_system_role() {
        let backend_msg = make_test_message(MessageRole::System, "System notification");
        let chat_msg = ChatMessage::from_backend(&backend_msg);
        assert_eq!(chat_msg.role, ChatRole::System);
        assert_eq!(chat_msg.content, "System notification");
    }

    #[test]
    fn chat_message_from_backend_agent_role() {
        let backend_msg = make_test_message(MessageRole::Agent("agent-42".to_string()), "Agent output");
        let chat_msg = ChatMessage::from_backend(&backend_msg);
        assert_eq!(chat_msg.role, ChatRole::Agent("agent-42".to_string()));
        assert_eq!(chat_msg.content, "Agent output");
    }

    #[test]
    fn format_timestamp_produces_valid_format() {
        let timestamp = Utc::now();
        let formatted = format_timestamp(timestamp);
        // Should be HH:MM:SS format
        assert_eq!(formatted.len(), 8);
        assert!(formatted.chars().nth(2) == Some(':'));
        assert!(formatted.chars().nth(5) == Some(':'));
    }

    #[test]
    fn chat_role_equality() {
        assert_eq!(ChatRole::User, ChatRole::User);
        assert_eq!(ChatRole::Assistant, ChatRole::Assistant);
        assert_eq!(ChatRole::System, ChatRole::System);
        assert_eq!(
            ChatRole::Agent("foo".to_string()),
            ChatRole::Agent("foo".to_string())
        );
        assert_ne!(
            ChatRole::Agent("foo".to_string()),
            ChatRole::Agent("bar".to_string())
        );
        assert_ne!(ChatRole::User, ChatRole::Assistant);
    }
}
