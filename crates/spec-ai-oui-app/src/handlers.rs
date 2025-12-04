//! Event handlers for the OUI demo - practical assistant

use crossterm::event::{KeyCode, KeyModifiers};

use spec_ai_oui::{
    OpticalEvent,
    input::{GestureType, SwipeDirection},
};

use crate::state::{DemoState, PanelFocus};

/// Handle an optical event
pub fn handle_event(event: OpticalEvent, state: &mut DemoState) -> bool {
    match event {
        OpticalEvent::Key(key) => {
            handle_key_event(key, state)
        }

        OpticalEvent::GazeMove { screen_pos, .. } => {
            state.gaze_pos = screen_pos;
            true
        }

        OpticalEvent::GazeDwell { target_id, .. } => {
            // Dwell on a POI to select it
            if target_id.starts_with("nav-") || target_id.starts_with("person-") || target_id.starts_with("place-") {
                state.select_poi(&target_id);
            }
            true
        }

        OpticalEvent::Gesture(gesture) => {
            match gesture.gesture {
                GestureType::AirTap { .. } => {
                    handle_select(state)
                }
                GestureType::Swipe { direction, .. } => {
                    handle_swipe(direction, state)
                }
                GestureType::Pinch { strength } if strength > 0.8 => {
                    handle_select(state)
                }
                GestureType::OpenPalm => {
                    // Close menu or dismiss notification
                    if state.show_menu {
                        state.show_menu = false;
                    }
                    true
                }
                GestureType::Fist => {
                    // Toggle menu
                    state.show_menu = !state.show_menu;
                    if state.show_menu {
                        state.menu_selection = Some(0);
                    }
                    true
                }
                _ => true,
            }
        }

        OpticalEvent::Voice { command, .. } => {
            handle_voice_command(&command, state)
        }

        OpticalEvent::HeadGesture(gesture) => {
            use spec_ai_oui::input::HeadGestureType;
            match gesture {
                HeadGestureType::Nod => {
                    // Confirm action
                    handle_select(state)
                }
                HeadGestureType::Shake => {
                    // Cancel/dismiss
                    if state.show_menu {
                        state.show_menu = false;
                    } else if state.show_calendar {
                        state.show_calendar = false;
                    }
                    true
                }
                _ => true,
            }
        }

        OpticalEvent::HeadPose { transform } => {
            // Update heading based on head rotation
            let forward = transform.forward();
            state.heading = forward.x.atan2(forward.z).to_degrees();
            if state.heading < 0.0 {
                state.heading += 360.0;
            }
            true
        }

        OpticalEvent::Tick => true,

        OpticalEvent::Resize { .. } => true,

        _ => true,
    }
}

/// Handle keyboard events
fn handle_key_event(key: crossterm::event::KeyEvent, state: &mut DemoState) -> bool {
    // Check for quit
    if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }
    if key.code == KeyCode::Esc {
        if state.show_menu {
            state.show_menu = false;
            return true;
        }
        if state.show_calendar {
            state.show_calendar = false;
            return true;
        }
        return false;
    }

    match key.code {
        // Tab: cycle focus
        KeyCode::Tab => {
            state.focus = match state.focus {
                PanelFocus::None => PanelFocus::Calendar,
                PanelFocus::Calendar => PanelFocus::Notifications,
                PanelFocus::Notifications => PanelFocus::Navigation,
                PanelFocus::Navigation => PanelFocus::None,
                PanelFocus::Menu => PanelFocus::None,
            };
        }

        // Enter: select/confirm
        KeyCode::Enter => {
            return handle_select(state);
        }

        // M: toggle menu
        KeyCode::Char('m') | KeyCode::Char('M') => {
            state.show_menu = !state.show_menu;
            if state.show_menu {
                state.focus = PanelFocus::Menu;
                state.menu_selection = Some(0);
            } else {
                state.focus = PanelFocus::None;
            }
        }

        // C: toggle calendar
        KeyCode::Char('c') | KeyCode::Char('C') => {
            state.toggle_calendar();
        }

        // N: mark notification read / cycle notifications
        KeyCode::Char('n') | KeyCode::Char('N') => {
            if let Some(notif) = state.notifications.iter().find(|n| !n.read) {
                let id = notif.id.clone();
                state.mark_read(&id);
            }
        }

        // 1-4: quick actions
        KeyCode::Char('1') => state.trigger_action(0),
        KeyCode::Char('2') => state.trigger_action(1),
        KeyCode::Char('3') => state.trigger_action(2),
        KeyCode::Char('4') => state.trigger_action(3),

        // P: select next POI
        KeyCode::Char('p') | KeyCode::Char('P') => {
            let current = state.points_of_interest.iter().position(|p| p.selected);
            let next = match current {
                Some(i) => (i + 1) % state.points_of_interest.len(),
                None => 0,
            };
            if let Some(poi) = state.points_of_interest.get(next) {
                let id = poi.id.clone();
                state.select_poi(&id);
            }
        }

        // +/-: adjust density
        KeyCode::Char('+') | KeyCode::Char('=') => {
            state.density = match state.density {
                spec_ai_oui::InformationDensity::Minimal => spec_ai_oui::InformationDensity::Low,
                spec_ai_oui::InformationDensity::Low => spec_ai_oui::InformationDensity::Normal,
                spec_ai_oui::InformationDensity::Normal => spec_ai_oui::InformationDensity::High,
                spec_ai_oui::InformationDensity::High => spec_ai_oui::InformationDensity::Maximum,
                spec_ai_oui::InformationDensity::Maximum => spec_ai_oui::InformationDensity::Maximum,
            };
        }
        KeyCode::Char('-') => {
            state.density = match state.density {
                spec_ai_oui::InformationDensity::Minimal => spec_ai_oui::InformationDensity::Minimal,
                spec_ai_oui::InformationDensity::Low => spec_ai_oui::InformationDensity::Minimal,
                spec_ai_oui::InformationDensity::Normal => spec_ai_oui::InformationDensity::Low,
                spec_ai_oui::InformationDensity::High => spec_ai_oui::InformationDensity::Normal,
                spec_ai_oui::InformationDensity::Maximum => spec_ai_oui::InformationDensity::High,
            };
        }

        // F: toggle focus mode
        KeyCode::Char('f') | KeyCode::Char('F') => {
            state.mode = if state.mode == spec_ai_oui::DisplayMode::Focus {
                spec_ai_oui::DisplayMode::Navigate
            } else {
                spec_ai_oui::DisplayMode::Focus
            };
        }

        _ => {}
    }

    true
}

/// Handle select/confirm action
fn handle_select(state: &mut DemoState) -> bool {
    // Calendar overlay - dismiss
    if state.show_calendar {
        state.show_calendar = false;
        return true;
    }

    // Menu selection
    if state.show_menu {
        if let Some(selection) = state.menu_selection {
            match selection {
                0 => {
                    state.status_message = Some("Calendar opened".to_string());
                    state.show_calendar = true;
                }
                1 => {
                    state.status_message = Some("Messages opened".to_string());
                }
                2 => {
                    state.status_message = Some("Navigation started".to_string());
                }
                3 => {
                    state.status_message = Some("Settings opened".to_string());
                }
                _ => {}
            }
            state.show_menu = false;
        }
        return true;
    }

    // If a POI is selected, navigate to it
    if let Some(poi) = state.points_of_interest.iter().find(|p| p.selected) {
        state.status_message = Some(format!("Navigating to {}", poi.name));
    }

    true
}

/// Handle swipe gestures
fn handle_swipe(direction: SwipeDirection, state: &mut DemoState) -> bool {
    if state.show_menu {
        // Navigate menu
        let menu_items = 4;
        match direction {
            SwipeDirection::Up => {
                state.menu_selection = Some(
                    state.menu_selection.map(|s| if s == 0 { menu_items - 1 } else { s - 1 }).unwrap_or(0)
                );
            }
            SwipeDirection::Down => {
                state.menu_selection = Some(
                    state.menu_selection.map(|s| (s + 1) % menu_items).unwrap_or(0)
                );
            }
            SwipeDirection::Left => {
                state.show_menu = false;
            }
            SwipeDirection::Right => {
                return handle_select(state);
            }
        }
    } else {
        // Navigation / look around
        match direction {
            SwipeDirection::Left => {
                state.heading = (state.heading - 15.0 + 360.0) % 360.0;
            }
            SwipeDirection::Right => {
                state.heading = (state.heading + 15.0) % 360.0;
            }
            SwipeDirection::Up => {
                // Dismiss notification
                if let Some(notif) = state.notifications.iter().find(|n| !n.read) {
                    let id = notif.id.clone();
                    state.mark_read(&id);
                }
            }
            SwipeDirection::Down => {
                // Show quick glance
                state.status_message = Some(format!("{} unread", state.unread_count()));
            }
        }
    }

    true
}

/// Handle voice commands
fn handle_voice_command(command: &str, state: &mut DemoState) -> bool {
    let cmd = command.to_lowercase();

    if cmd.contains("select") || cmd.contains("confirm") || cmd.contains("ok") {
        return handle_select(state);
    }
    if cmd.contains("menu") {
        state.show_menu = !state.show_menu;
        return true;
    }
    if cmd.contains("back") || cmd.contains("cancel") || cmd.contains("dismiss") {
        if state.show_menu {
            state.show_menu = false;
        } else if state.show_calendar {
            state.show_calendar = false;
        }
        return true;
    }
    if cmd.contains("calendar") || cmd.contains("schedule") {
        state.toggle_calendar();
        return true;
    }
    if cmd.contains("navigate") || cmd.contains("directions") {
        state.status_message = Some("Navigation mode".to_string());
        return true;
    }
    if cmd.contains("call") {
        state.trigger_action(0);
        return true;
    }
    if cmd.contains("message") || cmd.contains("text") {
        state.trigger_action(1);
        return true;
    }
    if cmd.contains("focus") || cmd.contains("do not disturb") {
        state.mode = spec_ai_oui::DisplayMode::Focus;
        state.status_message = Some("Focus mode enabled".to_string());
        return true;
    }
    if cmd.contains("read") && cmd.contains("notification") {
        if let Some(notif) = state.notifications.iter().find(|n| !n.read) {
            state.status_message = Some(format!("{}: {}", notif.title, notif.preview));
        }
        return true;
    }

    state.status_message = Some(format!("\"{}\"", command));
    true
}
