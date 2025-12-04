//! System status and quick actions display
//!
//! Ergonomic design principles:
//! - Bottom zone for secondary information (natural downward glance)
//! - Quick actions accessible but not prominent
//! - Weather/context info is ambient, not attention-grabbing
//! - Only show what's relevant to current mode

use spec_ai_oui::renderer::{RenderBackend, Color};
use crate::state::DemoState;

/// Render system status in bottom-left (ambient info zone)
pub fn render_system_status(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Only show at Normal density or higher
    if state.density < spec_ai_oui::InformationDensity::Normal {
        return;
    }

    let x = 0.01;
    let y = 0.94;

    // Weather and location - ambient context
    let weather_text = format!("{} {}  {}", state.context.weather, state.context.temperature, state.context.location);
    backend.draw_hud_text(x, y, &weather_text, Color::DarkGrey);

    // Tasks remaining (subtle productivity hint)
    if state.context.tasks_remaining > 0 && state.density >= spec_ai_oui::InformationDensity::High {
        let tasks_text = format!("{} tasks today", state.context.tasks_remaining);
        backend.draw_hud_text(x + 0.35, y, &tasks_text, Color::Rgb(60, 65, 70));
    }
}

/// Render quick actions in bottom-right (action zone)
pub fn render_quick_actions(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Hide in Focus mode to reduce distraction
    if state.mode == spec_ai_oui::DisplayMode::Focus {
        return;
    }

    // Only show at Normal density or higher
    if state.density < spec_ai_oui::InformationDensity::Normal {
        return;
    }

    let base_x = 0.75;
    let y = 0.94;

    // Compact action hints
    for (i, action) in state.quick_actions.iter().take(4).enumerate() {
        let x = base_x + (i as f32 * 0.06);

        // Shortcut key
        backend.draw_hud_text(x, y, &action.shortcut, Color::Rgb(50, 55, 60));

        // Action icon - slightly more visible if available
        let color = if action.available {
            Color::Grey
        } else {
            Color::Rgb(35, 40, 45)
        };
        backend.draw_hud_text(x + 0.025, y, &action.icon.to_string(), color);
    }
}

/// Render calendar overlay when activated
pub fn render_calendar_overlay(state: &DemoState, backend: &mut dyn RenderBackend) {
    if !state.show_calendar {
        return;
    }

    // Semi-transparent overlay effect (draw dim background)
    backend.draw_hud_rect(0.2, 0.2, 0.6, 0.6, Color::Rgb(15, 20, 25));

    let x = 0.25;
    let y = 0.25;

    // Header
    backend.draw_hud_text(x, y, "Today's Schedule", Color::White);
    backend.draw_hud_text(x, y + 0.03, &state.context.date, Color::Grey);

    // Events list
    for (i, event) in state.context.upcoming_events.iter().enumerate() {
        let ey = y + 0.08 + (i as f32 * 0.08);

        // Time
        backend.draw_hud_text(x, ey, &event.time, Color::HUD_CYAN);

        // Title
        backend.draw_hud_text(x + 0.1, ey, &event.title, Color::White);

        // Location (if any)
        if let Some(loc) = &event.location {
            backend.draw_hud_text(x + 0.1, ey + 0.025, loc, Color::Grey);
        }

        // Time until
        let until_text = if event.minutes_until <= 0 {
            "Now".to_string()
        } else if event.minutes_until < 60 {
            format!("in {}m", event.minutes_until)
        } else {
            format!("in {}h {}m", event.minutes_until / 60, event.minutes_until % 60)
        };

        let until_color = if event.minutes_until <= 15 {
            Color::Yellow
        } else {
            Color::DarkGrey
        };
        backend.draw_hud_text(x + 0.4, ey, &until_text, until_color);

        // Attendees at high density
        if state.density >= spec_ai_oui::InformationDensity::High && !event.attendees.is_empty() {
            let attendees = event.attendees.join(", ");
            let attendees_short = if attendees.len() > 30 {
                format!("{}...", &attendees[..27])
            } else {
                attendees
            };
            backend.draw_hud_text(x + 0.1, ey + 0.045, &attendees_short, Color::DarkGrey);
        }
    }

    // Close hint
    backend.draw_hud_text(x, y + 0.5, "ESC or C to close", Color::Rgb(60, 65, 70));
}

/// Render help hints (only when needed)
pub fn render_help_hints(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Only show at Maximum density or first few seconds
    if state.density < spec_ai_oui::InformationDensity::Maximum && state.tick > 300 {
        return;
    }

    let x = 0.01;
    let y = 0.12;

    // Key hints - very subtle
    let hints = [
        ("M", "Menu"),
        ("C", "Calendar"),
        ("P", "Select POI"),
        ("F", "Focus mode"),
        ("+/-", "Density"),
    ];

    backend.draw_hud_text(x, y, "Keys:", Color::Rgb(50, 55, 60));

    for (i, (key, desc)) in hints.iter().enumerate() {
        let hx = x + 0.05 + (i as f32 * 0.1);
        backend.draw_hud_text(hx, y, &format!("{}:{}", key, desc), Color::Rgb(45, 50, 55));
    }
}
