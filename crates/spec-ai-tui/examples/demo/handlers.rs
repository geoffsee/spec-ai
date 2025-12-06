//! Event handling and tick logic for the demo app.

use crate::models::{ChatMessage, ProcessStatus, ToolExecution, ToolStatus};
use crate::state::{DemoState, OnboardingStep, Panel};
use spec_ai_tui::{
    event::{Event, KeyCode, KeyModifiers},
    style::truncate,
    widget::builtin::{EditorAction, Selection},
};

/// Actions represent state changes resulting from events or ticks.
/// This separates business logic (which actions to take) from state mutations (how to apply them).
#[derive(Debug, Clone)]
pub enum Action {
    // Quit & Status
    Quit,
    SetPendingQuit(bool),
    SetStatus(String),

    // Log Viewer
    OpenLogsForProcess(usize),
    CloseLogViewer,
    ScrollLogs(i32),

    // Overlay Panels
    ToggleProcessPanel,
    ToggleHistory,
    CloseOverlays,
    SelectProcess(usize),
    SelectSession(usize),

    // Session Management
    SwitchToSession(usize),
    SaveSessionMessages,

    // Process Management
    ToggleProcessStatus(usize),
    KillProcess(usize),
    RemoveProcess(usize),

    // Chat & Messages
    AddMessage(ChatMessage),
    ClearMessages,
    ScrollChat(i32),

    // Streaming & Tools
    StartStreaming,
    ProgressStreaming(String),
    CompleteStreaming(String),
    AddToolExecution(ToolExecution),
    ProgressTool(usize),
    CompleteTool(usize, String),

    // Listening
    StartListening,
    StopListening,
    ProgressListening(String),
    AddListenLog(String),

    // Focus & UI
    ChangeFocus(Panel),
    ResetScroll,

    // Onboarding
    NextOnboardingStep,
    PrevOnboardingStep,
    SelectProvider(usize),
    SelectModelKind(usize),
    ToggleVoice,
    ToggleFileWrite,
    CyclePolicy,
    ShowPolicyModal,
    HidePolicyModal,
    FinalizeOnboarding,

    // Animations & Ticks
    UpdateTick,
    UpdateReasoningDuringOnboarding(String, String, String),
    UpdateIdleReasoning(String, String, String),
    UpdateStreamingReasoning(String, String, String),
    UpdateToolReasoning(String, String, String),
    UpdateListeningReasoning(String, String, String),
}

/// Applies a list of actions to the state, mutating it accordingly.
pub fn apply_actions(state: &mut DemoState, actions: Vec<Action>) {
    for action in actions {
        match action {
            Action::Quit => state.quit = true,
            Action::SetPendingQuit(pending) => state.pending_quit = pending,
            Action::SetStatus(status) => state.status = status,

            Action::OpenLogsForProcess(idx) => {
                state.viewing_logs = Some(idx);
                state.log_scroll = 0;
            }
            Action::CloseLogViewer => state.viewing_logs = None,
            Action::ScrollLogs(delta) => {
                if delta > 0 {
                    state.log_scroll = state.log_scroll.saturating_add(delta as usize);
                } else {
                    state.log_scroll = state.log_scroll.saturating_sub((-delta) as usize);
                }
            }

            Action::ToggleProcessPanel => {
                state.show_process_panel = !state.show_process_panel;
                state.show_history = false;
            }
            Action::ToggleHistory => {
                state.show_history = !state.show_history;
                state.show_process_panel = false;
            }
            Action::CloseOverlays => {
                state.show_process_panel = false;
                state.show_history = false;
                state.viewing_logs = None;
            }
            Action::SelectProcess(idx) => state.selected_process = idx,
            Action::SelectSession(idx) => state.selected_session = idx,

            Action::SwitchToSession(idx) => {
                if idx != state.current_session && idx < state.sessions.len() {
                    if state.current_session == 0 {
                        state.sessions[0].messages = state.messages.clone();
                        state.sessions[0].message_count = state.messages.len();
                    }
                    state.current_session = idx;
                    state.messages = state.sessions[idx].messages.clone();
                    state.scroll_offset = 0;
                }
            }
            Action::SaveSessionMessages => {
                if state.current_session == 0 {
                    state.sessions[0].messages = state.messages.clone();
                    state.sessions[0].message_count = state.messages.len();
                }
            }

            Action::ToggleProcessStatus(idx) => {
                if let Some(proc) = state.processes.get_mut(idx) {
                    match proc.status {
                        ProcessStatus::Running => proc.status = ProcessStatus::Stopped,
                        ProcessStatus::Stopped => proc.status = ProcessStatus::Running,
                        _ => {}
                    }
                }
            }
            Action::KillProcess(idx) => {
                if let Some(proc) = state.processes.get_mut(idx) {
                    if proc.status == ProcessStatus::Running
                        || proc.status == ProcessStatus::Stopped
                    {
                        proc.status = ProcessStatus::Failed;
                        proc.exit_code = Some(-9);
                    }
                }
            }
            Action::RemoveProcess(idx) => {
                if idx < state.processes.len() {
                    state.processes.remove(idx);
                    if state.selected_process > 0 && state.selected_process >= state.processes.len()
                    {
                        state.selected_process -= 1;
                    }
                }
            }

            Action::AddMessage(msg) => state.messages.push(msg),
            Action::ClearMessages => state.messages.clear(),
            Action::ScrollChat(delta) => {
                if delta > 0 {
                    state.scroll_offset = state.scroll_offset.saturating_add(delta as u16);
                } else {
                    state.scroll_offset = state.scroll_offset.saturating_sub((-delta) as u16);
                }
            }

            Action::StartStreaming => {
                state.streaming = Some(String::new());
                state.stream_index = 0;
            }
            Action::ProgressStreaming(chunk) => {
                if let Some(ref mut s) = state.streaming {
                    s.push_str(&chunk);
                }
                state.stream_index += 1;
            }
            Action::CompleteStreaming(response) => {
                state.streaming = None;
                state.stream_index = 0;
                let timestamp = format!(
                    "{}:{:02}",
                    10 + state.messages.len() / 60,
                    state.messages.len() % 60
                );
                state
                    .messages
                    .push(ChatMessage::new("assistant", &response, &timestamp));
            }
            Action::AddToolExecution(tool) => state.tools.push(tool),
            Action::ProgressTool(idx) => {
                if let Some(tool) = state.tools.get_mut(idx) {
                    if tool.duration_ms.is_none() {
                        tool.duration_ms = Some(0);
                    }
                    tool.duration_ms = Some(tool.duration_ms.unwrap() + 100);
                }
            }
            Action::CompleteTool(idx, content) => {
                if let Some(tool) = state.tools.get_mut(idx) {
                    tool.status = ToolStatus::Success;
                }
                let timestamp = format!(
                    "{}:{:02}",
                    10 + state.messages.len() / 60,
                    state.messages.len() % 60
                );
                if let Some(tool) = state.tools.get(idx) {
                    state
                        .messages
                        .push(ChatMessage::tool(&tool.name, &content, &timestamp));
                }
            }

            Action::StartListening => {
                state.listening = true;
                state.listen_index = 0;
                state.listen_log.clear();
            }
            Action::StopListening => {
                state.listening = false;
                state.listen_index = 0;
            }
            Action::ProgressListening(transcript) => {
                state.listen_index += 1;
                state.listen_log.push(transcript);
                if state.listen_log.len() > 6 {
                    state.listen_log.remove(0);
                }
            }
            Action::AddListenLog(msg) => {
                state.listen_log.push(msg);
                if state.listen_log.len() > 6 {
                    state.listen_log.remove(0);
                }
            }

            Action::ChangeFocus(panel) => {
                state.focus = panel;
                if panel == Panel::Input {
                    state.editor.focused = true;
                } else {
                    state.editor.focused = false;
                }
            }
            Action::ResetScroll => state.scroll_offset = 0,

            Action::NextOnboardingStep => {
                if state.onboarding.step == OnboardingStep::Provider {
                    state.onboarding.step = OnboardingStep::Model;
                    state.onboarding.reset_model_cursor();
                    state.onboarding.confirm_cursor = 0;
                } else if state.onboarding.step == OnboardingStep::Model {
                    state.onboarding.step = OnboardingStep::Confirm;
                    state.onboarding.confirm_cursor = 0;
                }
            }
            Action::PrevOnboardingStep => {
                if state.onboarding.step == OnboardingStep::Model {
                    state.onboarding.step = OnboardingStep::Provider;
                    state.onboarding.reset_model_cursor();
                } else if state.onboarding.step == OnboardingStep::Confirm {
                    state.onboarding.step = OnboardingStep::Model;
                }
            }
            Action::SelectProvider(idx) => {
                state.onboarding.selected_provider = idx;
                state.onboarding.reset_model_cursor();
            }
            Action::SelectModelKind(idx) => state.onboarding.selected_kind = idx,
            Action::ToggleVoice => state.onboarding.voice_enabled = !state.onboarding.voice_enabled,
            Action::ToggleFileWrite => {
                let _ = toggle_file_write(&mut state.onboarding.selected_tools);
            }
            Action::CyclePolicy => {
                state.onboarding.policy_mode = state.onboarding.policy_mode.next()
            }
            Action::ShowPolicyModal => state.onboarding.show_policy_modal = true,
            Action::HidePolicyModal => state.onboarding.show_policy_modal = false,
            Action::FinalizeOnboarding => finalize_onboarding(state),

            Action::UpdateTick => state.tick += 1,
            Action::UpdateReasoningDuringOnboarding(r0, r1, r2) => {
                state.reasoning[0] = r0;
                state.reasoning[1] = r1;
                state.reasoning[2] = r2;
            }
            Action::UpdateIdleReasoning(r0, r1, r2) => {
                state.reasoning[0] = r0;
                state.reasoning[1] = r1;
                state.reasoning[2] = r2;
            }
            Action::UpdateStreamingReasoning(r0, r1, r2) => {
                state.reasoning[0] = r0;
                state.reasoning[1] = r1;
                state.reasoning[2] = r2;
            }
            Action::UpdateToolReasoning(r0, r1, r2) => {
                state.reasoning[0] = r0;
                state.reasoning[1] = r1;
                state.reasoning[2] = r2;
            }
            Action::UpdateListeningReasoning(r0, r1, r2) => {
                state.reasoning[0] = r0;
                state.reasoning[1] = r1;
                state.reasoning[2] = r2;
            }
        }
    }
}

/// Pure business logic for handling events. Returns actions to be applied to state.
/// This is testable and has no side effects.
pub fn handle_event_pure(event: Event, state: &DemoState) -> (Vec<Action>, bool) {
    let mut actions = Vec::new();
    let mut consumed = true;

    // Handle quit (Ctrl+C)
    if event.is_quit() {
        if state.pending_quit {
            actions.push(Action::Quit);
            return (actions, false); // Don't consume quit on second Ctrl+C to signal exit
        } else {
            actions.push(Action::SetPendingQuit(true));
            actions.push(Action::SetStatus("Press Ctrl+C again to exit".to_string()));
            return (actions, true);
        }
    }

    // Reset pending_quit on any key input (not tick events)
    if state.pending_quit {
        if let Event::Key(_) = event {
            actions.push(Action::SetPendingQuit(false));
            actions.push(Action::SetStatus("Ready".to_string()));
        }
    }

    if state.onboarding.active {
        // Delegate to onboarding handler
        let (mut onboarding_actions, onboarding_consumed) =
            handle_onboarding_event_pure(event, state);
        actions.append(&mut onboarding_actions);
        return (actions, onboarding_consumed);
    }

    match event {
        Event::Key(key) => {
            // Handle log viewing overlay (highest priority - sub-overlay of process panel)
            if let Some(proc_idx) = state.viewing_logs {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        actions.push(Action::CloseLogViewer);
                        return (actions, true);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        actions.push(Action::ScrollLogs(1));
                        return (actions, true);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        actions.push(Action::ScrollLogs(-1));
                        return (actions, true);
                    }
                    KeyCode::PageUp => {
                        actions.push(Action::ScrollLogs(10));
                        return (actions, true);
                    }
                    KeyCode::PageDown => {
                        actions.push(Action::ScrollLogs(-10));
                        return (actions, true);
                    }
                    KeyCode::Char('g') => {
                        // Jump to top (oldest logs)
                        if let Some(proc) = state.processes.get(proc_idx) {
                            actions.push(Action::ScrollLogs(proc.output_lines.len() as i32));
                        }
                        return (actions, true);
                    }
                    KeyCode::Char('G') => {
                        // Jump to bottom (newest logs)
                        actions.push(Action::ScrollLogs(-(state.log_scroll as i32)));
                        return (actions, true);
                    }
                    _ => return (actions, true), // Consume all keys when log view is open
                }
            }

            // Handle overlay panels first (Escape to close)
            if state.show_process_panel || state.show_history {
                match key.code {
                    KeyCode::Esc => {
                        actions.push(Action::CloseOverlays);
                        return (actions, true);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if state.show_process_panel {
                            actions.push(Action::SelectProcess(
                                state.selected_process.saturating_sub(1),
                            ));
                        } else if state.show_history {
                            actions.push(Action::SelectSession(
                                state.selected_session.saturating_sub(1),
                            ));
                        }
                        return (actions, true);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if state.show_process_panel {
                            let max = state.processes.len().saturating_sub(1);
                            actions
                                .push(Action::SelectProcess((state.selected_process + 1).min(max)));
                        } else if state.show_history {
                            let max = state.sessions.len().saturating_sub(1);
                            actions
                                .push(Action::SelectSession((state.selected_session + 1).min(max)));
                        }
                        return (actions, true);
                    }
                    KeyCode::Enter => {
                        if state.show_history && state.selected_session < state.sessions.len() {
                            if state.selected_session != state.current_session {
                                actions.push(Action::SaveSessionMessages);
                                actions.push(Action::SwitchToSession(state.selected_session));
                                actions.push(Action::SetStatus(format!(
                                    "Switched to: {}",
                                    state.sessions[state.selected_session].title
                                )));
                                actions.push(Action::CloseOverlays);
                            }
                        } else if state.show_process_panel
                            && state.selected_process < state.processes.len()
                        {
                            actions.push(Action::OpenLogsForProcess(state.selected_process));
                            if let Some(proc) = state.processes.get(state.selected_process) {
                                actions.push(Action::SetStatus(format!(
                                    "Logs: PID {} (↑↓ scroll, g/G top/bottom, Esc close)",
                                    proc.pid
                                )));
                            }
                        }
                        return (actions, true);
                    }
                    KeyCode::Char('s') if state.show_process_panel => {
                        if state.selected_process < state.processes.len() {
                            actions.push(Action::ToggleProcessStatus(state.selected_process));
                            if let Some(proc) = state.processes.get(state.selected_process) {
                                let status_str = match proc.status {
                                    ProcessStatus::Running => "Stopped",
                                    ProcessStatus::Stopped => "Continued",
                                    _ => "Unknown",
                                };
                                actions.push(Action::SetStatus(format!(
                                    "{} PID {}",
                                    status_str, proc.pid
                                )));
                            }
                        }
                        return (actions, true);
                    }
                    KeyCode::Char('x') if state.show_process_panel => {
                        if state.selected_process < state.processes.len() {
                            actions.push(Action::KillProcess(state.selected_process));
                            if let Some(proc) = state.processes.get(state.selected_process) {
                                actions.push(Action::SetStatus(format!(
                                    "Killed PID {} (SIGKILL)",
                                    proc.pid
                                )));
                            }
                        }
                        return (actions, true);
                    }
                    KeyCode::Char('d') if state.show_process_panel => {
                        if state.selected_process < state.processes.len() {
                            if let Some(proc) = state.processes.get(state.selected_process) {
                                if proc.status == ProcessStatus::Completed
                                    || proc.status == ProcessStatus::Failed
                                {
                                    let pid = proc.pid;
                                    actions.push(Action::RemoveProcess(state.selected_process));
                                    actions.push(Action::SetStatus(format!(
                                        "Removed PID {} from list",
                                        pid
                                    )));
                                }
                            }
                        }
                        return (actions, true);
                    }
                    _ => return (actions, true), // Consume all keys when overlay is open
                }
            }
        }
        Event::Tick => {
            // Will be handled by on_tick_pure
        }
        _ => {
            consumed = false;
        }
    }

    (actions, consumed)
}

/// Onboarding event handler (pure version)
fn handle_onboarding_event_pure(event: Event, state: &DemoState) -> (Vec<Action>, bool) {
    let mut actions = Vec::new();

    match event {
        Event::Key(key) => match state.onboarding.step {
            OnboardingStep::Provider => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    actions.push(Action::SelectProvider(
                        state.onboarding.selected_provider.saturating_sub(1),
                    ));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let max = state.onboarding.providers.len().saturating_sub(1);
                    actions.push(Action::SelectProvider(
                        (state.onboarding.selected_provider + 1).min(max),
                    ));
                }
                KeyCode::Enter => {
                    actions.push(Action::NextOnboardingStep);
                    if let Some(provider) = state.onboarding.current_provider() {
                        let models = state.onboarding.model_count_for_provider();
                        actions.push(Action::SetStatus(format!(
                            "Pulled {} models for {} (step 2/3)",
                            models, provider.name
                        )));
                    }
                }
                _ => {}
            },
            OnboardingStep::Model => match key.code {
                KeyCode::Enter => {
                    if state.onboarding.current_model().is_some() {
                        actions.push(Action::NextOnboardingStep);
                    }
                }
                KeyCode::Esc | KeyCode::Left => {
                    actions.push(Action::PrevOnboardingStep);
                }
                _ => {}
            },
            OnboardingStep::Confirm => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => match state.onboarding.confirm_cursor {
                    0 => {
                        actions.push(Action::ToggleVoice);
                    }
                    1 => {
                        actions.push(Action::ToggleFileWrite);
                    }
                    2 => {
                        actions.push(Action::CyclePolicy);
                    }
                    _ => {
                        actions.push(Action::FinalizeOnboarding);
                    }
                },
                KeyCode::Char('p') => {
                    actions.push(Action::ShowPolicyModal);
                }
                _ => {}
            },
        },
        _ => {}
    }

    (actions, true)
}

pub fn handle_event(event: Event, state: &mut DemoState) -> bool {
    // For now, use the original implementation to ensure backward compatibility
    // This will be refactored to use handle_event_pure once it's complete

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

    if state.onboarding.active {
        return handle_onboarding_event(event, state);
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
                                    state.messages =
                                        state.sessions[state.selected_session].messages.clone();
                                }
                                state.status = format!(
                                    "Switched to: {}",
                                    state.sessions[state.selected_session].title
                                );
                                state.scroll_offset = 0;
                            }
                            state.show_history = false;
                        } else if state.show_process_panel
                            && state.selected_process < state.processes.len()
                        {
                            // Open log view for selected process
                            state.viewing_logs = Some(state.selected_process);
                            state.log_scroll = 0;
                            if let Some(proc) = state.processes.get(state.selected_process) {
                                state.status = format!(
                                    "Logs: PID {} (↑↓ scroll, g/G top/bottom, Esc close)",
                                    proc.pid
                                );
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
                                    state.status = format!(
                                        "Stopped PID {}: {}",
                                        proc.pid,
                                        truncate(&proc.command, 30)
                                    );
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
                            if proc.status == ProcessStatus::Running
                                || proc.status == ProcessStatus::Stopped
                            {
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
                            if proc.status == ProcessStatus::Completed
                                || proc.status == ProcessStatus::Failed
                            {
                                let pid = proc.pid;
                                state.processes.remove(state.selected_process);
                                if state.selected_process > 0
                                    && state.selected_process >= state.processes.len()
                                {
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
                            state.status =
                                "Processes (↑↓ nav, Enter stop/cont, x kill, d remove, Esc close)"
                                    .to_string();
                        }
                        return true;
                    }
                    KeyCode::Char('h') => {
                        // Toggle history panel
                        state.show_history = !state.show_history;
                        state.show_process_panel = false;
                        if state.show_history {
                            state.status =
                                "Session history (↑↓ select, Enter switch, Esc close)".to_string();
                        }
                        return true;
                    }
                    KeyCode::Char('l') => {
                        // Clear chat
                        state.messages.clear();
                        state.status = "Chat cleared".to_string();
                        return true;
                    }
                    KeyCode::Char('a') => {
                        // Toggle mock listening mode
                        toggle_listening(state);
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
                    KeyCode::Char('p') => {
                        // Simulate model prompting for user input
                        let timestamp = format!(
                            "{}:{:02}",
                            10 + state.messages.len() / 60,
                            state.messages.len() % 60
                        );
                        state.messages.push(ChatMessage::prompt(
                            "I found multiple matching files. Which one would you like me to read?\n\n\
                             1. src/main.rs (entry point)\n\
                             2. src/lib.rs (library root)\n\
                             3. src/config.rs (configuration)",
                            &timestamp,
                        ));
                        state.status = "Awaiting user response...".to_string();
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
                    KeyCode::Tab => {
                        if complete_slash_command(state) {
                            return true;
                        }
                        let count = filtered_command_count(state);
                        state.slash_menu.next(count);
                        return true;
                    }
                    KeyCode::BackTab => {
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
                            // If input is a slash command, execute it directly
                            let trimmed = text.trim();
                            if let Some(cmd) = trimmed.strip_prefix('/') {
                                let cmd_name = cmd.split_whitespace().next().unwrap_or("");
                                if !cmd_name.is_empty() {
                                    execute_slash_command(cmd_name, state);
                                    state.editor.clear();
                                    state.slash_menu.hide();
                                    return true;
                                }
                            }

                            if !text.is_empty() {
                                // Add user message
                                let timestamp = format!(
                                    "{}:{:02}",
                                    10 + state.messages.len() / 60,
                                    state.messages.len() % 60
                                );
                                state
                                    .messages
                                    .push(ChatMessage::new("user", &text, &timestamp));

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
                                    state.focus = Panel::Agent;
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
                Panel::Agent => {
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

fn handle_onboarding_event(event: Event, state: &mut DemoState) -> bool {
    if state.onboarding.show_policy_modal {
        return handle_policy_modal_event(event, state);
    }

    match event {
        Event::Key(key) => match state.onboarding.step {
            OnboardingStep::Provider => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    state.onboarding.selected_provider =
                        state.onboarding.selected_provider.saturating_sub(1);
                    state.onboarding.reset_model_cursor();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let max = state.onboarding.providers.len().saturating_sub(1);
                    state.onboarding.selected_provider =
                        (state.onboarding.selected_provider + 1).min(max);
                    state.onboarding.reset_model_cursor();
                }
                KeyCode::Enter => {
                    state.onboarding.step = OnboardingStep::Model;
                    state.onboarding.reset_model_cursor();
                    state.onboarding.confirm_cursor = 0;
                    if let Some(provider) = state.onboarding.current_provider() {
                        let models = state.onboarding.model_count_for_provider();
                        state.status =
                            format!("Pulled {} models for {} (step 2/3)", models, provider.name);
                    }
                }
                _ => {}
            },
            OnboardingStep::Model => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(kind) = state.onboarding.current_kind() {
                        let current = state
                            .onboarding
                            .selected_models
                            .get(&kind)
                            .copied()
                            .unwrap_or(0);
                        let new_idx = current.saturating_sub(1);
                        state.onboarding.selected_models.insert(kind, new_idx);
                    }
                }
                KeyCode::Left => {
                    state.onboarding.selected_kind =
                        state.onboarding.selected_kind.saturating_sub(1);
                }
                KeyCode::Right => {
                    let max = state.onboarding.provider_kinds().len().saturating_sub(1);
                    state.onboarding.selected_kind = (state.onboarding.selected_kind + 1).min(max);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(kind) = state.onboarding.current_kind() {
                        let models = state.onboarding.models_for_current();
                        if models.is_empty() {
                            state.status = format!("No {} models for this provider", kind.label());
                        } else {
                            let max = models.len().saturating_sub(1);
                            let current = state
                                .onboarding
                                .selected_models
                                .get(&kind)
                                .copied()
                                .unwrap_or(0);
                            state
                                .onboarding
                                .selected_models
                                .insert(kind, (current + 1).min(max));
                        }
                    }
                }
                KeyCode::Enter => {
                    let model_name = state.onboarding.current_model().map(|m| m.name.clone());
                    if let Some(model_name) = model_name {
                        state.onboarding.step = OnboardingStep::Confirm;
                        state.onboarding.confirm_cursor = 0;
                        let kind_label = state.onboarding.current_kind_label();
                        state.status = format!(
                            "Selected {} ({}) • confirm tools/voice (step 3/3)",
                            model_name, kind_label
                        );
                    } else {
                        state.status = "No models detected for this provider".to_string();
                    }
                }
                KeyCode::Esc | KeyCode::Left => {
                    state.onboarding.step = OnboardingStep::Provider;
                    state.onboarding.reset_model_cursor();
                    state.status = "Back to provider selection (step 1/3)".to_string();
                }
                _ => {}
            },
            OnboardingStep::Confirm => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    state.onboarding.confirm_cursor =
                        state.onboarding.confirm_cursor.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let max = state.onboarding.confirm_options_len().saturating_sub(1);
                    state.onboarding.confirm_cursor =
                        (state.onboarding.confirm_cursor + 1).min(max);
                }
                KeyCode::Esc | KeyCode::Left => {
                    state.onboarding.step = OnboardingStep::Model;
                    state.status = "Back to model selection (step 2/3)".to_string();
                }
                KeyCode::Char('p') => {
                    state.onboarding.show_policy_modal = true;
                    state.status =
                        "Policy editor open (←/→ policy, t toggle file_write, Enter to close)"
                            .to_string();
                }
                KeyCode::Enter | KeyCode::Char(' ') => match state.onboarding.confirm_cursor {
                    0 => {
                        state.onboarding.voice_enabled = !state.onboarding.voice_enabled;
                        state.status = format!(
                            "Voice {}",
                            if state.onboarding.voice_enabled {
                                "enabled"
                            } else {
                                "disabled"
                            }
                        );
                    }
                    1 => {
                        let enabled = toggle_file_write(&mut state.onboarding.selected_tools);
                        state.status = if enabled {
                            "Enabled file_write tool (demo)".to_string()
                        } else {
                            "Removed file_write tool (demo)".to_string()
                        };
                    }
                    2 => {
                        state.onboarding.policy_mode = state.onboarding.policy_mode.next();
                        state.status = format!("Policy: {}", state.onboarding.policy_mode.label());
                    }
                    _ => finalize_onboarding(state),
                },
                _ => {}
            },
        },
        _ => {}
    }

    true
}

fn toggle_file_write(tools: &mut Vec<String>) -> bool {
    if tools.iter().any(|t| t == "file_write") {
        tools.retain(|t| t != "file_write");
        false
    } else {
        tools.push("file_write".to_string());
        true
    }
}

fn handle_policy_modal_event(event: Event, state: &mut DemoState) -> bool {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Char(' ') => {
                state.onboarding.policy_mode = state.onboarding.policy_mode.next();
                state.status = format!("Policy set to {}", state.onboarding.policy_mode.label());
            }
            KeyCode::Char('t') => {
                let enabled = toggle_file_write(&mut state.onboarding.selected_tools);
                state.status = if enabled {
                    "Enabled file_write tool (policy editor)".to_string()
                } else {
                    "Removed file_write tool (policy editor)".to_string()
                };
            }
            KeyCode::Enter | KeyCode::Esc => {
                state.onboarding.show_policy_modal = false;
                state.status = "Policy editor closed".to_string();
            }
            _ => {}
        },
        _ => {}
    }

    true
}

fn finalize_onboarding(state: &mut DemoState) {
    state.voice_enabled = state.onboarding.voice_enabled;
    state.policy_mode = state.onboarding.policy_mode;
    state.allowed_tools = state.onboarding.selected_tools.clone();
    state.onboarding.active = false;
    state.focus = Panel::Input;
    state.editor.focused = true;

    let provider = state
        .onboarding
        .current_provider()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "unknown-provider".to_string());
    let selections: Vec<String> = state
        .onboarding
        .provider_kinds()
        .into_iter()
        .filter_map(|k| {
            state
                .onboarding
                .model_for_kind(k)
                .map(|m| format!("{}: {}", k.label(), m.name))
        })
        .collect();
    let model_summary = if selections.is_empty() {
        "default models".to_string()
    } else {
        selections.join(" | ")
    };

    let voice_label = if state.voice_enabled { "on" } else { "off" };
    let tools_summary = if state.allowed_tools.is_empty() {
        "no tools".to_string()
    } else {
        state.allowed_tools.join(", ")
    };

    state.status = format!(
        "Setup complete • {} • {} • voice {} • {}",
        provider,
        model_summary,
        voice_label,
        state.policy_mode.label()
    );

    state.reasoning[0] = format!("✓ Provider ready: {}", provider);
    state.reasoning[1] = format!("  Models: {} | Voice: {}", model_summary, voice_label);
    state.reasoning[2] = format!(
        "  Tools: {} | Policy: {}",
        tools_summary,
        state.policy_mode.label()
    );

    let timestamp = format!(
        "{}:{:02}",
        10 + state.messages.len() / 60,
        state.messages.len() % 60
    );
    state.messages.push(ChatMessage::new(
        "system",
        &format!(
            "Connected to {provider} using [{model_summary}]. Voice: {voice_label}. Policy: {}. Tools: {tools_summary}.",
            state.policy_mode.label()
        ),
        &timestamp,
    ));
}

pub fn on_tick(state: &mut DemoState) {
    state.tick += 1;

    // Spinner frames for animation
    let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spin_char = spinner[(state.tick / 2) as usize % spinner.len()];

    if state.onboarding.active {
        let step_label = match state.onboarding.step {
            OnboardingStep::Provider => "Select a provider",
            OnboardingStep::Model => "Choose chat + fast + embeddings + audio",
            OnboardingStep::Confirm => "Confirm tools & voice",
        };

        state.reasoning[0] = format!("{} Setup: {}", spin_char, step_label);
        state.reasoning[1] = "  ↑/↓ to move, Enter to confirm".to_string();
        state.reasoning[2] = match state.onboarding.step {
            OnboardingStep::Provider => "  Detected providers from config".to_string(),
            OnboardingStep::Model => "  ←/→ type • ↑/↓ model".to_string(),
            OnboardingStep::Confirm => "  Space toggles, Enter to start".to_string(),
        };
        return;
    }

    // Simulate streaming
    if let Some(ref mut streaming) = state.streaming {
        if state.stream_index < state.stream_buffer.len() {
            streaming.push_str(state.stream_buffer[state.stream_index]);
            state.stream_index += 1;

            // Update reasoning during streaming
            let tokens_out = streaming.split_whitespace().count();
            state.reasoning[0] = format!("{} Generating response...", spin_char);
            state.reasoning[1] = format!("  Tokens: ~{} output", tokens_out);
            state.reasoning[2] = format!(
                "  Progress: {}/{} chunks",
                state.stream_index,
                state.stream_buffer.len()
            );
        } else {
            // Streaming complete
            let response = state.streaming.take().unwrap();
            let timestamp = format!(
                "{}:{:02}",
                10 + state.messages.len() / 60,
                state.messages.len() % 60
            );
            state
                .messages
                .push(ChatMessage::new("assistant", &response, &timestamp));
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

                // Emit condensed tool result into chat
                let timestamp = format!(
                    "{}:{:02}",
                    10 + state.messages.len() / 60,
                    state.messages.len() % 60
                );
                let content = match tool.name.as_str() {
                    "code_search" => {
                        "Searching for: \"fn main\" in workspace\n\
                         Found 3 results:\n\
                         → crates/spec-ai-tui/examples/demo.rs: async fn main()\n\
                         → crates/spec-ai-cli/src/main.rs: fn main()\n\
                         → crates/spec-ai-api/src/lib.rs: pub fn run()\n\
                         Showing top matches..."
                    }
                    "file_read" => {
                        "Reading: crates/spec-ai-tui/src/lib.rs (first 40 lines)\n\
                         use spec_ai_tui::app::App;\n\
                         use spec_ai_tui::buffer::Buffer;\n\
                         use spec_ai_tui::geometry::Rect;\n\
                         pub struct DemoApp; ..."
                    }
                    _ => "Tool execution finished successfully.",
                };
                state
                    .messages
                    .push(ChatMessage::tool(&tool.name, content, &timestamp));
            } else {
                state.reasoning[0] = format!("{} Running {}...", spin_char, tool.name);
                state.reasoning[1] = format!("  Elapsed: {}ms", tool.duration_ms.unwrap());
                state.reasoning[2] = "  Waiting for results".to_string();
            }
        }
    }

    // Simulated listening (mock audio transcription)
    if state.listening {
        any_running = true;

        if state.tick % 5 == 0 && state.listen_index < state.listen_buffer.len() {
            let transcript = state.listen_buffer[state.listen_index];
            state.listen_index += 1;

            state.listen_log.push(transcript.to_string());
            if state.listen_log.len() > 6 {
                state.listen_log.remove(0);
            }
            state.status = "Listening (mock mic input)".to_string();
            state.reasoning[0] = format!("{} Listening (mock)...", spin_char);
            state.reasoning[1] = format!("  Segments captured: {}", state.listen_index);
            state.reasoning[2] = "  /listen to stop".to_string();
        } else if state.listen_index >= state.listen_buffer.len() {
            state
                .listen_log
                .push("Listening complete. Saved mock transcripts.".to_string());
            if state.listen_log.len() > 6 {
                state.listen_log.remove(0);
            }
            stop_listening(state, "Listening complete (mock)");
        } else {
            state.reasoning[0] = format!("{} Listening (mock)...", spin_char);
            state.reasoning[1] = format!("  Segments captured: {}", state.listen_index);
            state.reasoning[2] = "  /listen to stop".to_string();
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

fn filtered_command_count(state: &DemoState) -> usize {
    state
        .slash_commands
        .iter()
        .filter(|cmd| cmd.matches(&state.editor.slash_query))
        .count()
}

fn selected_slash_command(state: &DemoState) -> Option<String> {
    let filtered: Vec<_> = state
        .slash_commands
        .iter()
        .filter(|c| c.matches(&state.editor.slash_query))
        .collect();

    filtered
        .get(state.slash_menu.selected_index())
        .map(|c| c.name.clone())
}

fn complete_slash_command(state: &mut DemoState) -> bool {
    if let Some(cmd) = selected_slash_command(state) {
        let text = format!("/{cmd}");
        state.editor.text = text.clone();
        state.editor.selection = Selection::cursor(text.len());
        state.editor.show_slash_menu = false;
        state.editor.slash_query.clear();
        state.slash_menu.hide();
        state.status = format!("Prepared /{} (Enter to run, add args manually)", cmd);
        true
    } else {
        false
    }
}

fn toggle_listening(state: &mut DemoState) {
    if state.listening {
        stop_listening(state, "Listening stopped (mock)");
    } else {
        start_listening(state);
    }
}

fn start_listening(state: &mut DemoState) {
    state.listening = true;
    state.listen_index = 0;
    state.listen_log.clear();
    state.status = "Listening (mock mic input)...".to_string();
    state.reasoning[0] = "◇ Listening (mock mic)".to_string();
    state.reasoning[1] = "  Capturing microphone input".to_string();
    state.reasoning[2] = "  /listen to stop".to_string();

    state
        .listen_log
        .push("Started mock listening session (demo only)".to_string());
}

fn stop_listening(state: &mut DemoState, status: &str) {
    if !state.listening {
        return;
    }
    state.listening = false;
    state.listen_index = 0;
    state.status = status.to_string();
    state.reasoning[0] = "✓ Listening stopped".to_string();
    state.reasoning[1] = "  Transcriptions paused".to_string();
    state.reasoning[2] = "  Use /listen to start again".to_string();
}

fn execute_slash_command(cmd: &str, state: &mut DemoState) {
    // Find the command that matches
    let selected_cmd = selected_slash_command(state)
        .as_deref()
        .unwrap_or(cmd)
        .to_string();

    let timestamp = format!(
        "{}:{:02}",
        10 + state.messages.len() / 60,
        state.messages.len() % 60
    );

    match selected_cmd.as_str() {
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
                 │  Ctrl+A     Toggle mock listening               │\n\
                 │                                                 │\n\
                 │  SLASH COMMANDS                                 │\n\
                 │  /help /clear /model /system /export            │\n\
                 │  /settings /theme /tools /listen                │\n\
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
        "listen" => {
            toggle_listening(state);
        }
        _ => {
            state.status = format!("Unknown command: /{}", selected_cmd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spec_ai_tui::event::KeyEvent;

    fn make_test_state() -> DemoState {
        let mut state = DemoState::default();
        // Disable onboarding for most tests
        state.onboarding.active = false;
        state
    }

    fn make_key_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        })
    }

    #[test]
    fn test_quit_on_double_ctrl_c() {
        let state = make_test_state();
        // Ctrl+C is represented as Char('c') with CONTROL modifier
        let quit_event = Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });

        // First Ctrl+C - should set pending_quit
        let (actions, consumed) = handle_event_pure(quit_event.clone(), &state);
        assert!(consumed);
        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::SetPendingQuit(true))));
        assert!(actions.iter().any(|a| matches!(a, Action::SetStatus(_))));

        // Apply actions to get state for second Ctrl+C
        let mut state2 = state.clone();
        apply_actions(&mut state2, actions);

        // Second Ctrl+C - should quit
        let (actions2, consumed2) = handle_event_pure(quit_event, &state2);
        assert!(!consumed2); // Quit should not be consumed
        assert!(actions2.iter().any(|a| matches!(a, Action::Quit)));
    }

    #[test]
    fn test_reset_pending_quit_on_key() {
        let mut state = make_test_state();
        state.pending_quit = true;

        let key_event = make_key_event(KeyCode::Char('a'));
        let (actions, _) = handle_event_pure(key_event, &state);

        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::SetPendingQuit(false))));
        assert!(actions.iter().any(|a| matches!(a, Action::SetStatus(_))));
    }

    #[test]
    fn test_close_log_viewer() {
        let mut state = make_test_state();
        state.viewing_logs = Some(0);

        let close_event = make_key_event(KeyCode::Esc);
        let (actions, _) = handle_event_pure(close_event, &state);

        assert!(actions.iter().any(|a| matches!(a, Action::CloseLogViewer)));
    }

    #[test]
    fn test_scroll_logs() {
        let mut state = make_test_state();
        state.viewing_logs = Some(0);

        // Scroll up
        let scroll_up = make_key_event(KeyCode::Up);
        let (actions, _) = handle_event_pure(scroll_up, &state);
        assert!(actions.iter().any(|a| matches!(a, Action::ScrollLogs(1))));

        // Scroll down
        let scroll_down = make_key_event(KeyCode::Down);
        let (actions, _) = handle_event_pure(scroll_down, &state);
        assert!(actions.iter().any(|a| matches!(a, Action::ScrollLogs(-1))));
    }

    #[test]
    fn test_toggle_process_panel() {
        let state = make_test_state();
        assert!(!state.show_process_panel);

        // Test the action application
        let mut state2 = state.clone();
        let actions = vec![Action::ToggleProcessPanel];
        apply_actions(&mut state2, actions);

        assert!(state2.show_process_panel);
        assert!(!state2.show_history);
    }

    #[test]
    fn test_select_process() {
        let state = make_test_state();
        let mut state2 = state.clone();

        apply_actions(&mut state2, vec![Action::SelectProcess(5)]);
        assert_eq!(state2.selected_process, 5);
    }

    #[test]
    fn test_add_message() {
        let state = make_test_state();
        let mut state2 = state.clone();
        let initial_count = state2.messages.len();

        let msg = ChatMessage::new("user", "Hello", "10:00");
        apply_actions(&mut state2, vec![Action::AddMessage(msg)]);

        assert_eq!(state2.messages.len(), initial_count + 1);
    }

    #[test]
    fn test_clear_messages() {
        let state = make_test_state();
        let mut state2 = state.clone();
        state2
            .messages
            .push(ChatMessage::new("user", "Test", "10:00"));
        assert!(!state2.messages.is_empty());

        apply_actions(&mut state2, vec![Action::ClearMessages]);
        assert!(state2.messages.is_empty());
    }

    #[test]
    fn test_change_focus() {
        let state = make_test_state();
        let mut state2 = state.clone();
        assert_eq!(state2.focus, Panel::Input);

        apply_actions(&mut state2, vec![Action::ChangeFocus(Panel::Agent)]);
        assert_eq!(state2.focus, Panel::Agent);
        assert!(!state2.editor.focused);

        apply_actions(&mut state2, vec![Action::ChangeFocus(Panel::Input)]);
        assert_eq!(state2.focus, Panel::Input);
        assert!(state2.editor.focused);
    }

    #[test]
    fn test_start_and_progress_streaming() {
        let state = make_test_state();
        let mut state2 = state.clone();

        apply_actions(&mut state2, vec![Action::StartStreaming]);
        assert!(state2.streaming.is_some());
        assert_eq!(state2.stream_index, 0);

        apply_actions(
            &mut state2,
            vec![Action::ProgressStreaming("hello ".to_string())],
        );
        assert_eq!(state2.streaming.as_ref().unwrap(), "hello ");
        assert_eq!(state2.stream_index, 1);
    }

    #[test]
    fn test_toggle_voice() {
        let state = make_test_state();
        let mut state2 = state.clone();
        let initial = state2.onboarding.voice_enabled;

        apply_actions(&mut state2, vec![Action::ToggleVoice]);
        assert_eq!(state2.onboarding.voice_enabled, !initial);

        apply_actions(&mut state2, vec![Action::ToggleVoice]);
        assert_eq!(state2.onboarding.voice_enabled, initial);
    }

    #[test]
    fn test_cycle_policy() {
        use crate::state::PolicyMode;
        let state = make_test_state();
        let mut state2 = state.clone();
        state2.onboarding.policy_mode = PolicyMode::Standard;

        apply_actions(&mut state2, vec![Action::CyclePolicy]);
        assert_eq!(state2.onboarding.policy_mode, PolicyMode::Expanded);

        apply_actions(&mut state2, vec![Action::CyclePolicy]);
        assert_eq!(state2.onboarding.policy_mode, PolicyMode::Standard);
    }

    #[test]
    fn test_update_reasoning() {
        let state = make_test_state();
        let mut state2 = state.clone();

        apply_actions(
            &mut state2,
            vec![Action::UpdateIdleReasoning(
                "◇ Ready".to_string(),
                "Waiting".to_string(),
                "Type / for commands".to_string(),
            )],
        );

        assert_eq!(state2.reasoning[0], "◇ Ready");
        assert_eq!(state2.reasoning[1], "Waiting");
        assert_eq!(state2.reasoning[2], "Type / for commands");
    }
}
