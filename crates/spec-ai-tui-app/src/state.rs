use crate::backend::BackendEvent;
use crate::models::ChatMessage;
use spec_ai_core::types::{Message, MessageRole};
use spec_ai_tui::widget::builtin::{EditorState, SlashCommand, SlashMenuState};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    Input,
    Chat,
}

pub struct AppState {
    pub editor: EditorState,
    pub slash_menu: SlashMenuState,
    pub slash_commands: Vec<SlashCommand>,
    pub messages: Vec<ChatMessage>,
    pub reasoning: Vec<String>,
    pub status: String,
    pub focus: PanelFocus,
    pub scroll_offset: u16,
    pub quit: bool,
    pub busy: bool,
    pub tick: u64,
    pub active_agent: Option<String>,
    pub error: Option<String>,
    pub backend_rx: UnboundedReceiver<BackendEvent>,
    pub last_submitted_text: Option<String>,
}

impl AppState {
    pub fn new(backend_rx: UnboundedReceiver<BackendEvent>) -> Self {
        Self {
            editor: EditorState::new(),
            slash_menu: SlashMenuState::new(),
            slash_commands: default_slash_commands(),
            messages: Vec::new(),
            reasoning: default_reasoning(),
            status: "Connecting to spec-ai backend...".to_string(),
            focus: PanelFocus::Input,
            scroll_offset: 0,
            quit: false,
            busy: true,
            tick: 0,
            active_agent: None,
            error: None,
            backend_rx,
            last_submitted_text: None,
        }
    }

    pub fn drain_backend_events(&mut self) {
        while let Ok(event) = self.backend_rx.try_recv() {
            self.apply_backend_event(event);
        }
    }

    fn apply_backend_event(&mut self, event: BackendEvent) {
        match event {
            BackendEvent::Initialized {
                agent,
                messages,
                reasoning,
                status,
            } => {
                self.active_agent = agent;
                self.messages = messages
                    .iter()
                    .map(ChatMessage::from_backend)
                    .collect::<Vec<_>>();
                self.reasoning = if reasoning.is_empty() {
                    default_reasoning()
                } else {
                    reasoning
                };
                self.status = status;
                self.busy = false;
                self.error = None;
                self.scroll_offset = 0;
            }
            BackendEvent::CommandResult {
                response,
                new_messages,
                reasoning,
                status,
            } => {
                self.busy = false;
                self.error = None;
                if !reasoning.is_empty() {
                    self.reasoning = reasoning;
                }
                self.status = status;
                if !new_messages.is_empty() {
                    self.append_messages(&new_messages);
                }
                if let Some(text) = response {
                    if new_messages.is_empty() && !text.trim().is_empty() {
                        self.messages.push(ChatMessage::system(clean_text(&text)));
                        self.scroll_offset = 0;
                    }
                }
                self.last_submitted_text = None;
            }
            BackendEvent::Error { context, message } => {
                self.busy = false;
                self.error = Some(message.clone());
                self.status = format!("Error while handling '{}'", context);
                self.messages
                    .push(ChatMessage::system(format!("Error: {}", message)));
                self.scroll_offset = 0;
                self.last_submitted_text = None;
            }
            BackendEvent::Quit => {
                self.quit = true;
            }
        }
    }

    fn append_messages(&mut self, incoming: &[Message]) {
        let mut skipped_user = false;

        for message in incoming {
            if message.role == MessageRole::User {
                if let Some(pending) = &self.last_submitted_text {
                    if !skipped_user && message.content.trim() == pending.trim() {
                        skipped_user = true;
                        continue;
                    }
                }
            }
            self.messages.push(ChatMessage::from_backend(&message));
        }

        if skipped_user {
            self.last_submitted_text = None;
        }

        if !incoming.is_empty() {
            self.scroll_offset = 0;
        }
    }
}

fn default_reasoning() -> Vec<String> {
    vec![
        "Recall: idle".to_string(),
        "Tool: idle".to_string(),
        "Tokens: waiting".to_string(),
    ]
}

fn default_slash_commands() -> Vec<SlashCommand> {
    vec![
        SlashCommand::new("help", "Show available commands"),
        SlashCommand::new("config", "Reload or show config (/config reload|show)"),
        SlashCommand::new("policy", "Reload policies"),
        SlashCommand::new("agents", "List configured agents"),
        SlashCommand::new("switch", "Switch active agent (/switch <name>)"),
        SlashCommand::new("memory", "Show recent memory (/memory show [n])"),
        SlashCommand::new("session", "Session actions (/session new|list|switch)"),
        SlashCommand::new("graph", "Graph tools (/graph status|show|clear)"),
        SlashCommand::new("sync", "List sync-enabled graphs"),
        SlashCommand::new("init", "Bootstrap knowledge graph (first command only)"),
        SlashCommand::new("refresh", "Refresh knowledge graph cache"),
        SlashCommand::new("listen", "Start or stop background transcription"),
        SlashCommand::new("spec", "Run a spec file (/spec run specs/smoke.spec)"),
        SlashCommand::new("speak", "Toggle spoken responses"),
    ]
}

fn clean_text(text: &str) -> String {
    strip_ansi_escapes::strip(text)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_else(|_| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_state() -> AppState {
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        AppState::new(rx)
    }

    #[test]
    fn panel_focus_equality() {
        assert_eq!(PanelFocus::Input, PanelFocus::Input);
        assert_eq!(PanelFocus::Chat, PanelFocus::Chat);
        assert_ne!(PanelFocus::Input, PanelFocus::Chat);
    }

    #[test]
    fn default_reasoning_returns_three_lines() {
        let reasoning = default_reasoning();
        assert_eq!(reasoning.len(), 3);
    }

    #[test]
    fn default_reasoning_contains_expected_content() {
        let reasoning = default_reasoning();
        assert!(reasoning[0].contains("Recall"));
        assert!(reasoning[1].contains("Tool"));
        assert!(reasoning[2].contains("Tokens"));
    }

    #[test]
    fn default_slash_commands_not_empty() {
        let commands = default_slash_commands();
        assert!(!commands.is_empty());
    }

    #[test]
    fn default_slash_commands_contains_help() {
        let commands = default_slash_commands();
        assert!(commands.iter().any(|cmd| cmd.name == "help"));
    }

    #[test]
    fn default_slash_commands_contains_config() {
        let commands = default_slash_commands();
        assert!(commands.iter().any(|cmd| cmd.name == "config"));
    }

    #[test]
    fn default_slash_commands_contains_switch() {
        let commands = default_slash_commands();
        assert!(commands.iter().any(|cmd| cmd.name == "switch"));
    }

    #[test]
    fn default_slash_commands_all_have_descriptions() {
        let commands = default_slash_commands();
        for cmd in commands {
            assert!(
                !cmd.description.is_empty(),
                "Command '{}' has no description",
                cmd.name
            );
        }
    }

    #[test]
    fn clean_text_preserves_plain_text() {
        let text = "Hello, world!";
        assert_eq!(clean_text(text), "Hello, world!");
    }

    #[test]
    fn clean_text_strips_ansi_color_codes() {
        let text = "\x1b[31mRed text\x1b[0m";
        assert_eq!(clean_text(text), "Red text");
    }

    #[test]
    fn clean_text_strips_bold_codes() {
        let text = "\x1b[1mBold\x1b[0m";
        assert_eq!(clean_text(text), "Bold");
    }

    #[test]
    fn clean_text_strips_multiple_codes() {
        let text = "\x1b[1;32mGreen Bold\x1b[0m and \x1b[34mBlue\x1b[0m";
        assert_eq!(clean_text(text), "Green Bold and Blue");
    }

    #[test]
    fn clean_text_handles_empty_string() {
        assert_eq!(clean_text(""), "");
    }

    #[test]
    fn app_state_new_initializes_correctly() {
        let state = create_test_state();
        assert_eq!(state.focus, PanelFocus::Input);
        assert_eq!(state.scroll_offset, 0);
        assert!(!state.quit);
        assert!(state.busy);
        assert_eq!(state.tick, 0);
        assert!(state.messages.is_empty());
        assert!(state.active_agent.is_none());
        assert!(state.error.is_none());
        assert!(state.last_submitted_text.is_none());
    }

    #[test]
    fn app_state_new_has_default_status() {
        let state = create_test_state();
        assert!(state.status.contains("Connecting"));
    }

    #[test]
    fn app_state_new_has_slash_commands() {
        let state = create_test_state();
        assert!(!state.slash_commands.is_empty());
    }

    #[test]
    fn app_state_new_has_default_reasoning() {
        let state = create_test_state();
        assert_eq!(state.reasoning.len(), 3);
    }

    #[test]
    fn apply_backend_event_quit_sets_flag() {
        let mut state = create_test_state();
        assert!(!state.quit);
        state.apply_backend_event(BackendEvent::Quit);
        assert!(state.quit);
    }

    #[test]
    fn apply_backend_event_error_sets_error() {
        let mut state = create_test_state();
        state.apply_backend_event(BackendEvent::Error {
            context: "test context".to_string(),
            message: "test error".to_string(),
        });
        assert!(!state.busy);
        assert_eq!(state.error, Some("test error".to_string()));
        assert!(state.status.contains("test context"));
    }

    #[test]
    fn apply_backend_event_error_adds_system_message() {
        let mut state = create_test_state();
        state.apply_backend_event(BackendEvent::Error {
            context: "ctx".to_string(),
            message: "error msg".to_string(),
        });
        assert_eq!(state.messages.len(), 1);
        assert!(state.messages[0].content.contains("error msg"));
    }

    #[test]
    fn apply_backend_event_initialized_sets_agent() {
        let mut state = create_test_state();
        state.apply_backend_event(BackendEvent::Initialized {
            agent: Some("test-agent".to_string()),
            messages: vec![],
            reasoning: vec![],
            status: "Ready".to_string(),
        });
        assert_eq!(state.active_agent, Some("test-agent".to_string()));
        assert!(!state.busy);
    }

    #[test]
    fn apply_backend_event_initialized_with_empty_reasoning_uses_default() {
        let mut state = create_test_state();
        state.apply_backend_event(BackendEvent::Initialized {
            agent: None,
            messages: vec![],
            reasoning: vec![],
            status: "Ready".to_string(),
        });
        // Should use default reasoning when empty
        assert_eq!(state.reasoning.len(), 3);
    }

    #[test]
    fn apply_backend_event_initialized_with_reasoning_preserves_it() {
        let mut state = create_test_state();
        let custom_reasoning = vec!["Custom line".to_string()];
        state.apply_backend_event(BackendEvent::Initialized {
            agent: None,
            messages: vec![],
            reasoning: custom_reasoning.clone(),
            status: "Ready".to_string(),
        });
        assert_eq!(state.reasoning, custom_reasoning);
    }

    #[test]
    fn apply_backend_event_command_result_clears_busy() {
        let mut state = create_test_state();
        state.busy = true;
        state.apply_backend_event(BackendEvent::CommandResult {
            response: None,
            new_messages: vec![],
            reasoning: vec![],
            status: "Done".to_string(),
        });
        assert!(!state.busy);
    }

    #[test]
    fn apply_backend_event_command_result_updates_status() {
        let mut state = create_test_state();
        state.apply_backend_event(BackendEvent::CommandResult {
            response: None,
            new_messages: vec![],
            reasoning: vec![],
            status: "New status".to_string(),
        });
        assert_eq!(state.status, "New status");
    }

    #[test]
    fn apply_backend_event_command_result_with_response_adds_message() {
        let mut state = create_test_state();
        state.apply_backend_event(BackendEvent::CommandResult {
            response: Some("Response text".to_string()),
            new_messages: vec![],
            reasoning: vec![],
            status: "Done".to_string(),
        });
        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.messages[0].content, "Response text");
    }

    #[test]
    fn apply_backend_event_command_result_empty_response_not_added() {
        let mut state = create_test_state();
        state.apply_backend_event(BackendEvent::CommandResult {
            response: Some("   ".to_string()),
            new_messages: vec![],
            reasoning: vec![],
            status: "Done".to_string(),
        });
        // Empty/whitespace-only response should not add a message
        assert!(state.messages.is_empty());
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
    fn append_messages_adds_new_messages() {
        let mut state = create_test_state();
        let messages = vec![make_test_message(MessageRole::Assistant, "Hello")];
        state.append_messages(&messages);
        assert_eq!(state.messages.len(), 1);
    }

    #[test]
    fn append_messages_skips_duplicate_user_message() {
        let mut state = create_test_state();
        state.last_submitted_text = Some("Hello".to_string());
        let messages = vec![make_test_message(MessageRole::User, "Hello")];
        state.append_messages(&messages);
        // Should skip the duplicate
        assert!(state.messages.is_empty());
    }

    #[test]
    fn append_messages_keeps_non_duplicate_user_message() {
        let mut state = create_test_state();
        state.last_submitted_text = Some("Hello".to_string());
        let messages = vec![make_test_message(MessageRole::User, "Different message")];
        state.append_messages(&messages);
        assert_eq!(state.messages.len(), 1);
    }

    #[test]
    fn append_messages_clears_last_submitted_after_skip() {
        let mut state = create_test_state();
        state.last_submitted_text = Some("Hello".to_string());
        let messages = vec![make_test_message(MessageRole::User, "Hello")];
        state.append_messages(&messages);
        assert!(state.last_submitted_text.is_none());
    }

    #[test]
    fn append_messages_resets_scroll_offset() {
        let mut state = create_test_state();
        state.scroll_offset = 10;
        let messages = vec![make_test_message(MessageRole::Assistant, "New message")];
        state.append_messages(&messages);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn append_messages_only_skips_first_matching_user() {
        let mut state = create_test_state();
        state.last_submitted_text = Some("Hello".to_string());
        let messages = vec![
            make_test_message(MessageRole::User, "Hello"),
            make_test_message(MessageRole::User, "Hello"),
        ];
        state.append_messages(&messages);
        // First "Hello" is skipped, second one should be added
        assert_eq!(state.messages.len(), 1);
    }
}
