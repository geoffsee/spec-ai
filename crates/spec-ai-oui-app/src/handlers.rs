//! Super OUI Event Handlers

use crossterm::event::{KeyCode, KeyModifiers};
use spec_ai_oui::{OpticalEvent, input::{GestureType, SwipeDirection}};
use crate::state::{DemoState, PanelFocus};

pub fn handle_event(event: OpticalEvent, state: &mut DemoState) -> bool {
    match event {
        OpticalEvent::Key(key) => handle_key(key, state),
        OpticalEvent::GazeMove { screen_pos, .. } => { state.gaze_pos = screen_pos; true }
        OpticalEvent::GazeDwell { target_id, .. } => { if target_id.starts_with("person-") { state.select_person(&target_id); } true }
        OpticalEvent::Gesture(g) => match g.gesture {
            GestureType::AirTap { .. } => handle_select(state),
            GestureType::Swipe { direction, .. } => handle_swipe(direction, state),
            GestureType::Pinch { strength } if strength > 0.8 => { if let Some(id) = state.first_active_notification_id() { state.dismiss_notification(&id); state.status_message = Some("Dismissed".to_string()); } true }
            GestureType::Fist => { state.toggle_private(); true }
            GestureType::OpenPalm => { state.show_menu = false; state.show_research = false; state.show_calendar = false; true }
            _ => true,
        },
        OpticalEvent::Voice { command, .. } => handle_voice(&command, state),
        OpticalEvent::HeadGesture(g) => {
            use spec_ai_oui::input::HeadGestureType;
            match g {
                HeadGestureType::Nod => { if let Some(id) = state.first_active_notification_id() { state.dismiss_notification(&id); } true }
                HeadGestureType::Shake => { state.show_menu = false; state.show_research = false; state.show_calendar = false; true }
                _ => true,
            }
        }
        OpticalEvent::HeadPose { transform } => { let f = transform.forward(); state.heading = f.x.atan2(f.z).to_degrees(); if state.heading < 0.0 { state.heading += 360.0; } true }
        _ => true,
    }
}

fn handle_key(key: crossterm::event::KeyEvent, state: &mut DemoState) -> bool {
    if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) { return false; }
    if key.code == KeyCode::Esc {
        if state.show_menu || state.show_research || state.show_calendar { state.show_menu = false; state.show_research = false; state.show_calendar = false; return true; }
        if state.recording.active { state.toggle_recording(); return true; }
        return false;
    }
    match key.code {
        KeyCode::Tab => { state.focus = match state.focus { PanelFocus::None => PanelFocus::Person, PanelFocus::Person => PanelFocus::Cues, PanelFocus::Cues => PanelFocus::Facts, PanelFocus::Facts => PanelFocus::Calendar, PanelFocus::Calendar => PanelFocus::Notifications, _ => PanelFocus::None }; }
        KeyCode::Enter => { return handle_select(state); }
        KeyCode::Char('s') | KeyCode::Char('S') => { let cur = state.people.iter().position(|p| p.selected); let next = cur.map(|i| (i + 1) % state.people.len()).unwrap_or(0); if let Some(p) = state.people.get(next) { let id = p.id.clone(); state.select_person(&id); } }
        KeyCode::Char('d') | KeyCode::Char('D') => { if let Some(id) = state.first_active_notification_id() { state.dismiss_notification(&id); state.status_message = Some("Dismissed".to_string()); } }
        KeyCode::Char('r') | KeyCode::Char('R') => { state.toggle_recording(); }
        KeyCode::Char('f') | KeyCode::Char('F') => { state.status_message = Some("Photo captured".to_string()); }
        KeyCode::Char('n') | KeyCode::Char('N') => { state.status_message = Some("Note saved".to_string()); }
        KeyCode::Char('v') | KeyCode::Char('V') => { state.status_message = Some("Fact-check queued".to_string()); }
        KeyCode::Char('p') | KeyCode::Char('P') => { state.toggle_private(); }
        KeyCode::Char('c') | KeyCode::Char('C') => { state.show_calendar = !state.show_calendar; }
        KeyCode::Char('i') | KeyCode::Char('I') => { state.show_research = !state.show_research; }
        KeyCode::Char('m') | KeyCode::Char('M') => { state.show_menu = !state.show_menu; if state.show_menu { state.menu_selection = Some(0); } }
        KeyCode::Char(' ') => { state.cycle_mode(); state.status_message = Some(format!("{} mode", state.mode.name())); }
        KeyCode::Char('+') | KeyCode::Char('=') => { state.density = match state.density { spec_ai_oui::InformationDensity::Minimal => spec_ai_oui::InformationDensity::Low, spec_ai_oui::InformationDensity::Low => spec_ai_oui::InformationDensity::Normal, spec_ai_oui::InformationDensity::Normal => spec_ai_oui::InformationDensity::High, _ => spec_ai_oui::InformationDensity::Maximum }; }
        KeyCode::Char('-') => { state.density = match state.density { spec_ai_oui::InformationDensity::Maximum => spec_ai_oui::InformationDensity::High, spec_ai_oui::InformationDensity::High => spec_ai_oui::InformationDensity::Normal, spec_ai_oui::InformationDensity::Normal => spec_ai_oui::InformationDensity::Low, _ => spec_ai_oui::InformationDensity::Minimal }; }
        KeyCode::Up if state.show_menu => { state.menu_selection = Some(state.menu_selection.map(|s| if s == 0 { 5 } else { s - 1 }).unwrap_or(0)); }
        KeyCode::Down if state.show_menu => { state.menu_selection = Some(state.menu_selection.map(|s| (s + 1) % 6).unwrap_or(0)); }
        _ => {}
    }
    true
}

fn handle_select(state: &mut DemoState) -> bool {
    if state.show_calendar { state.show_calendar = false; return true; }
    if state.show_research { state.show_research = false; return true; }
    if state.show_menu {
        if let Some(sel) = state.menu_selection {
            match sel { 0 => state.show_research = true, 1 => state.show_calendar = true, 2 => state.toggle_recording(), 3 => state.toggle_private(), 4 => state.status_message = Some("People nearby".to_string()), 5 => state.status_message = Some("Settings".to_string()), _ => {} }
            state.show_menu = false;
        }
        return true;
    }
    if let Some(p) = state.selected_person() { state.status_message = Some(format!("{}: {}", p.name, p.emotional_state.label())); }
    true
}

fn handle_swipe(dir: SwipeDirection, state: &mut DemoState) -> bool {
    if state.show_menu { match dir { SwipeDirection::Up => { state.menu_selection = Some(state.menu_selection.map(|s| if s == 0 { 5 } else { s - 1 }).unwrap_or(0)); } SwipeDirection::Down => { state.menu_selection = Some(state.menu_selection.map(|s| (s + 1) % 6).unwrap_or(0)); } SwipeDirection::Left => state.show_menu = false, SwipeDirection::Right => return handle_select(state), } }
    else { match dir {
        SwipeDirection::Left | SwipeDirection::Right => { let cur = state.people.iter().position(|p| p.selected); let next = match (cur, dir) { (Some(i), SwipeDirection::Right) => (i + 1) % state.people.len(), (Some(i), SwipeDirection::Left) => if i == 0 { state.people.len() - 1 } else { i - 1 }, _ => 0 }; if let Some(p) = state.people.get(next) { let id = p.id.clone(); state.select_person(&id); } }
        SwipeDirection::Up => { if let Some(id) = state.first_active_notification_id() { state.dismiss_notification(&id); } }
        SwipeDirection::Down => { state.status_message = Some(format!("{} notifications", state.active_notification_count())); }
    } }
    true
}

fn handle_voice(cmd: &str, state: &mut DemoState) -> bool {
    let c = cmd.to_lowercase();
    if c.contains("record") { state.toggle_recording(); return true; }
    if c.contains("stop") && state.recording.active { state.toggle_recording(); return true; }
    if c.contains("private") { state.toggle_private(); return true; }
    if c.contains("dismiss") { if let Some(id) = state.first_active_notification_id() { state.dismiss_notification(&id); } return true; }
    if c.contains("note") { state.status_message = Some("Note saved".to_string()); return true; }
    if c.contains("verify") || c.contains("fact") { state.status_message = Some("Fact-check queued".to_string()); return true; }
    if c.contains("calendar") || c.contains("schedule") { state.show_calendar = !state.show_calendar; return true; }
    if c.contains("research") || c.contains("docs") { state.show_research = !state.show_research; return true; }
    if c.contains("who") { if let Some(p) = state.selected_person() { state.status_message = Some(format!("{}, {}", p.name, p.relationship.label())); } return true; }
    state.status_message = Some(format!("\"{}\"", cmd)); true
}
