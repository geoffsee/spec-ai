//! Ring-style input handlers
//!
//! Controls simulate a wearable ring:
//! - Scroll/Up/Down: Navigate within focused panel
//! - Enter/Select: Activate current selection
//! - Tab: Toggle focus between menu and events
//! - Esc: Back to default view

use crate::state::DemoState;
use crossterm::event::{KeyCode, KeyModifiers};
use spec_ai_oui::{
    input::{GestureType, SwipeDirection},
    OpticalEvent,
};

pub fn handle_event(event: OpticalEvent, state: &mut DemoState) -> bool {
    match event {
        OpticalEvent::Key(key) => handle_key(key, state),
        OpticalEvent::Gesture(g) => match g.gesture {
            GestureType::AirTap { .. } => {
                state.select();
                true
            }
            GestureType::Swipe { direction, .. } => {
                match direction {
                    SwipeDirection::Up => state.scroll_up(),
                    SwipeDirection::Down => state.scroll_down(),
                    SwipeDirection::Left => state.back(),
                    SwipeDirection::Right => state.toggle_focus(),
                }
                true
            }
            GestureType::Pinch { strength } if strength > 0.8 => {
                state.back();
                true
            }
            _ => true,
        },
        OpticalEvent::Voice { command, .. } => handle_voice(&command, state),
        _ => true,
    }
}

fn handle_key(key: crossterm::event::KeyEvent, state: &mut DemoState) -> bool {
    // Quit on Ctrl+Q
    if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }

    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Char('Q') => return false,

        // Navigation - scroll within current panel
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            state.scroll_up();
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            state.scroll_down();
        }

        // Toggle focus between panels
        KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
            state.toggle_focus();
        }

        // Select
        KeyCode::Enter | KeyCode::Char(' ') => {
            state.select();
        }

        // Back
        KeyCode::Esc | KeyCode::Backspace => {
            state.back();
        }

        _ => {}
    }
    true
}

fn handle_voice(cmd: &str, state: &mut DemoState) -> bool {
    let c = cmd.to_lowercase();
    if c.contains("mode") {
        state.menu_index = 0;
        state.select();
    } else if c.contains("alert") {
        state.menu_index = 1;
        state.select();
    } else if c.contains("setting") {
        state.menu_index = 2;
        state.select();
    } else if c.contains("back") || c.contains("home") {
        state.back();
    } else if c.contains("up") || c.contains("previous") {
        state.scroll_up();
    } else if c.contains("down") || c.contains("next") {
        state.scroll_down();
    } else if c.contains("select") || c.contains("enter") {
        state.select();
    }
    true
}
