//! Event handlers for the OUI demo

use crossterm::event::{KeyCode, KeyModifiers};

use spec_ai_oui::{
    OpticalEvent,
    input::{GestureType, SwipeDirection},
};

use crate::state::{DemoState, PanelFocus, MissionStatus};

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
            // Dwell on a target to start locking
            if target_id.starts_with("target-") || target_id.starts_with("hostile-") {
                state.lock_target(&target_id);
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
                    // Close menu
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
                    // Cancel/back
                    if state.show_menu {
                        state.show_menu = false;
                    } else if state.show_briefing {
                        // Do nothing - can't cancel briefing with shake
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
        if state.show_briefing && state.mission.status == MissionStatus::Briefing {
            return true; // Can't dismiss briefing with Esc during briefing
        }
        return false;
    }

    match key.code {
        // Tab: cycle focus
        KeyCode::Tab => {
            state.focus = match state.focus {
                PanelFocus::None => PanelFocus::Mission,
                PanelFocus::Mission => PanelFocus::Status,
                PanelFocus::Status => PanelFocus::Targets,
                PanelFocus::Targets => PanelFocus::None,
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

        // B: toggle briefing
        KeyCode::Char('b') | KeyCode::Char('B') => {
            if state.mission.status != MissionStatus::Briefing {
                state.show_briefing = !state.show_briefing;
            }
        }

        // 1-3: gadgets
        KeyCode::Char('1') => state.use_gadget(0),
        KeyCode::Char('2') => state.use_gadget(1),
        KeyCode::Char('3') => state.use_gadget(2),

        // T: lock next target
        KeyCode::Char('t') | KeyCode::Char('T') => {
            let current_locked = state.mission.targets.iter().position(|t| t.locked);
            let next = match current_locked {
                Some(i) => (i + 1) % state.mission.targets.len(),
                None => 0,
            };
            if let Some(target) = state.mission.targets.get(next) {
                let id = target.id.clone();
                state.lock_target(&id);
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

        _ => {}
    }

    true
}

/// Handle select/confirm action
fn handle_select(state: &mut DemoState) -> bool {
    // In briefing mode, start mission
    if state.show_briefing && state.mission.status == MissionStatus::Briefing {
        state.start_mission();
        return true;
    }

    // In menu mode, select item
    if state.show_menu {
        if let Some(selection) = state.menu_selection {
            match selection {
                0 => {
                    state.status_message = Some("Map view".to_string());
                }
                1 => {
                    state.status_message = Some("Comms".to_string());
                }
                2 => {
                    state.status_message = Some("Inventory".to_string());
                }
                3 => {
                    state.status_message = Some("Settings".to_string());
                }
                _ => {}
            }
            state.show_menu = false;
        }
        return true;
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
        // Navigation
        match direction {
            SwipeDirection::Left => {
                state.heading = (state.heading - 15.0 + 360.0) % 360.0;
            }
            SwipeDirection::Right => {
                state.heading = (state.heading + 15.0) % 360.0;
            }
            _ => {}
        }
    }

    true
}

/// Handle voice commands
fn handle_voice_command(command: &str, state: &mut DemoState) -> bool {
    let cmd = command.to_lowercase();

    if cmd.contains("select") || cmd.contains("confirm") {
        return handle_select(state);
    }
    if cmd.contains("menu") {
        state.show_menu = !state.show_menu;
        return true;
    }
    if cmd.contains("back") || cmd.contains("cancel") {
        if state.show_menu {
            state.show_menu = false;
        }
        return true;
    }
    if cmd.contains("target") {
        // Lock next target
        let current = state.mission.targets.iter().position(|t| t.locked);
        let next = match current {
            Some(i) => (i + 1) % state.mission.targets.len(),
            None => 0,
        };
        if let Some(target) = state.mission.targets.get(next) {
            let id = target.id.clone();
            state.lock_target(&id);
        }
        return true;
    }
    if cmd.contains("gadget") || cmd.contains("emp") {
        state.use_gadget(0);
        return true;
    }

    state.status_message = Some(format!("Command: {}", command));
    true
}
