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
        if let KeyCode::Char('l') = key.code {
            state.messages.clear();
            state.status = "Chat cleared".to_string();
            state.scroll_offset = 0;
            return;
        }
    }

    let was_showing = state.editor.show_slash_menu;
    match state.editor.handle_event(event) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> AppState {
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        AppState::new(rx)
    }

    fn create_backend_channel() -> UnboundedSender<BackendRequest> {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        tx
    }

    #[test]
    fn on_tick_increments_tick_counter() {
        let mut state = create_test_state();
        assert_eq!(state.tick, 0);
        on_tick(&mut state);
        assert_eq!(state.tick, 1);
        on_tick(&mut state);
        assert_eq!(state.tick, 2);
    }

    #[test]
    fn on_tick_saturates_at_max() {
        let mut state = create_test_state();
        state.tick = u64::MAX;
        on_tick(&mut state);
        assert_eq!(state.tick, u64::MAX);
    }

    #[test]
    fn filtered_command_count_returns_all_when_empty_query() {
        let state = create_test_state();
        let total_commands = state.slash_commands.len();
        let count = filtered_command_count(&state);
        assert_eq!(count, total_commands);
    }

    #[test]
    fn filtered_command_count_filters_by_query() {
        let mut state = create_test_state();
        state.editor.slash_query = "help".to_string();
        let count = filtered_command_count(&state);
        assert_eq!(count, 1);
    }

    #[test]
    fn filtered_command_count_returns_zero_for_no_match() {
        let mut state = create_test_state();
        state.editor.slash_query = "zzzznonexistent".to_string();
        let count = filtered_command_count(&state);
        assert_eq!(count, 0);
    }

    #[test]
    fn selected_slash_command_returns_first_when_index_zero() {
        let mut state = create_test_state();
        state.slash_menu.show();
        let cmd = selected_slash_command(&state);
        assert!(cmd.is_some());
        // First command should be "help" based on default_slash_commands()
        assert_eq!(cmd.unwrap().name, "help");
    }

    #[test]
    fn selected_slash_command_returns_none_for_empty_filter() {
        let mut state = create_test_state();
        state.editor.slash_query = "zzzznonexistent".to_string();
        let cmd = selected_slash_command(&state);
        assert!(cmd.is_none());
    }

    #[test]
    fn selected_slash_command_respects_filter() {
        let mut state = create_test_state();
        state.editor.slash_query = "conf".to_string();
        let cmd = selected_slash_command(&state);
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name, "config");
    }

    #[test]
    fn complete_slash_command_returns_false_when_no_match() {
        let mut state = create_test_state();
        state.editor.slash_query = "zzzznonexistent".to_string();
        let result = complete_slash_command(&mut state);
        assert!(!result);
    }

    #[test]
    fn complete_slash_command_sets_editor_text() {
        let mut state = create_test_state();
        state.editor.slash_query = "help".to_string();
        let result = complete_slash_command(&mut state);
        assert!(result);
        assert_eq!(state.editor.text, "/help");
    }

    #[test]
    fn complete_slash_command_hides_menu() {
        let mut state = create_test_state();
        state.editor.show_slash_menu = true;
        state.slash_menu.show();
        state.editor.slash_query = "help".to_string();
        complete_slash_command(&mut state);
        assert!(!state.editor.show_slash_menu);
    }

    #[test]
    fn complete_slash_command_clears_query() {
        let mut state = create_test_state();
        state.editor.slash_query = "help".to_string();
        complete_slash_command(&mut state);
        assert!(state.editor.slash_query.is_empty());
    }

    #[test]
    fn complete_slash_command_updates_status() {
        let mut state = create_test_state();
        state.editor.slash_query = "help".to_string();
        complete_slash_command(&mut state);
        assert!(state.status.contains("Prepared /help"));
    }

    #[test]
    fn sync_slash_menu_visibility_shows_menu() {
        let mut state = create_test_state();
        state.editor.show_slash_menu = true;
        sync_slash_menu_visibility(&mut state, false);
        assert!(state.slash_menu.visible);
    }

    #[test]
    fn sync_slash_menu_visibility_hides_menu() {
        let mut state = create_test_state();
        state.slash_menu.show();
        state.editor.show_slash_menu = false;
        sync_slash_menu_visibility(&mut state, true);
        assert!(!state.slash_menu.visible);
    }

    #[test]
    fn sync_slash_menu_visibility_no_change_when_same() {
        let mut state = create_test_state();
        state.editor.show_slash_menu = true;
        sync_slash_menu_visibility(&mut state, true);
        // State should remain unchanged (neither show() nor hide() called)
    }

    #[test]
    fn handle_chat_key_down_decrements_scroll() {
        let mut state = create_test_state();
        state.focus = PanelFocus::Chat;
        state.scroll_offset = 5;
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        handle_chat_key(&key, &mut state);
        assert_eq!(state.scroll_offset, 4);
    }

    #[test]
    fn handle_chat_key_down_switches_to_input_when_at_bottom() {
        let mut state = create_test_state();
        state.focus = PanelFocus::Chat;
        state.scroll_offset = 0;
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        handle_chat_key(&key, &mut state);
        assert_eq!(state.focus, PanelFocus::Input);
        assert!(state.editor.focused);
    }

    #[test]
    fn handle_chat_key_up_increments_scroll() {
        let mut state = create_test_state();
        state.focus = PanelFocus::Chat;
        state.scroll_offset = 5;
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        handle_chat_key(&key, &mut state);
        assert_eq!(state.scroll_offset, 6);
    }

    #[test]
    fn handle_chat_key_j_acts_like_down() {
        let mut state = create_test_state();
        state.focus = PanelFocus::Chat;
        state.scroll_offset = 5;
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        handle_chat_key(&key, &mut state);
        assert_eq!(state.scroll_offset, 4);
    }

    #[test]
    fn handle_chat_key_k_acts_like_up() {
        let mut state = create_test_state();
        state.focus = PanelFocus::Chat;
        state.scroll_offset = 5;
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        handle_chat_key(&key, &mut state);
        assert_eq!(state.scroll_offset, 6);
    }

    #[test]
    fn handle_chat_key_page_up_scrolls_by_8() {
        let mut state = create_test_state();
        state.focus = PanelFocus::Chat;
        state.scroll_offset = 5;
        let key = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);
        handle_chat_key(&key, &mut state);
        assert_eq!(state.scroll_offset, 13);
    }

    #[test]
    fn handle_chat_key_page_down_scrolls_by_8() {
        let mut state = create_test_state();
        state.focus = PanelFocus::Chat;
        state.scroll_offset = 10;
        let key = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
        handle_chat_key(&key, &mut state);
        assert_eq!(state.scroll_offset, 2);
    }

    #[test]
    fn handle_chat_key_tab_switches_to_input() {
        let mut state = create_test_state();
        state.focus = PanelFocus::Chat;
        state.editor.focused = false;
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        handle_chat_key(&key, &mut state);
        assert_eq!(state.focus, PanelFocus::Input);
        assert!(state.editor.focused);
    }

    #[test]
    fn handle_chat_key_scroll_saturates_at_zero() {
        let mut state = create_test_state();
        state.focus = PanelFocus::Chat;
        state.scroll_offset = 2;
        let key = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
        handle_chat_key(&key, &mut state);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn handle_event_returns_false_on_quit() {
        let mut state = create_test_state();
        state.quit = true;
        let backend_tx = create_backend_channel();
        let result = handle_event(Event::Tick, &mut state, &backend_tx);
        assert!(!result);
    }

    #[test]
    fn handle_event_tick_increments_counter() {
        let mut state = create_test_state();
        let backend_tx = create_backend_channel();
        assert_eq!(state.tick, 0);
        handle_event(Event::Tick, &mut state, &backend_tx);
        assert_eq!(state.tick, 1);
    }
}
