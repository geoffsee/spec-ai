//! Demo application showcasing spec-ai-tui features
//!
//! Run with: cargo run -p spec-ai-tui --example demo

use spec_ai_tui::{
    app::{App, AppRunner},
    buffer::Buffer,
    event::{Event, KeyCode, KeyModifiers},
    geometry::Rect,
    layout::{Constraint, Layout},
    style::{Color, Line, Span, Style},
    widget::{
        builtin::{
            Block, Editor, EditorAction, EditorState,
            SlashCommand, SlashMenu, SlashMenuState,
            StatusBar, StatusSection,
        },
        Widget, StatefulWidget,
    },
};

/// Mock chat message
#[derive(Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
    timestamp: String,
    /// Optional tool name (for tool role)
    tool_name: Option<String>,
}

impl ChatMessage {
    fn new(role: &str, content: &str, timestamp: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: timestamp.to_string(),
            tool_name: None,
        }
    }

    fn tool(name: &str, content: &str, timestamp: &str) -> Self {
        Self {
            role: "tool".to_string(),
            content: content.to_string(),
            timestamp: timestamp.to_string(),
            tool_name: Some(name.to_string()),
        }
    }
}

/// Mock tool execution
#[derive(Debug, Clone)]
struct ToolExecution {
    name: String,
    status: ToolStatus,
    duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum ToolStatus {
    Running,
    Success,
    Failed,
}

/// Agent-spawned process
#[derive(Debug, Clone)]
struct AgentProcess {
    pid: u32,
    command: String,
    agent: String,  // Which agent spawned this
    status: ProcessStatus,
    exit_code: Option<i32>,
    elapsed_ms: u64,
    output_lines: Vec<String>,  // Last few lines of output
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ProcessStatus {
    Running,
    Stopped,
    Completed,
    Failed,
}

impl AgentProcess {
    fn new(pid: u32, command: &str, agent: &str) -> Self {
        Self {
            pid,
            command: command.to_string(),
            agent: agent.to_string(),
            status: ProcessStatus::Running,
            exit_code: None,
            elapsed_ms: 0,
            output_lines: Vec::new(),
        }
    }

    fn status_icon(&self) -> (&'static str, Color) {
        match self.status {
            ProcessStatus::Running => ("●", Color::Green),
            ProcessStatus::Stopped => ("◉", Color::Yellow),
            ProcessStatus::Completed => ("✓", Color::DarkGrey),
            ProcessStatus::Failed => ("✗", Color::Red),
        }
    }

    fn elapsed_display(&self) -> String {
        let secs = self.elapsed_ms / 1000;
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m{}s", secs / 60, secs % 60)
        } else {
            format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
        }
    }
}

/// Chat session for history
#[derive(Debug, Clone)]
struct Session {
    id: usize,
    title: String,
    preview: String,
    timestamp: String,
    message_count: usize,
    messages: Vec<ChatMessage>,
}

/// Demo application state
struct DemoState {
    /// Editor field state
    editor: EditorState,
    /// Slash menu state
    slash_menu: SlashMenuState,
    /// Available slash commands
    slash_commands: Vec<SlashCommand>,
    /// Chat messages
    messages: Vec<ChatMessage>,
    /// Current streaming response (simulated)
    streaming: Option<String>,
    /// Scroll offset for chat
    scroll_offset: u16,
    /// Status message
    status: String,
    /// Active tools
    tools: Vec<ToolExecution>,
    /// Reasoning messages
    reasoning: Vec<String>,
    /// Should quit
    quit: bool,
    /// Current panel focus
    focus: Panel,
    /// Tick counter for animations
    tick: u64,
    /// Simulated streaming state
    stream_buffer: Vec<&'static str>,
    stream_index: usize,
    /// Agent-spawned processes
    processes: Vec<AgentProcess>,
    /// Show process manager overlay
    show_process_panel: bool,
    /// Selected process in panel
    selected_process: usize,
    /// Viewing logs for process (index)
    viewing_logs: Option<usize>,
    /// Log scroll offset
    log_scroll: usize,
    /// Session history
    sessions: Vec<Session>,
    /// Current session index
    current_session: usize,
    /// Show history overlay
    show_history: bool,
    /// Selected session in history
    selected_session: usize,
    /// Pending quit (first Ctrl+C pressed)
    pending_quit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Panel {
    Input,
    Chat,
}

impl Default for DemoState {
    fn default() -> Self {
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
            status: "Ready".to_string(),
            tools: vec![
                ToolExecution { name: "code_search".to_string(), status: ToolStatus::Success, duration_ms: Some(45) },
                ToolExecution { name: "file_read".to_string(), status: ToolStatus::Success, duration_ms: Some(12) },
                ToolExecution { name: "grep".to_string(), status: ToolStatus::Success, duration_ms: Some(89) },
            ],
            reasoning: vec![
                "◆ Analyzing user query...".to_string(),
                "◆ Searching codebase for entry points".to_string(),
                "◆ Context: 3 tools used, 847 tokens".to_string(),
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
        }
    }
}

/// Demo application
struct DemoApp;

impl App for DemoApp {
    type State = DemoState;

    fn init(&self) -> Self::State {
        let mut state = DemoState::default();
        state.editor.focused = true;
        state
    }

    fn handle_event(&mut self, event: Event, state: &mut Self::State) -> bool {
        // Handle quit (Ctrl+C)
        if event.is_quit() {
            if state.pending_quit {
                // Second Ctrl+C - actually quit
                state.quit = true;
                return false;
            } else {
                // First Ctrl+C - show warning
                state.pending_quit = true;
                state.status = "Press Ctrl+C again to exit".to_string();
                return true;
            }
        }

        // Reset pending_quit on any key input (not tick events)
        if state.pending_quit {
            if let Event::Key(_) = event {
                state.pending_quit = false;
                state.status = "Ready".to_string();
            }
        }

        match event {
            Event::Key(key) => {
                // Handle log viewing overlay (highest priority - sub-overlay of process panel)
                if let Some(proc_idx) = state.viewing_logs {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            state.viewing_logs = None;
                            return true;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            state.log_scroll = state.log_scroll.saturating_add(1);
                            return true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            state.log_scroll = state.log_scroll.saturating_sub(1);
                            return true;
                        }
                        KeyCode::PageUp => {
                            state.log_scroll = state.log_scroll.saturating_add(10);
                            return true;
                        }
                        KeyCode::PageDown => {
                            state.log_scroll = state.log_scroll.saturating_sub(10);
                            return true;
                        }
                        KeyCode::Char('g') => {
                            // Jump to top (oldest logs)
                            if let Some(proc) = state.processes.get(proc_idx) {
                                state.log_scroll = proc.output_lines.len().saturating_sub(1);
                            }
                            return true;
                        }
                        KeyCode::Char('G') => {
                            // Jump to bottom (newest logs)
                            state.log_scroll = 0;
                            return true;
                        }
                        _ => return true, // Consume all keys when log view is open
                    }
                }

                // Handle overlay panels first (Escape to close)
                if state.show_process_panel || state.show_history {
                    match key.code {
                        KeyCode::Esc => {
                            state.show_process_panel = false;
                            state.show_history = false;
                            return true;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.show_process_panel {
                                state.selected_process = state.selected_process.saturating_sub(1);
                            } else if state.show_history {
                                state.selected_session = state.selected_session.saturating_sub(1);
                            }
                            return true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.show_process_panel {
                                let max = state.processes.len().saturating_sub(1);
                                state.selected_process = (state.selected_process + 1).min(max);
                            } else if state.show_history {
                                let max = state.sessions.len().saturating_sub(1);
                                state.selected_session = (state.selected_session + 1).min(max);
                            }
                            return true;
                        }
                        KeyCode::Enter => {
                            if state.show_history && state.selected_session < state.sessions.len() {
                                // Switch to selected session
                                if state.selected_session != state.current_session {
                                    // Save current messages to current session
                                    if state.current_session == 0 {
                                        state.sessions[0].messages = state.messages.clone();
                                        state.sessions[0].message_count = state.messages.len();
                                    }
                                    // Load selected session
                                    state.current_session = state.selected_session;
                                    if state.selected_session == 0 {
                                        // Restore saved messages or keep current
                                        state.messages = state.sessions[0].messages.clone();
                                    } else {
                                        state.messages = state.sessions[state.selected_session].messages.clone();
                                    }
                                    state.status = format!("Switched to: {}", state.sessions[state.selected_session].title);
                                    state.scroll_offset = 0;
                                }
                                state.show_history = false;
                            } else if state.show_process_panel && state.selected_process < state.processes.len() {
                                // Open log view for selected process
                                state.viewing_logs = Some(state.selected_process);
                                state.log_scroll = 0;
                                if let Some(proc) = state.processes.get(state.selected_process) {
                                    state.status = format!("Logs: PID {} (↑↓ scroll, g/G top/bottom, Esc close)", proc.pid);
                                }
                            }
                            return true;
                        }
                        KeyCode::Char('s') if state.show_process_panel => {
                            // Toggle stop/continue for selected process
                            if state.selected_process < state.processes.len() {
                                let proc = &mut state.processes[state.selected_process];
                                match proc.status {
                                    ProcessStatus::Running => {
                                        proc.status = ProcessStatus::Stopped;
                                        state.status = format!("Stopped PID {}: {}", proc.pid, truncate_cmd(&proc.command, 30));
                                    }
                                    ProcessStatus::Stopped => {
                                        proc.status = ProcessStatus::Running;
                                        state.status = format!("Continued PID {}", proc.pid);
                                    }
                                    _ => {}
                                }
                            }
                            return true;
                        }
                        KeyCode::Char('x') if state.show_process_panel => {
                            // Kill selected process (x for terminate)
                            if state.selected_process < state.processes.len() {
                                let proc = &mut state.processes[state.selected_process];
                                if proc.status == ProcessStatus::Running || proc.status == ProcessStatus::Stopped {
                                    proc.status = ProcessStatus::Failed;
                                    proc.exit_code = Some(-9); // SIGKILL
                                    state.status = format!("Killed PID {} (SIGKILL)", proc.pid);
                                }
                            }
                            return true;
                        }
                        KeyCode::Char('d') if state.show_process_panel => {
                            // Remove completed/failed process from list
                            if state.selected_process < state.processes.len() {
                                let proc = &state.processes[state.selected_process];
                                if proc.status == ProcessStatus::Completed || proc.status == ProcessStatus::Failed {
                                    let pid = proc.pid;
                                    state.processes.remove(state.selected_process);
                                    if state.selected_process > 0 && state.selected_process >= state.processes.len() {
                                        state.selected_process -= 1;
                                    }
                                    state.status = format!("Removed PID {} from list", pid);
                                }
                            }
                            return true;
                        }
                        _ => return true, // Consume all keys when overlay is open
                    }
                }

                // Global shortcuts (when not in slash menu)
                if !state.editor.show_slash_menu && key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('t') => {
                            // Toggle process manager
                            state.show_process_panel = !state.show_process_panel;
                            state.show_history = false;
                            if state.show_process_panel {
                                state.status = "Processes (↑↓ nav, Enter stop/cont, x kill, d remove, Esc close)".to_string();
                            }
                            return true;
                        }
                        KeyCode::Char('h') => {
                            // Toggle history panel
                            state.show_history = !state.show_history;
                            state.show_process_panel = false;
                            if state.show_history {
                                state.status = "Session history (↑↓ select, Enter switch, Esc close)".to_string();
                            }
                            return true;
                        }
                        KeyCode::Char('l') => {
                            // Clear chat
                            state.messages.clear();
                            state.status = "Chat cleared".to_string();
                            return true;
                        }
                        KeyCode::Char('r') => {
                            // Simulate tool running
                            state.tools.push(ToolExecution {
                                name: "code_search".to_string(),
                                status: ToolStatus::Running,
                                duration_ms: None,
                            });
                            state.reasoning[1] = "◆ Tools: code_search (running...)".to_string();
                            state.status = "Running tool...".to_string();
                            return true;
                        }
                        KeyCode::Char('s') => {
                            // Start streaming simulation
                            if state.streaming.is_none() {
                                state.streaming = Some(String::new());
                                state.stream_index = 0;
                                state.status = "Streaming...".to_string();
                            }
                            return true;
                        }
                        _ => {}
                    }
                }

                // Handle arrow keys for slash menu navigation
                if state.editor.show_slash_menu {
                    match key.code {
                        KeyCode::Down => {
                            let count = filtered_command_count(state);
                            state.slash_menu.next(count);
                            return true;
                        }
                        KeyCode::Up => {
                            let count = filtered_command_count(state);
                            state.slash_menu.prev(count);
                            return true;
                        }
                        _ => {}
                    }
                }

                // Panel-specific handling
                match state.focus {
                    Panel::Input => {
                        // Update slash menu visibility from editor state
                        let was_showing = state.editor.show_slash_menu;

                        // Let the editor handle the event
                        match state.editor.handle_event(&event) {
                            EditorAction::Handled => {
                                // Sync slash menu visibility
                                if state.editor.show_slash_menu && !was_showing {
                                    state.slash_menu.show();
                                } else if !state.editor.show_slash_menu && was_showing {
                                    state.slash_menu.hide();
                                }
                            }
                            EditorAction::Submit(text) => {
                                if !text.is_empty() {
                                    // Add user message
                                    let timestamp = format!("{}:{:02}",
                                        10 + state.messages.len() / 60,
                                        state.messages.len() % 60);
                                    state.messages.push(ChatMessage::new("user", &text, &timestamp));

                                    // Start streaming response
                                    state.streaming = Some(String::new());
                                    state.stream_index = 0;
                                    state.status = "Generating response...".to_string();
                                }
                            }
                            EditorAction::SlashCommand(cmd) => {
                                // Execute the slash command
                                execute_slash_command(&cmd, state);
                                state.editor.clear();
                                state.slash_menu.hide();
                            }
                            EditorAction::SlashMenuNext => {
                                let count = filtered_command_count(state);
                                state.slash_menu.next(count);
                            }
                            EditorAction::SlashMenuPrev => {
                                let count = filtered_command_count(state);
                                state.slash_menu.prev(count);
                            }
                            EditorAction::Escape => {
                                // Do nothing, slash menu already closed
                            }
                            EditorAction::Ignored => {
                                // Handle keys not handled by editor
                                match key.code {
                                    KeyCode::Up if !state.editor.show_slash_menu => {
                                        // Switch to chat panel
                                        state.focus = Panel::Chat;
                                        state.editor.focused = false;
                                    }
                                    KeyCode::PageUp => {
                                        state.scroll_offset = state.scroll_offset.saturating_add(5);
                                    }
                                    KeyCode::PageDown => {
                                        state.scroll_offset = state.scroll_offset.saturating_sub(5);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Panel::Chat => {
                        match key.code {
                            KeyCode::Tab => {
                                // Tab always switches to input
                                state.focus = Panel::Input;
                                state.editor.focused = true;
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                // Scroll down, or switch to input if at bottom
                                if state.scroll_offset > 0 {
                                    state.scroll_offset = state.scroll_offset.saturating_sub(1);
                                } else {
                                    state.focus = Panel::Input;
                                    state.editor.focused = true;
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                state.scroll_offset = state.scroll_offset.saturating_add(1);
                            }
                            KeyCode::PageUp => {
                                state.scroll_offset = state.scroll_offset.saturating_add(10);
                            }
                            KeyCode::PageDown => {
                                state.scroll_offset = state.scroll_offset.saturating_sub(10);
                            }
                            KeyCode::Char('g') => {
                                state.scroll_offset = 100; // Top
                            }
                            KeyCode::Char('G') => {
                                state.scroll_offset = 0; // Bottom
                            }
                            _ => {}
                        }
                    }
                }
            }
            Event::Paste(_) => {
                // Handle paste when input is focused
                if state.focus == Panel::Input {
                    let was_showing = state.editor.show_slash_menu;
                    match state.editor.handle_event(&event) {
                        EditorAction::Handled => {
                            if state.editor.show_slash_menu && !was_showing {
                                state.slash_menu.show();
                            } else if !state.editor.show_slash_menu && was_showing {
                                state.slash_menu.hide();
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Resize { .. } => {
                // Terminal will handle resize
            }
            Event::Tick => {
                // Handle tick for animations
            }
            _ => {}
        }

        true
    }

    fn on_tick(&mut self, state: &mut Self::State) {
        state.tick += 1;

        // Spinner frames for animation
        let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let spin_char = spinner[(state.tick / 2) as usize % spinner.len()];

        // Simulate streaming
        if let Some(ref mut streaming) = state.streaming {
            if state.stream_index < state.stream_buffer.len() {
                streaming.push_str(state.stream_buffer[state.stream_index]);
                state.stream_index += 1;

                // Update reasoning during streaming
                let tokens_out = streaming.split_whitespace().count();
                state.reasoning[0] = format!("{} Generating response...", spin_char);
                state.reasoning[1] = format!("  Tokens: ~{} output", tokens_out);
                state.reasoning[2] = format!("  Progress: {}/{} chunks",
                    state.stream_index, state.stream_buffer.len());
            } else {
                // Streaming complete
                let response = state.streaming.take().unwrap();
                let timestamp = format!("{}:{:02}",
                    10 + state.messages.len() / 60,
                    state.messages.len() % 60);
                state.messages.push(ChatMessage::new("assistant", &response, &timestamp));
                state.status = "Ready".to_string();
                state.stream_index = 0;

                // Update reasoning to show completion
                state.reasoning[0] = "✓ Response complete".to_string();
                state.reasoning[1] = format!("  Total messages: {}", state.messages.len());
                state.reasoning[2] = format!("  Tools used: {}", state.tools.len());
            }
        }

        // Simulate tool execution
        let mut any_running = false;
        for tool in &mut state.tools {
            if tool.status == ToolStatus::Running {
                any_running = true;
                if tool.duration_ms.is_none() {
                    tool.duration_ms = Some(0);
                }
                tool.duration_ms = Some(tool.duration_ms.unwrap() + 100);

                // Complete after ~3 seconds
                if tool.duration_ms.unwrap() >= 3000 {
                    tool.status = ToolStatus::Success;
                    state.reasoning[0] = format!("✓ {} completed", tool.name);
                    state.reasoning[1] = format!("  Duration: {}ms", tool.duration_ms.unwrap());
                    state.status = "Ready".to_string();
                } else {
                    state.reasoning[0] = format!("{} Running {}...", spin_char, tool.name);
                    state.reasoning[1] = format!("  Elapsed: {}ms", tool.duration_ms.unwrap());
                    state.reasoning[2] = "  Waiting for results".to_string();
                }
            }
        }

        // Update running process elapsed times
        let mut running_count = 0;
        for proc in &mut state.processes {
            if proc.status == ProcessStatus::Running {
                running_count += 1;
                proc.elapsed_ms += 100;
            }
        }

        // Idle state animation
        if state.streaming.is_none() && !any_running {
            if state.tick % 50 == 0 {
                // Occasionally update idle reasoning
                let proc_info = if running_count > 0 {
                    format!("  {} processes running", running_count)
                } else {
                    "  Type / for commands".to_string()
                };

                let shortcuts_hint = "  Ctrl+T for processes, Ctrl+H for history";
                let waiting_hint = "  Waiting for input";
                let context_hint = "  Context loaded";
                let messages_hint = format!("  {} messages in history", state.messages.len());
                let tools_hint = format!("  {} tools available", 8);

                let idx = ((state.tick / 50) as usize) % 3;
                match idx {
                    0 => {
                        state.reasoning[0] = "◇ Ready".to_string();
                        state.reasoning[1] = waiting_hint.to_string();
                        state.reasoning[2] = proc_info;
                    }
                    1 => {
                        state.reasoning[0] = "◇ Idle".to_string();
                        state.reasoning[1] = context_hint.to_string();
                        state.reasoning[2] = messages_hint;
                    }
                    _ => {
                        state.reasoning[0] = "◇ Ready".to_string();
                        state.reasoning[1] = tools_hint;
                        state.reasoning[2] = shortcuts_hint.to_string();
                    }
                }
            }
        }
    }

    fn render(&self, state: &Self::State, area: Rect, buf: &mut Buffer) {
        // Main layout: content + status
        let main_chunks = Layout::vertical()
            .constraints([
                Constraint::Fill(1),   // Main content
                Constraint::Fixed(3),  // Reasoning panel
                Constraint::Fixed(1),  // Status bar
            ])
            .split(area);

        // Content layout: chat + input
        let content_chunks = Layout::vertical()
            .constraints([
                Constraint::Fill(1),   // Chat
                Constraint::Fixed(8),  // Input area (taller for wrapped text)
            ])
            .split(main_chunks[0]);

        // Render chat area
        self.render_chat(state, content_chunks[0], buf);

        // Render input area (with slash menu)
        self.render_input(state, content_chunks[1], buf);

        // Render reasoning panel
        self.render_reasoning(state, main_chunks[1], buf);

        // Render status bar
        self.render_status(state, main_chunks[2], buf);

        // Render overlay panels (on top of everything)
        if state.show_process_panel {
            self.render_process_overlay(state, area, buf);
        }
        if state.show_history {
            self.render_history_overlay(state, area, buf);
        }
        // Log viewer (on top of process overlay)
        if state.viewing_logs.is_some() {
            self.render_log_overlay(state, area, buf);
        }
    }
}

/// Truncate a command string to fit width, preserving the start
fn truncate_cmd(cmd: &str, max_width: usize) -> String {
    if cmd.len() <= max_width {
        cmd.to_string()
    } else if max_width <= 3 {
        "...".chars().take(max_width).collect()
    } else {
        format!("{}...", &cmd[..max_width - 3])
    }
}

fn filtered_command_count(state: &DemoState) -> usize {
    state.slash_commands
        .iter()
        .filter(|cmd| cmd.matches(&state.editor.slash_query))
        .count()
}

fn execute_slash_command(cmd: &str, state: &mut DemoState) {
    // Find the command that matches
    let filtered: Vec<_> = state.slash_commands
        .iter()
        .filter(|c| c.matches(&state.editor.slash_query))
        .collect();

    let selected_cmd = filtered.get(state.slash_menu.selected_index())
        .map(|c| c.name.as_str())
        .unwrap_or(cmd);

    let timestamp = format!("{}:{:02}",
        10 + state.messages.len() / 60,
        state.messages.len() % 60);

    match selected_cmd {
        "help" => {
            state.messages.push(ChatMessage::new(
                "system",
                "╭─ Keyboard Controls ─────────────────────────────╮\n\
                 │                                                 │\n\
                 │  INPUT PANEL                                    │\n\
                 │  Enter      Send message (triggers streaming)   │\n\
                 │  /          Open slash command menu             │\n\
                 │  Alt+b/f    Word navigation (back/forward)      │\n\
                 │  Ctrl+Z     Undo                                │\n\
                 │  ↑          Switch to Chat panel                │\n\
                 │                                                 │\n\
                 │  CHAT PANEL                                     │\n\
                 │  ↑/k        Scroll up                           │\n\
                 │  ↓/j        Scroll down                         │\n\
                 │  PageUp/Dn  Scroll by 10 lines                  │\n\
                 │  g/G        Jump to top/bottom                  │\n\
                 │  Tab        Return to Input panel               │\n\
                 │                                                 │\n\
                 │  GLOBAL                                         │\n\
                 │  Ctrl+C     Quit                                │\n\
                 │  Ctrl+L     Clear chat                          │\n\
                 │  Ctrl+T     Open agent processes panel          │\n\
                 │  Ctrl+H     Open session history                │\n\
                 │  Ctrl+R     Simulate tool execution             │\n\
                 │  Ctrl+S     Simulate streaming response         │\n\
                 │                                                 │\n\
                 │  SLASH COMMANDS                                 │\n\
                 │  /help /clear /model /system /export            │\n\
                 │  /settings /theme /tools                        │\n\
                 │                                                 │\n\
                 ╰─────────────────────────────────────────────────╯",
                &timestamp,
            ));
            state.status = "Help displayed".to_string();
        }
        "clear" => {
            state.messages.clear();
            state.status = "Chat cleared".to_string();
        }
        "model" => {
            state.status = "Model selection not implemented".to_string();
        }
        "theme" => {
            state.status = "Theme selection not implemented".to_string();
        }
        "tools" => {
            state.messages.push(ChatMessage::new(
                "system",
                "Available tools:\n\
                 • code_search - Search codebase\n\
                 • file_read - Read file contents\n\
                 • file_write - Write to files\n\
                 • bash - Execute shell commands",
                &timestamp,
            ));
            state.status = "Tools listed".to_string();
        }
        _ => {
            state.status = format!("Unknown command: /{}", selected_cmd);
        }
    }
}

/// Word-wrap a string to fit within max_width, preserving a prefix for continuation lines
fn wrap_text(text: &str, max_width: usize, prefix: &str) -> Vec<String> {
    if max_width == 0 {
        return vec![];
    }

    let mut result = Vec::new();
    let prefix_width = unicode_width::UnicodeWidthStr::width(prefix);
    let first_line_width = max_width;
    let continuation_width = max_width.saturating_sub(prefix_width);

    if continuation_width == 0 {
        return vec![text.chars().take(max_width).collect()];
    }

    let mut current_line = String::new();
    let mut current_width = 0usize;
    let mut is_first_line = true;

    for word in text.split_whitespace() {
        let word_width = unicode_width::UnicodeWidthStr::width(word);

        // Determine available width for this line
        let line_max = if is_first_line { first_line_width } else { continuation_width };

        if current_width == 0 {
            // First word on line
            if word_width <= line_max {
                current_line.push_str(word);
                current_width = word_width;
            } else {
                // Word is too long, need to break it
                let mut chars = word.chars().peekable();
                while chars.peek().is_some() {
                    let chunk: String = chars.by_ref().take(line_max).collect();
                    if !current_line.is_empty() {
                        if is_first_line {
                            result.push(current_line);
                        } else {
                            result.push(format!("{}{}", prefix, current_line));
                        }
                        is_first_line = false;
                    }
                    current_line = chunk;
                    current_width = unicode_width::UnicodeWidthStr::width(current_line.as_str());
                }
            }
        } else if current_width + 1 + word_width <= line_max {
            // Word fits on current line
            current_line.push(' ');
            current_line.push_str(word);
            current_width += 1 + word_width;
        } else {
            // Need to wrap
            if is_first_line {
                result.push(current_line);
            } else {
                result.push(format!("{}{}", prefix, current_line));
            }
            is_first_line = false;
            current_line = word.to_string();
            current_width = word_width;
        }
    }

    // Push remaining content
    if !current_line.is_empty() {
        if is_first_line {
            result.push(current_line);
        } else {
            result.push(format!("{}{}", prefix, current_line));
        }
    }

    if result.is_empty() {
        result.push(String::new());
    }

    result
}

impl DemoApp {
    fn render_chat(&self, state: &DemoState, area: Rect, buf: &mut Buffer) {
        // Draw border
        let border_style = if state.focus == Panel::Chat {
            Style::new().fg(Color::Cyan)
        } else {
            Style::new().fg(Color::DarkGrey)
        };

        let block = Block::bordered()
            .title("Chat")
            .border_style(border_style);
        Widget::render(&block, area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        // Reserve 1 char for scrollbar
        let content_width = inner.width.saturating_sub(1) as usize;

        // Build chat content with word wrapping
        let mut lines: Vec<Line> = Vec::new();

        for msg in &state.messages {
            // Role header
            let (role_style, role_display) = match msg.role.as_str() {
                "user" => (Style::new().fg(Color::Green).bold(), "user".to_string()),
                "assistant" => (Style::new().fg(Color::Cyan).bold(), "assistant".to_string()),
                "system" => (Style::new().fg(Color::Yellow).bold(), "system".to_string()),
                "tool" => {
                    let name = msg.tool_name.as_deref().unwrap_or("tool");
                    (Style::new().fg(Color::Magenta).bold(), format!("⚙ {}", name))
                }
                _ => (Style::new().fg(Color::White), msg.role.clone()),
            };

            lines.push(Line::from_spans([
                Span::styled(format!("[{}] ", msg.timestamp), Style::new().fg(Color::DarkGrey)),
                Span::styled(format!("{}:", role_display), role_style),
            ]));

            // Content lines with word wrapping
            // Tool messages get a special background indicator
            let is_tool = msg.role == "tool";
            let content_style = if is_tool {
                Style::new().fg(Color::DarkGrey)
            } else {
                Style::new()
            };
            let prefix = if is_tool { "  │ " } else { "  " };

            for content_line in msg.content.lines() {
                let prefixed = format!("{}{}", prefix, content_line);
                let wrapped = wrap_text(&prefixed, content_width, prefix);
                for wrapped_line in wrapped {
                    if is_tool {
                        lines.push(Line::styled(wrapped_line, content_style));
                    } else {
                        lines.push(Line::raw(wrapped_line));
                    }
                }
            }

            lines.push(Line::empty());
        }

        // Add streaming content
        if let Some(ref streaming) = state.streaming {
            lines.push(Line::from_spans([
                Span::styled("[--:--] ", Style::new().fg(Color::DarkGrey)),
                Span::styled("assistant:", Style::new().fg(Color::Cyan).bold()),
                Span::styled(" (streaming...)", Style::new().fg(Color::DarkGrey).italic()),
            ]));

            for content_line in streaming.lines() {
                let prefixed = format!("  {}", content_line);
                let wrapped = wrap_text(&prefixed, content_width, "  ");
                for wrapped_line in wrapped {
                    lines.push(Line::raw(wrapped_line));
                }
            }

            // Blinking cursor - always include line to prevent bobbing
            let cursor_char = if state.tick % 10 < 5 { "█" } else { " " };
            lines.push(Line::styled(format!("  {}", cursor_char), Style::new().fg(Color::Cyan)));
        }

        // Calculate visible lines
        let visible_height = inner.height as usize;
        let total_lines = lines.len();
        let scroll = state.scroll_offset as usize;

        // Scroll from bottom
        let start = if total_lines > visible_height + scroll {
            total_lines - visible_height - scroll
        } else {
            0
        };
        let end = (start + visible_height).min(total_lines);

        // Render visible lines
        for (i, line) in lines[start..end].iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            buf.set_line(inner.x, y, line);
        }

        // Scroll indicator
        if total_lines > visible_height {
            let scrollbar_height = inner.height.saturating_sub(2);
            let thumb_pos = if total_lines > 0 {
                ((start as u32 * scrollbar_height as u32) / total_lines as u32) as u16
            } else {
                0
            };

            for y in 0..scrollbar_height {
                let char = if y == thumb_pos { "█" } else { "░" };
                buf.set_string(
                    inner.right().saturating_sub(1),
                    inner.y + 1 + y,
                    char,
                    Style::new().fg(Color::DarkGrey),
                );
            }
        }
    }

    fn render_input(&self, state: &DemoState, area: Rect, buf: &mut Buffer) {
        let border_style = if state.focus == Panel::Input {
            Style::new().fg(Color::Cyan)
        } else {
            Style::new().fg(Color::DarkGrey)
        };

        let block = Block::bordered()
            .title("Input")
            .border_style(border_style);
        Widget::render(&block, area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        // Help text at top
        let help_text = if state.editor.show_slash_menu {
            "↑/↓: select | Enter: execute | Esc: cancel"
        } else {
            "Ctrl+C: quit | Ctrl+L: clear | / commands | Ctrl+Z: undo | Alt+b/f: word nav"
        };
        buf.set_string(
            inner.x,
            inner.y,
            help_text,
            Style::new().fg(Color::DarkGrey),
        );

        // Prompt on second line
        buf.set_string(inner.x, inner.y + 1, "▸ ", Style::new().fg(Color::Green));

        // Editor field - uses remaining height
        let editor_height = inner.height.saturating_sub(1); // Leave 1 line for help
        let editor_area = Rect::new(inner.x + 2, inner.y + 1, inner.width.saturating_sub(2), editor_height);
        let editor = Editor::new()
            .placeholder("Type a message... (/ for commands)")
            .style(Style::new().fg(Color::White));

        let mut editor_state = state.editor.clone();
        editor.render(editor_area, buf, &mut editor_state);

        // Render slash menu if active
        if state.editor.show_slash_menu {
            // Create the menu widget
            let filtered_commands: Vec<SlashCommand> = state.slash_commands
                .iter()
                .filter(|cmd| cmd.matches(&state.editor.slash_query))
                .cloned()
                .collect();

            if !filtered_commands.is_empty() {
                let menu = SlashMenu::new()
                    .commands(filtered_commands)
                    .query(&state.editor.slash_query);

                // Position the menu relative to the input area
                // Menu will render above the input
                let menu_area = Rect::new(
                    inner.x + 2,
                    area.y, // Use the full input area for positioning
                    inner.width.saturating_sub(2).min(50),
                    area.height,
                );

                let mut menu_state = state.slash_menu.clone();
                menu_state.visible = true;
                menu.render(menu_area, buf, &mut menu_state);
            }
        }
    }

    fn render_reasoning(&self, state: &DemoState, area: Rect, buf: &mut Buffer) {
        // Background
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                if let Some(cell) = buf.get_mut(x, y) {
                    cell.bg = Color::Rgb(25, 25, 35);
                }
            }
        }

        // Left border accent
        for y in area.y..area.bottom() {
            if let Some(cell) = buf.get_mut(area.x, y) {
                cell.symbol = "│".to_string();
                cell.fg = Color::Rgb(60, 60, 80);
            }
        }

        // Render reasoning lines with dynamic styling
        for (i, line) in state.reasoning.iter().enumerate() {
            if area.y + i as u16 >= area.bottom() {
                break;
            }

            // Style based on content
            let style = if line.starts_with('✓') {
                Style::new().fg(Color::Green)
            } else if line.contains("Running") || line.contains("Generating") {
                Style::new().fg(Color::Yellow)
            } else if line.starts_with('◇') || line.starts_with('◆') {
                Style::new().fg(Color::Rgb(100, 100, 120))
            } else if line.starts_with("  ") {
                Style::new().fg(Color::Rgb(80, 80, 100))
            } else {
                // Spinner or other - use yellow for activity
                Style::new().fg(Color::Yellow)
            };

            buf.set_string(area.x + 2, area.y + i as u16, line, style);
        }
    }

    fn render_status(&self, state: &DemoState, area: Rect, buf: &mut Buffer) {
        let status_style = match state.status.as_str() {
            s if s.contains("Error") || s.contains("Unknown") => Style::new().fg(Color::Red),
            s if s.contains("Running") || s.contains("Streaming") || s.contains("Generating") => {
                Style::new().fg(Color::Yellow)
            }
            _ => Style::new().fg(Color::Green),
        };

        let bar = StatusBar::new()
            .style(Style::new().bg(Color::DarkGrey).fg(Color::White))
            .left([
                StatusSection::new("spec-ai").style(Style::new().fg(Color::Cyan).bold()),
                StatusSection::new("demo").style(Style::new().fg(Color::DarkGrey)),
            ])
            .center([
                StatusSection::new(&state.status).style(status_style),
            ])
            .right([
                StatusSection::new(format!("msgs: {}", state.messages.len())),
                StatusSection::new(format!("tick: {}", state.tick)),
            ]);

        Widget::render(&bar, area, buf);
    }

    fn render_process_overlay(&self, state: &DemoState, area: Rect, buf: &mut Buffer) {
        // Calculate overlay dimensions (centered, 70% width, 60% height)
        let overlay_width = (area.width as f32 * 0.75) as u16;
        let overlay_height = (area.height as f32 * 0.65) as u16;
        let overlay_x = area.x + (area.width - overlay_width) / 2;
        let overlay_y = area.y + (area.height - overlay_height) / 2;
        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

        // Draw background
        for y in overlay_area.y..overlay_area.bottom() {
            for x in overlay_area.x..overlay_area.right() {
                if let Some(cell) = buf.get_mut(x, y) {
                    cell.bg = Color::Rgb(15, 15, 25);
                    cell.fg = Color::White;
                    cell.symbol = " ".to_string();
                }
            }
        }

        // Draw border
        let border_style = Style::new().fg(Color::Cyan);
        buf.set_string(overlay_area.x, overlay_area.y, "╭", border_style);
        buf.set_string(overlay_area.right() - 1, overlay_area.y, "╮", border_style);
        buf.set_string(overlay_area.x, overlay_area.bottom() - 1, "╰", border_style);
        buf.set_string(overlay_area.right() - 1, overlay_area.bottom() - 1, "╯", border_style);

        for x in (overlay_area.x + 1)..(overlay_area.right() - 1) {
            buf.set_string(x, overlay_area.y, "─", border_style);
            buf.set_string(x, overlay_area.bottom() - 1, "─", border_style);
        }
        for y in (overlay_area.y + 1)..(overlay_area.bottom() - 1) {
            buf.set_string(overlay_area.x, y, "│", border_style);
            buf.set_string(overlay_area.right() - 1, y, "│", border_style);
        }

        // Title with process count
        let running = state.processes.iter().filter(|p| p.status == ProcessStatus::Running).count();
        let title = format!(" Agent Processes ({} running) ", running);
        let title_x = overlay_area.x + (overlay_area.width - title.len() as u16) / 2;
        buf.set_string(title_x, overlay_area.y, &title, Style::new().fg(Color::Cyan).bold());

        let inner_x = overlay_area.x + 2;
        let inner_width = (overlay_area.width - 4) as usize;
        let mut y = overlay_area.y + 2;

        // Header row
        buf.set_string(inner_x, y, "PID", Style::new().fg(Color::DarkGrey).bold());
        buf.set_string(inner_x + 8, y, "AGENT", Style::new().fg(Color::DarkGrey).bold());
        buf.set_string(inner_x + 22, y, "COMMAND", Style::new().fg(Color::DarkGrey).bold());
        buf.set_string(overlay_area.right() - 12, y, "TIME", Style::new().fg(Color::DarkGrey).bold());
        y += 1;

        // Separator
        for x in inner_x..(overlay_area.right() - 2) {
            buf.set_string(x, y, "─", Style::new().fg(Color::Rgb(40, 40, 50)));
        }
        y += 1;

        if state.processes.is_empty() {
            buf.set_string(inner_x, y, "No agent processes running", Style::new().fg(Color::DarkGrey));
        } else {
            for (i, proc) in state.processes.iter().enumerate() {
                if y >= overlay_area.bottom() - 4 {
                    break;
                }

                let is_selected = i == state.selected_process;
                let bg = if is_selected { Color::Rgb(35, 35, 55) } else { Color::Rgb(15, 15, 25) };

                // Clear row
                for x in inner_x..(overlay_area.right() - 2) {
                    if let Some(cell) = buf.get_mut(x, y) {
                        cell.bg = bg;
                        cell.symbol = " ".to_string();
                    }
                }

                // Status icon
                let (icon, icon_color) = proc.status_icon();
                buf.set_string(inner_x, y, icon, Style::new().fg(icon_color));

                // PID
                let pid_style = if is_selected {
                    Style::new().fg(Color::White).bold()
                } else {
                    Style::new().fg(Color::White)
                };
                buf.set_string(inner_x + 2, y, &format!("{}", proc.pid), pid_style);

                // Agent name
                let agent_style = Style::new().fg(Color::Magenta);
                let agent: String = proc.agent.chars().take(12).collect();
                buf.set_string(inner_x + 8, y, &agent, agent_style);

                // Command (truncated)
                let cmd_width = inner_width.saturating_sub(35);
                let cmd = truncate_cmd(&proc.command, cmd_width);
                let cmd_style = if is_selected {
                    Style::new().fg(Color::Cyan)
                } else {
                    Style::new().fg(Color::White)
                };
                buf.set_string(inner_x + 22, y, &cmd, cmd_style);

                // Elapsed time
                let elapsed = proc.elapsed_display();
                let time_x = overlay_area.right() - 3 - elapsed.len() as u16;
                buf.set_string(time_x, y, &elapsed, Style::new().fg(Color::DarkGrey));

                y += 1;

                // Show last output line if selected
                if is_selected && !proc.output_lines.is_empty() {
                    for x in inner_x..(overlay_area.right() - 2) {
                        if let Some(cell) = buf.get_mut(x, y) {
                            cell.bg = Color::Rgb(25, 25, 35);
                            cell.symbol = " ".to_string();
                        }
                    }
                    let last_line = proc.output_lines.last().unwrap();
                    let output: String = last_line.chars().take(inner_width - 4).collect();
                    buf.set_string(inner_x + 2, y, &format!("└─ {}", output), Style::new().fg(Color::DarkGrey));
                    y += 1;
                }
            }
        }

        // Help text at bottom
        let help = "Enter: logs │ s: stop/cont │ x: kill │ d: remove │ Esc: close";
        buf.set_string(
            overlay_area.x + 2,
            overlay_area.bottom() - 2,
            help,
            Style::new().fg(Color::DarkGrey),
        );
    }

    fn render_history_overlay(&self, state: &DemoState, area: Rect, buf: &mut Buffer) {
        // Calculate overlay dimensions
        let overlay_width = (area.width as f32 * 0.6) as u16;
        let overlay_height = (area.height as f32 * 0.6) as u16;
        let overlay_x = area.x + (area.width - overlay_width) / 2;
        let overlay_y = area.y + (area.height - overlay_height) / 2;
        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

        // Draw background
        for y in overlay_area.y..overlay_area.bottom() {
            for x in overlay_area.x..overlay_area.right() {
                if let Some(cell) = buf.get_mut(x, y) {
                    cell.bg = Color::Rgb(20, 20, 30);
                    cell.fg = Color::White;
                    cell.symbol = " ".to_string();
                }
            }
        }

        // Draw border
        let border_style = Style::new().fg(Color::Magenta);
        buf.set_string(overlay_area.x, overlay_area.y, "╭", border_style);
        buf.set_string(overlay_area.right() - 1, overlay_area.y, "╮", border_style);
        buf.set_string(overlay_area.x, overlay_area.bottom() - 1, "╰", border_style);
        buf.set_string(overlay_area.right() - 1, overlay_area.bottom() - 1, "╯", border_style);

        for x in (overlay_area.x + 1)..(overlay_area.right() - 1) {
            buf.set_string(x, overlay_area.y, "─", border_style);
            buf.set_string(x, overlay_area.bottom() - 1, "─", border_style);
        }
        for y in (overlay_area.y + 1)..(overlay_area.bottom() - 1) {
            buf.set_string(overlay_area.x, y, "│", border_style);
            buf.set_string(overlay_area.right() - 1, y, "│", border_style);
        }

        // Title
        let title = " Session History ";
        let title_x = overlay_area.x + (overlay_area.width - title.len() as u16) / 2;
        buf.set_string(title_x, overlay_area.y, title, Style::new().fg(Color::Magenta).bold());

        // Render sessions
        let inner_x = overlay_area.x + 2;
        let inner_width = (overlay_area.width - 4) as usize;
        let mut y = overlay_area.y + 2;

        for (i, session) in state.sessions.iter().enumerate() {
            if y >= overlay_area.bottom() - 3 {
                break;
            }

            let is_selected = i == state.selected_session;
            let is_current = i == state.current_session;
            let bg = if is_selected { Color::Rgb(40, 40, 60) } else { Color::Rgb(20, 20, 30) };

            // Clear row
            for x in inner_x..(overlay_area.right() - 2) {
                if let Some(cell) = buf.get_mut(x, y) {
                    cell.bg = bg;
                    cell.symbol = " ".to_string();
                }
            }

            // Current session indicator
            let indicator = if is_current { "●" } else { "○" };
            let ind_style = if is_current {
                Style::new().fg(Color::Green)
            } else {
                Style::new().fg(Color::DarkGrey)
            };
            buf.set_string(inner_x, y, indicator, ind_style);

            // Session title
            let title_style = if is_selected {
                Style::new().fg(Color::White).bold()
            } else if is_current {
                Style::new().fg(Color::Green)
            } else {
                Style::new().fg(Color::White)
            };
            buf.set_string(inner_x + 2, y, &session.title, title_style);

            // Timestamp
            buf.set_string(
                overlay_area.right() - 3 - session.timestamp.len() as u16,
                y,
                &session.timestamp,
                Style::new().fg(Color::DarkGrey),
            );

            y += 1;

            // Preview and message count
            if y < overlay_area.bottom() - 3 {
                for x in inner_x..(overlay_area.right() - 2) {
                    if let Some(cell) = buf.get_mut(x, y) {
                        cell.bg = bg;
                        cell.symbol = " ".to_string();
                    }
                }

                let preview: String = session.preview.chars().take(inner_width - 15).collect();
                buf.set_string(inner_x + 2, y, &preview, Style::new().fg(Color::DarkGrey));

                // Message count
                let msg_count = if i == 0 {
                    format!("{} msgs", state.messages.len())
                } else {
                    format!("{} msgs", session.message_count)
                };
                buf.set_string(
                    overlay_area.right() - 3 - msg_count.len() as u16,
                    y,
                    &msg_count,
                    Style::new().fg(Color::DarkGrey),
                );

                y += 2;
            }
        }

        // Help text at bottom
        let help = "Enter: switch session | Esc: close";
        buf.set_string(
            overlay_area.x + 2,
            overlay_area.bottom() - 2,
            help,
            Style::new().fg(Color::DarkGrey),
        );
    }

    fn render_log_overlay(&self, state: &DemoState, area: Rect, buf: &mut Buffer) {
        let proc_idx = match state.viewing_logs {
            Some(idx) => idx,
            None => return,
        };
        let proc = match state.processes.get(proc_idx) {
            Some(p) => p,
            None => return,
        };

        // Calculate overlay dimensions (centered, 85% width, 80% height - larger for logs)
        let overlay_width = (area.width as f32 * 0.85) as u16;
        let overlay_height = (area.height as f32 * 0.80) as u16;
        let overlay_x = area.x + (area.width - overlay_width) / 2;
        let overlay_y = area.y + (area.height - overlay_height) / 2;
        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

        // Draw background
        for y in overlay_area.y..overlay_area.bottom() {
            for x in overlay_area.x..overlay_area.right() {
                if let Some(cell) = buf.get_mut(x, y) {
                    cell.bg = Color::Rgb(10, 10, 15);
                    cell.fg = Color::White;
                    cell.symbol = " ".to_string();
                }
            }
        }

        // Draw border
        let border_style = Style::new().fg(Color::Green);
        buf.set_string(overlay_area.x, overlay_area.y, "╭", border_style);
        buf.set_string(overlay_area.right() - 1, overlay_area.y, "╮", border_style);
        buf.set_string(overlay_area.x, overlay_area.bottom() - 1, "╰", border_style);
        buf.set_string(overlay_area.right() - 1, overlay_area.bottom() - 1, "╯", border_style);

        for x in (overlay_area.x + 1)..(overlay_area.right() - 1) {
            buf.set_string(x, overlay_area.y, "─", border_style);
            buf.set_string(x, overlay_area.bottom() - 1, "─", border_style);
        }
        for y in (overlay_area.y + 1)..(overlay_area.bottom() - 1) {
            buf.set_string(overlay_area.x, y, "│", border_style);
            buf.set_string(overlay_area.right() - 1, y, "│", border_style);
        }

        // Title with process info
        let (status_icon, status_color) = proc.status_icon();
        let title = format!(" {} PID {} │ {} │ {} ", status_icon, proc.pid, proc.agent, truncate_cmd(&proc.command, 40));
        let title_x = overlay_area.x + 2;
        buf.set_string(title_x, overlay_area.y, &title, Style::new().fg(status_color).bold());

        // Calculate visible area for log lines
        let inner_x = overlay_area.x + 2;
        let inner_width = (overlay_area.width - 4) as usize;
        let inner_height = (overlay_area.height - 4) as usize; // Leave room for title, separator, and help

        // Separator line below title
        let sep_y = overlay_area.y + 1;
        for x in (overlay_area.x + 1)..(overlay_area.right() - 1) {
            buf.set_string(x, sep_y, "─", Style::new().fg(Color::Rgb(40, 40, 50)));
        }

        // Get log lines and calculate scroll
        let total_lines = proc.output_lines.len();
        let visible_lines = inner_height;

        // Clamp scroll offset
        let max_scroll = total_lines.saturating_sub(visible_lines);
        let scroll = state.log_scroll.min(max_scroll);

        // Calculate which lines to show (from bottom, with scroll offset going up)
        let start = if total_lines > visible_lines + scroll {
            total_lines - visible_lines - scroll
        } else {
            0
        };
        let end = (start + visible_lines).min(total_lines);

        // Render log lines
        for (i, line_idx) in (start..end).enumerate() {
            let render_y = overlay_area.y + 2 + i as u16;
            if render_y >= overlay_area.bottom() - 2 {
                break;
            }

            let log_line = &proc.output_lines[line_idx];
            let truncated: String = log_line.chars().take(inner_width.saturating_sub(6)).collect();

            // Style based on content (simple heuristics)
            let style = if log_line.contains("error") || log_line.contains("Error") || log_line.contains("ERROR") {
                Style::new().fg(Color::Red)
            } else if log_line.contains("warn") || log_line.contains("WARN") {
                Style::new().fg(Color::Yellow)
            } else if log_line.contains("✓") || log_line.contains("ok") || log_line.contains("Ready") || log_line.contains("Finished") {
                Style::new().fg(Color::Green)
            } else if log_line.starts_with("   ") || log_line.starts_with("    ") {
                Style::new().fg(Color::DarkGrey)
            } else {
                Style::new().fg(Color::White)
            };

            // Line number
            let line_num = format!("{:>4} ", line_idx + 1);
            buf.set_string(inner_x, render_y, &line_num, Style::new().fg(Color::Rgb(60, 60, 70)));
            buf.set_string(inner_x + 5, render_y, &truncated, style);
        }

        // Scroll indicator
        if total_lines > visible_lines {
            let scrollbar_height = inner_height.saturating_sub(2) as u16;
            let thumb_pos = if total_lines > 0 && scrollbar_height > 0 {
                ((start as u32 * scrollbar_height as u32) / total_lines as u32) as u16
            } else {
                0
            };

            for i in 0..scrollbar_height {
                let char = if i == thumb_pos { "█" } else { "░" };
                buf.set_string(
                    overlay_area.right() - 2,
                    overlay_area.y + 2 + i,
                    char,
                    Style::new().fg(Color::Rgb(60, 60, 70)),
                );
            }

            // Scroll position indicator
            let scroll_info = format!("{}/{}", start + 1, total_lines);
            buf.set_string(
                overlay_area.right() - 3 - scroll_info.len() as u16,
                overlay_area.y,
                &scroll_info,
                Style::new().fg(Color::DarkGrey),
            );
        }

        // Help text at bottom
        let help = "↑/k: scroll up │ ↓/j: scroll down │ g/G: top/bottom │ Esc/q: close";
        buf.set_string(
            overlay_area.x + 2,
            overlay_area.bottom() - 2,
            help,
            Style::new().fg(Color::DarkGrey),
        );

        // Status indicator at bottom right
        let status_text = match proc.status {
            ProcessStatus::Running => "● LIVE",
            ProcessStatus::Stopped => "◉ PAUSED",
            ProcessStatus::Completed => "✓ DONE",
            ProcessStatus::Failed => "✗ FAILED",
        };
        buf.set_string(
            overlay_area.right() - 3 - status_text.len() as u16,
            overlay_area.bottom() - 2,
            status_text,
            Style::new().fg(status_color),
        );
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = DemoApp;
    let mut runner = AppRunner::new(app)?;

    runner.run().await
}
