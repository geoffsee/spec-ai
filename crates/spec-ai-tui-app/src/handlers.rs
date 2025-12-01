use crate::backend::BackendRequest;
use crate::models::ChatMessage;
use crate::state::{AppState, PanelFocus};
use spec_ai_tui::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use spec_ai_tui::widget::builtin::{EditorAction, Selection, SlashCommand};
use tokio::sync::mpsc::UnboundedSender;

pub fn handle_event(
    event: Event,
    state: &mut AppState,
    backend_tx: &UnboundedSender<BackendRequest>,
) -> bool {
    state.drain_backend_events();
    if state.quit {
        return false;
    }

    match &event {
        Event::Key(key) => {
            if event.is_quit() {
                state.quit = true;
                return false;
            }

            match state.focus {
                PanelFocus::Input => handle_input_key(&event, key, state, backend_tx),
                PanelFocus::Chat => handle_chat_key(key, state),
            }
        }
        Event::Paste(_) => {
            if state.focus == PanelFocus::Input {
                let was_showing = state.editor.show_slash_menu;
                if let EditorAction::Handled = state.editor.handle_event(&event) {
                    sync_slash_menu_visibility(state, was_showing);
                }
            }
        }
        Event::Tick => {
            on_tick(state);
        }
        Event::Resize { .. } => {
            state.drain_backend_events();
        }
        _ => {}
    }

    !state.quit
}

pub fn on_tick(state: &mut AppState) {
    state.tick = state.tick.saturating_add(1);
    state.drain_backend_events();
}

fn handle_chat_key(key: &KeyEvent, state: &mut AppState) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if state.scroll_offset > 0 {
                state.scroll_offset = state.scroll_offset.saturating_sub(1);
            } else {
                state.focus = PanelFocus::Input;
                state.editor.focused = true;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.scroll_offset = state.scroll_offset.saturating_add(1);
        }
        KeyCode::PageUp => {
            state.scroll_offset = state.scroll_offset.saturating_add(8);
        }
        KeyCode::PageDown => {
            state.scroll_offset = state.scroll_offset.saturating_sub(8);
        }
        KeyCode::Tab => {
            state.focus = PanelFocus::Input;
            state.editor.focused = true;
        }
        _ => {}
    }
}

fn handle_input_key(
    event: &Event,
    key: &KeyEvent,
    state: &mut AppState,
    backend_tx: &UnboundedSender<BackendRequest>,
) {
    // Global shortcuts while focused on input
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('l') => {
                state.messages.clear();
                state.status = "Chat cleared".to_string();
                state.scroll_offset = 0;
                return;
            }
            _ => {}
        }
    }

    let was_showing = state.editor.show_slash_menu;
    match state.editor.handle_event(&event) {
        EditorAction::Handled => {
            sync_slash_menu_visibility(state, was_showing);
        }
        EditorAction::Submit(text) => {
            submit_text(state, backend_tx, text);
        }
        EditorAction::SlashCommand(cmd) => {
            submit_text(state, backend_tx, format!("/{}", cmd));
        }
        EditorAction::SlashMenuNext => {
            if complete_slash_command(state) {
                return;
            }
            let count = filtered_command_count(state);
            state.slash_menu.next(count);
        }
        EditorAction::SlashMenuPrev => {
            let count = filtered_command_count(state);
            state.slash_menu.prev(count);
        }
        EditorAction::Escape => {
            state.editor.show_slash_menu = false;
            state.editor.slash_query.clear();
            state.slash_menu.hide();
        }
        EditorAction::Ignored => match key.code {
            KeyCode::Up if !state.editor.show_slash_menu => {
                state.focus = PanelFocus::Chat;
                state.editor.focused = false;
            }
            KeyCode::Up if state.editor.show_slash_menu => {
                let count = filtered_command_count(state);
                state.slash_menu.prev(count);
            }
            KeyCode::Down if state.editor.show_slash_menu => {
                let count = filtered_command_count(state);
                state.slash_menu.next(count);
            }
            KeyCode::PageUp => {
                state.scroll_offset = state.scroll_offset.saturating_add(5);
            }
            KeyCode::PageDown => {
                state.scroll_offset = state.scroll_offset.saturating_sub(5);
            }
            KeyCode::Tab => {
                state.focus = PanelFocus::Chat;
                state.editor.focused = false;
            }
            _ => {}
        },
    }
}

fn submit_text(state: &mut AppState, backend_tx: &UnboundedSender<BackendRequest>, text: String) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }

    state.messages.push(ChatMessage::user(trimmed));
    state.scroll_offset = 0;
    state.busy = true;
    state.status = "Running command...".to_string();
    state.last_submitted_text = Some(trimmed.to_string());

    state.editor.clear();
    state.editor.show_slash_menu = false;
    state.editor.slash_query.clear();
    state.slash_menu.hide();

    if backend_tx
        .send(BackendRequest::Submit(trimmed.to_string()))
        .is_err()
    {
        state.busy = false;
        state.status = "Backend unavailable".to_string();
        state.error = Some("Backend channel closed".to_string());
    }
}

fn sync_slash_menu_visibility(state: &mut AppState, was_showing: bool) {
    if state.editor.show_slash_menu && !was_showing {
        state.slash_menu.show();
    } else if !state.editor.show_slash_menu && was_showing {
        state.slash_menu.hide();
    }
}

fn filtered_command_count(state: &AppState) -> usize {
    state
        .slash_commands
        .iter()
        .filter(|cmd| cmd.matches(&state.editor.slash_query))
        .count()
}

pub fn selected_slash_command(state: &AppState) -> Option<SlashCommand> {
    let filtered: Vec<_> = state
        .slash_commands
        .iter()
        .filter(|c| c.matches(&state.editor.slash_query))
        .cloned()
        .collect();

    filtered.get(state.slash_menu.selected_index()).cloned()
}

pub fn complete_slash_command(state: &mut AppState) -> bool {
    if let Some(cmd) = selected_slash_command(state) {
        let text = format!("/{}", cmd.name);
        state.editor.text = text.clone();
        state.editor.selection = Selection::cursor(text.len());
        state.editor.show_slash_menu = false;
        state.editor.slash_query.clear();
        state.slash_menu.hide();
        state.status = format!("Prepared /{} (Enter to run, add args manually)", cmd.name);
        true
    } else {
        false
    }
}
