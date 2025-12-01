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
