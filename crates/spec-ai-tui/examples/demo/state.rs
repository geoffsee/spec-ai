//! Demo application state and defaults.

use crate::models::{AgentProcess, ChatMessage, Session, ToolExecution, ToolStatus};
use std::collections::BTreeMap;
use spec_ai_tui::widget::builtin::{EditorState, SlashCommand, SlashMenuState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Input,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OnboardingStep {
    Provider,
    Model,
    Confirm,
}

#[derive(Debug, Clone)]
pub struct ProviderOption {
    pub name: String,
    pub detected: bool,
    pub latency_ms: u32,
    pub region: String,
    pub note: String,
}

impl ProviderOption {
    pub fn new(name: &str, detected: bool, latency_ms: u32, region: &str, note: &str) -> Self {
        Self {
            name: name.to_string(),
            detected,
            latency_ms,
            region: region.to_string(),
            note: note.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelOption {
    pub provider: String,
    pub name: String,
    pub kind: ModelKind,
    pub context_window: String,
    pub modalities: String,
    pub pricing: String,
    pub latency: String,
    pub highlights: String,
}

impl ModelOption {
    pub fn new(
        provider: &str,
        name: &str,
        kind: ModelKind,
        context_window: &str,
        modalities: &str,
        pricing: &str,
        latency: &str,
        highlights: &str,
    ) -> Self {
        Self {
            provider: provider.to_string(),
            name: name.to_string(),
            kind,
            context_window: context_window.to_string(),
            modalities: modalities.to_string(),
            pricing: pricing.to_string(),
            latency: latency.to_string(),
            highlights: highlights.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelKind {
    Chat,
    FastChat,
    Embeddings,
    Audio,
}

impl ModelKind {
    pub fn label(&self) -> &'static str {
        match self {
            ModelKind::Chat => "chat",
            ModelKind::FastChat => "fast chat",
            ModelKind::Embeddings => "embeddings",
            ModelKind::Audio => "audio",
        }
    }

    pub fn detail(&self) -> &'static str {
        match self {
            ModelKind::Chat => "Primary model for responses",
            ModelKind::FastChat => "Fast model for routing/light tasks",
            ModelKind::Embeddings => "Vector model for search/recall",
            ModelKind::Audio => "Transcription/voice model",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolicyMode {
    Standard,
    Expanded,
}

impl PolicyMode {
    pub fn label(&self) -> &'static str {
        match self {
            PolicyMode::Standard => "standard policy",
            PolicyMode::Expanded => "expanded policy",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PolicyMode::Standard => "Safe defaults, read-only tools",
            PolicyMode::Expanded => "Broader tools, more autonomy",
        }
    }

    pub fn next(self) -> Self {
        match self {
            PolicyMode::Standard => PolicyMode::Expanded,
            PolicyMode::Expanded => PolicyMode::Standard,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Onboarding {
    pub active: bool,
    pub step: OnboardingStep,
    pub providers: Vec<ProviderOption>,
    pub selected_provider: usize,
    pub model_catalog: Vec<ModelOption>,
    pub selected_kind: usize,
    pub selected_models: BTreeMap<ModelKind, usize>,
    pub voice_enabled: bool,
    pub selected_tools: Vec<String>,
    pub policy_mode: PolicyMode,
    pub confirm_cursor: usize,
    pub show_policy_modal: bool,
}

impl Onboarding {
    pub fn new() -> Self {
        Self {
            active: true,
            step: OnboardingStep::Provider,
            providers: vec![
                ProviderOption::new(
                    "OpenAI",
                    true,
                    180,
                    "global",
                    "Detected from config",
                ),
                ProviderOption::new(
                    "Anthropic",
                    true,
                    210,
                    "us-east-1",
                    "API key in env",
                ),
                ProviderOption::new(
                    "Local",
                    false,
                    40,
                    "localhost",
                    "Start a local server to use",
                ),
            ],
            selected_provider: 0,
            model_catalog: vec![
                ModelOption::new(
                    "OpenAI",
                    "gpt-4.1",
                    ModelKind::Chat,
                    "128k ctx",
                    "text+vision",
                    "$5/$15 per 1M",
                    "~250ms first token",
                    "Strong reasoning + tools",
                ),
                ModelOption::new(
                    "OpenAI",
                    "gpt-4.1-mini",
                    ModelKind::FastChat,
                    "128k ctx",
                    "text+vision",
                    "$0.15/$0.60 per 1M",
                    "~90ms first token",
                    "Fast + cheap",
                ),
                ModelOption::new(
                    "OpenAI",
                    "text-embedding-3-small",
                    ModelKind::Embeddings,
                    "8k ctx",
                    "embeddings",
                    "$0.02 per 1M",
                    "n/a",
                    "Vectors for semantic search",
                ),
                ModelOption::new(
                    "vttrs",
                    "whisper-1",
                    ModelKind::Audio,
                    "n/a",
                    "audio",
                    "$0.01 per min",
                    "~500ms chunk",
                    "Streaming transcription",
                ),
                ModelOption::new(
                    "Anthropic",
                    "claude-3.5-sonnet",
                    ModelKind::Chat,
                    "200k ctx",
                    "text+vision",
                    "$3/$15 per 1M",
                    "~300ms first token",
                    "Good at tool use",
                ),
                ModelOption::new(
                    "Anthropic",
                    "claude-3-haiku",
                    ModelKind::FastChat,
                    "200k ctx",
                    "text",
                    "$0.25/$1.25 per 1M",
                    "~120ms first token",
                    "Budget friendly",
                ),
                ModelOption::new(
                    "Local",
                    "llama-3.1-8b",
                    ModelKind::Chat,
                    "16k ctx",
                    "text",
                    "n/a",
                    "~40ms first token",
                    "Runs on your box",
                ),
                ModelOption::new(
                    "Local",
                    "all-minilm-l6",
                    ModelKind::Embeddings,
                    "4k ctx",
                    "embeddings",
                    "n/a",
                    "n/a",
                    "Local embeddings",
                ),
                ModelOption::new(
                    "Local",
                    "faster-whisper",
                    ModelKind::Audio,
                    "n/a",
                    "audio",
                    "n/a",
                    "~400ms chunk",
                    "Offline transcription",
                ),
            ],
            selected_kind: 0,
            selected_models: BTreeMap::new(),
            voice_enabled: false,
            selected_tools: vec![
                "code_search".to_string(),
                "file_read".to_string(),
                "bash".to_string(),
            ],
            policy_mode: PolicyMode::Standard,
            confirm_cursor: 0,
            show_policy_modal: false,
        }
    }

    pub fn current_provider(&self) -> Option<&ProviderOption> {
        self.providers.get(self.selected_provider)
    }

    pub fn model_count_for_provider(&self) -> usize {
        self.models_for_provider().len()
    }

    pub fn provider_kinds(&self) -> Vec<ModelKind> {
        let provider = match self.current_provider() {
            Some(p) => p.name.as_str(),
            None => return Vec::new(),
        };
        let mut kinds: Vec<ModelKind> = self
            .model_catalog
            .iter()
            .filter(|m| m.provider == provider)
            .map(|m| m.kind)
            .collect();
        kinds.sort();
        kinds.dedup();
        kinds
    }

    pub fn current_kind(&self) -> Option<ModelKind> {
        self.provider_kinds().get(self.selected_kind).copied()
    }

    pub fn models_for_provider(&self) -> Vec<&ModelOption> {
        let provider = match self.current_provider() {
            Some(p) => p.name.as_str(),
            None => return Vec::new(),
        };
        self.model_catalog
            .iter()
            .filter(|m| m.provider == provider)
            .collect()
    }

    pub fn models_for_current(&self) -> Vec<&ModelOption> {
        let provider = match self.current_provider() {
            Some(p) => p.name.as_str(),
            None => return Vec::new(),
        };
        let kind = match self.current_kind() {
            Some(k) => k,
            None => return Vec::new(),
        };
        self.model_catalog
            .iter()
            .filter(|m| m.provider == provider && m.kind == kind)
            .collect()
    }

    pub fn current_model(&self) -> Option<&ModelOption> {
        let kind = self.current_kind()?;
        let models = self.models_for_current();
        let idx = self.selected_models.get(&kind).copied().unwrap_or(0);
        models.get(idx).copied()
    }

    pub fn model_for_kind(&self, kind: ModelKind) -> Option<&ModelOption> {
        let provider = self.current_provider()?.name.clone();
        let idx = self.selected_models.get(&kind).copied().unwrap_or(0);
        self.model_catalog
            .iter()
            .filter(|m| m.provider == provider && m.kind == kind)
            .nth(idx)
    }

    pub fn reset_model_cursor(&mut self) {
        self.selected_kind = 0;
        self.selected_models.clear();
    }

    pub fn current_kind_label(&self) -> String {
        self.current_kind()
            .map(|k| k.label().to_string())
            .unwrap_or_else(|| "model".to_string())
    }

    pub fn confirm_options_len(&self) -> usize {
        // Voice toggle, tool toggle, policy switch, finalize
        4
    }
}

/// Demo application state
#[derive(Clone)]
pub struct DemoState {
    /// Editor field state
    pub editor: EditorState,
    /// Slash menu state
    pub slash_menu: SlashMenuState,
    /// Available slash commands
    pub slash_commands: Vec<SlashCommand>,
    /// Chat messages
    pub messages: Vec<ChatMessage>,
    /// Current streaming response (simulated)
    pub streaming: Option<String>,
    /// Scroll offset for chat
    pub scroll_offset: u16,
    /// Status message
    pub status: String,
    /// Active tools
    pub tools: Vec<ToolExecution>,
    /// Reasoning messages
    pub reasoning: Vec<String>,
    /// Should quit
    pub quit: bool,
    /// Current panel focus
    pub focus: Panel,
    /// Tick counter for animations
    pub tick: u64,
    /// Simulated streaming state
    pub stream_buffer: Vec<&'static str>,
    pub stream_index: usize,
    /// Mock listening mode active
    pub listening: bool,
    /// Simulated listening transcript buffer
    pub listen_buffer: Vec<&'static str>,
    /// Current index in listening buffer
    pub listen_index: usize,
    /// Display buffer for recent listening lines
    pub listen_log: Vec<String>,
    /// Agent-spawned processes
    pub processes: Vec<AgentProcess>,
    /// Show process manager overlay
    pub show_process_panel: bool,
    /// Selected process in panel
    pub selected_process: usize,
    /// Viewing logs for process (index)
    pub viewing_logs: Option<usize>,
    /// Log scroll offset
    pub log_scroll: usize,
    /// Session history
    pub sessions: Vec<Session>,
    /// Current session index
    pub current_session: usize,
    /// Show history overlay
    pub show_history: bool,
    /// Selected session in history
    pub selected_session: usize,
    /// Pending quit (first Ctrl+C pressed)
    pub pending_quit: bool,
    /// Onboarding flow
    pub onboarding: Onboarding,
    /// Voice output enabled after onboarding
    pub voice_enabled: bool,
    /// Active policy mode
    pub policy_mode: PolicyMode,
    /// Tools allowed after onboarding
    pub allowed_tools: Vec<String>,
}

impl Default for DemoState {
    fn default() -> Self {
        let onboarding = Onboarding::new();
        let default_tools = onboarding.selected_tools.clone();
        let policy_mode = onboarding.policy_mode;
        let voice_enabled = onboarding.voice_enabled;
        Self {
            editor: EditorState::new(),
            slash_menu: SlashMenuState::new(),
            slash_commands: vec![
                SlashCommand::new("help", "Show available commands"),
                SlashCommand::new("clear", "Clear the chat history"),
                SlashCommand::new("model", "Switch AI model"),
                SlashCommand::new("system", "Set system prompt"),
                SlashCommand::new("export", "Export conversation"),
                SlashCommand::new("settings", "Open settings"),
                SlashCommand::new("theme", "Change color theme"),
                SlashCommand::new("tools", "List available tools"),
                SlashCommand::new("listen", "Toggle mock audio listening"),
            ],
            messages: vec![
                ChatMessage::new("system", "Welcome to spec-ai! I'm your AI assistant.", "10:00"),
                ChatMessage::new("user", "Can you find the main entry point of the TUI crate?", "10:01"),
                ChatMessage::new("assistant", "I'll search for the main entry point in the TUI crate.", "10:01"),
                ChatMessage::tool(
                    "code_search",
                    "Searching for: \"fn main\" in crates/spec-ai-tui/\n\
                     Found 2 results:\n\
                     → examples/demo.rs:704 - async fn main()\n\
                     → src/lib.rs - (no main, library crate)",
                    "10:01"
                ),
                ChatMessage::new(
                    "assistant",
                    "Found it! The main entry point is in `examples/demo.rs` at line 704. Let me read that file to show you the structure.",
                    "10:01"
                ),
                ChatMessage::tool(
                    "file_read",
                    "Reading: examples/demo.rs (lines 704-720)\n\
                     ```rust\n\
                     #[tokio::main]\n\
                     async fn main() -> std::io::Result<()> {\n\
                         let app = DemoApp;\n\
                         let mut runner = AppRunner::new(app)?;\n\
                         runner.run().await\n\
                     }\n\
                     ```",
                    "10:02"
                ),
                ChatMessage::new(
                    "assistant",
                    "The TUI uses an async main function with tokio. Here's how it works:\n\n\
                     • **DemoApp** - Implements the App trait with init(), handle_event(), render()\n\
                     • **AppRunner** - Manages the terminal, event loop, and rendering\n\
                     • **run()** - Starts the async event loop\n\n\
                     The App trait follows an Elm-like architecture for clean state management.",
                    "10:02"
                ),
                ChatMessage::new("user", "What about error handling?", "10:03"),
                ChatMessage::tool(
                    "grep",
                    "Searching for: \"Result<\" in src/\n\
                     Found 47 matches across 12 files\n\
                     Most common: io::Result<()> for terminal operations",
                    "10:03"
                ),
                ChatMessage::new(
                    "assistant",
                    "Error handling uses Rust's standard Result type:\n\n\
                     • Terminal operations return `io::Result<()>`\n\
                     • The `?` operator propagates errors up\n\
                     • RAII guards ensure cleanup even on panic\n\n\
                     Try typing a message or use / to see available commands!",
                    "10:03"
                ),
            ],
            streaming: None,
            scroll_offset: 0,
            status: "Setup: pick a provider to start (↑/↓, Enter)".to_string(),
            tools: vec![
                ToolExecution { name: "code_search".to_string(), status: ToolStatus::Success, duration_ms: Some(45) },
                ToolExecution { name: "file_read".to_string(), status: ToolStatus::Success, duration_ms: Some(12) },
                ToolExecution { name: "grep".to_string(), status: ToolStatus::Success, duration_ms: Some(89) },
            ],
            reasoning: vec![
                "◆ Setup required: select a provider".to_string(),
                "  Step 1/3 (provider → model → confirm)".to_string(),
                "  ↑/↓ to move, Enter to confirm".to_string(),
            ],
            quit: false,
            focus: Panel::Input,
            tick: 0,
            stream_buffer: vec![
                "I'm ",
                "simulating ",
                "a ",
                "streaming ",
                "response ",
                "to ",
                "demonstrate ",
                "the ",
                "real-time ",
                "token ",
                "rendering ",
                "capability ",
                "of ",
                "this ",
                "TUI. ",
                "\n\n",
                "Each ",
                "word ",
                "appears ",
                "progressively, ",
                "just ",
                "like ",
                "an ",
                "actual ",
                "LLM ",
                "response!",
            ],
            stream_index: 0,
            listening: false,
            listen_buffer: vec![
                "[mic] Calibrating input gain... (ok)",
                "[mic] User: \"hey there, can you summarize the last deploy?\"",
                "[mic] Assistant: \"Sure, checking the deploy logs now.\"",
                "[mic] User: \"focus on API changes and DB migrations\"",
                "[mic] (silence) listening...",
            ],
            listen_index: 0,
            listen_log: Vec::new(),
            // Mock agent-spawned processes with full log output
            processes: vec![
                {
                    let mut proc = AgentProcess::new(48291, "cargo test --all --no-fail-fast", "test-runner");
                    proc.elapsed_ms = 45200;
                    proc.output_lines = vec![
                        "   Compiling spec-ai-tui v0.4.16".to_string(),
                        "   Compiling spec-ai v0.4.16".to_string(),
                        "    Finished test [unoptimized + debuginfo] target(s) in 12.34s".to_string(),
                        "     Running unittests src/lib.rs".to_string(),
                        "".to_string(),
                        "running 47 tests".to_string(),
                        "test buffer::tests::test_cell_default ... ok".to_string(),
                        "test buffer::tests::test_buffer_creation ... ok".to_string(),
                        "test buffer::tests::test_set_string ... ok".to_string(),
                        "test geometry::tests::test_rect_new ... ok".to_string(),
                        "test geometry::tests::test_rect_intersection ... ok".to_string(),
                        "test geometry::tests::test_rect_union ... ok".to_string(),
                        "test layout::tests::test_vertical_split ... ok".to_string(),
                        "test layout::tests::test_horizontal_split ... ok".to_string(),
                        "test style::tests::test_color_rgb ... ok".to_string(),
                        "test style::tests::test_modifier_combine ... ok".to_string(),
                        "test widget::tests::test_block_render ... ok".to_string(),
                        "test widget::tests::test_editor_insert ... ok".to_string(),
                        "...".to_string(),
                        "test result: ok. 47 passed; 0 failed; 0 ignored".to_string(),
                    ];
                    proc
                },
                {
                    let mut proc = AgentProcess::new(48156, "npm run dev -- --port 3000", "dev-server");
                    proc.elapsed_ms = 182000;
                    proc.output_lines = vec![
                        "> frontend@0.1.0 dev".to_string(),
                        "> next dev --port 3000".to_string(),
                        "".to_string(),
                        "  ▲ Next.js 14.0.4".to_string(),
                        "  - Local:        http://localhost:3000".to_string(),
                        "  - Environments: .env.local".to_string(),
                        "".to_string(),
                        " ✓ Ready in 2.1s".to_string(),
                        " ○ Compiling /page ...".to_string(),
                        " ✓ Compiled /page in 892ms (512 modules)".to_string(),
                        " ○ Compiling /api/auth ...".to_string(),
                        " ✓ Compiled /api/auth in 234ms (89 modules)".to_string(),
                        " GET / 200 in 45ms".to_string(),
                        " GET /api/auth/session 200 in 12ms".to_string(),
                        " GET /_next/static/chunks/main.js 200 in 3ms".to_string(),
                    ];
                    proc
                },
                {
                    let mut proc = AgentProcess::new(47892, "cargo watch -x check", "file-watcher");
                    proc.elapsed_ms = 892000;
                    proc.output_lines = vec![
                        "[cargo-watch] Watching /Users/dev/project".to_string(),
                        "[cargo-watch] Waiting for changes...".to_string(),
                        "[cargo-watch] Change detected: src/lib.rs".to_string(),
                        "[Running 'cargo check']".to_string(),
                        "    Checking spec-ai-tui v0.4.16".to_string(),
                        "    Checking spec-ai v0.4.16".to_string(),
                        "    Finished dev [unoptimized + debuginfo] target(s) in 4.21s".to_string(),
                        "[cargo-watch] Waiting for changes...".to_string(),
                        "[cargo-watch] Change detected: src/widget/mod.rs".to_string(),
                        "[Running 'cargo check']".to_string(),
                        "    Checking spec-ai-tui v0.4.16".to_string(),
                        "    Finished dev [unoptimized + debuginfo] target(s) in 1.89s".to_string(),
                        "[cargo-watch] Waiting for changes...".to_string(),
                    ];
                    proc
                },
                {
                    let mut proc = AgentProcess::new(48302, "docker compose up db redis", "infra");
                    proc.elapsed_ms = 120000;
                    proc.output_lines = vec![
                        "[+] Running 2/2".to_string(),
                        " ⠿ Container project-db-1     Created".to_string(),
                        " ⠿ Container project-redis-1  Created".to_string(),
                        "Attaching to db-1, redis-1".to_string(),
                        "db-1    | PostgreSQL Database directory appears to contain a database".to_string(),
                        "db-1    | 2024-01-15 10:00:01.234 UTC [1] LOG:  starting PostgreSQL 15.4".to_string(),
                        "db-1    | 2024-01-15 10:00:01.456 UTC [1] LOG:  listening on IPv4 address \"0.0.0.0\", port 5432".to_string(),
                        "db-1    | 2024-01-15 10:00:01.567 UTC [1] LOG:  database system is ready to accept connections".to_string(),
                        "redis-1 | 1:C 15 Jan 2024 10:00:01.123 # oO0OoO0OoO0Oo Redis is starting".to_string(),
                        "redis-1 | 1:M 15 Jan 2024 10:00:01.234 * Ready to accept connections".to_string(),
                        "db-1    | 2024-01-15 10:02:15.789 UTC [47] LOG:  connection received: host=172.18.0.1 port=54321".to_string(),
                        "db-1    | 2024-01-15 10:02:15.812 UTC [47] LOG:  connection authorized: user=app database=myapp".to_string(),
                    ];
                    proc
                },
            ],
            show_process_panel: false,
            selected_process: 0,
            viewing_logs: None,
            log_scroll: 0,
            // Mock sessions with history
            sessions: vec![
                Session {
                    id: 1,
                    title: "Current session".to_string(),
                    preview: "TUI framework development".to_string(),
                    timestamp: "Today 10:00".to_string(),
                    message_count: 0, // Will be updated from messages
                    messages: vec![], // Current session uses state.messages
                },
                Session {
                    id: 2,
                    title: "API refactoring".to_string(),
                    preview: "Discussed REST → GraphQL migration".to_string(),
                    timestamp: "Yesterday".to_string(),
                    message_count: 24,
                    messages: vec![
                        ChatMessage::new("user", "How should we approach the API migration?", "14:30"),
                        ChatMessage::new("assistant", "I recommend a phased approach: 1) Create GraphQL schema, 2) Implement resolvers, 3) Add Apollo client, 4) Deprecate REST endpoints.", "14:31"),
                        ChatMessage::new("user", "What about backwards compatibility?", "14:35"),
                        ChatMessage::new("assistant", "Keep REST endpoints during transition. Add deprecation headers and document timeline.", "14:36"),
                    ],
                },
                Session {
                    id: 3,
                    title: "Bug investigation".to_string(),
                    preview: "Memory leak in connection pool".to_string(),
                    timestamp: "2 days ago".to_string(),
                    message_count: 18,
                    messages: vec![
                        ChatMessage::new("user", "There's a memory leak when connections aren't returned to pool", "09:15"),
                        ChatMessage::tool("grep", "Searching for: 'pool.acquire' in src/\nFound 12 occurrences", "09:15"),
                        ChatMessage::new("assistant", "Found the issue - connections acquired in error paths aren't being released. Adding drop guards.", "09:16"),
                    ],
                },
                Session {
                    id: 4,
                    title: "Performance optimization".to_string(),
                    preview: "Reduced latency by 40%".to_string(),
                    timestamp: "Last week".to_string(),
                    message_count: 42,
                    messages: vec![
                        ChatMessage::new("user", "The dashboard is loading slowly", "16:00"),
                        ChatMessage::tool("profiler", "Flame graph generated: database queries take 800ms", "16:01"),
                        ChatMessage::new("assistant", "Main bottleneck is N+1 queries. Adding eager loading and query batching.", "16:02"),
                    ],
                },
                Session {
                    id: 5,
                    title: "Documentation sprint".to_string(),
                    preview: "Updated API docs and README".to_string(),
                    timestamp: "2 weeks ago".to_string(),
                    message_count: 15,
                    messages: vec![
                        ChatMessage::new("user", "Help me document the authentication flow", "11:00"),
                        ChatMessage::new("assistant", "I'll create a sequence diagram and update the API reference.", "11:01"),
                    ],
                },
            ],
            current_session: 0,
            show_history: false,
            selected_session: 0,
            pending_quit: false,
            onboarding,
            voice_enabled,
            policy_mode,
            allowed_tools: default_tools,
        }
    }
}
