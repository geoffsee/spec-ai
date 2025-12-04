//! UI rendering for the OUI demo - Practical AI Assistant
//!
//! Layout designed with clinical optometry ergonomics:
//! - Upper zone: Time, status, compass (quick glance info)
//! - Center zone: Minimal - reticle and gaze indicator only
//! - Left peripheral: Notifications (attention when needed)
//! - Right peripheral: Contextual info (details on demand)
//! - Bottom zone: Ambient info, quick actions (secondary)

mod tactical_hud;
mod agent_status;
mod target_system;

use spec_ai_oui::renderer::{RenderBackend, Color};

use crate::state::DemoState;

/// Render the complete demo UI
pub fn render_demo(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Clear with very dark background (reduces eye strain)
    backend.clear(Color::Rgb(8, 10, 14));

    // Upper zone: Status bar and compass
    tactical_hud::render_status_bar(state, backend);
    tactical_hud::render_compass(state, backend);
    tactical_hud::render_next_event(state, backend);

    // Help hints (fades after startup)
    agent_status::render_help_hints(state, backend);

    // Left peripheral: Notifications
    target_system::render_notifications(state, backend);

    // Right peripheral: Contextual info (POI details)
    target_system::render_contextual_info(state, backend);

    // Center: Minimal reticle
    target_system::render_reticle(state, backend);

    // Bottom zone: System status and quick actions
    agent_status::render_system_status(state, backend);
    agent_status::render_quick_actions(state, backend);

    // Overlays
    if state.show_menu {
        render_menu(state, backend);
    }

    if state.show_calendar {
        agent_status::render_calendar_overlay(state, backend);
    }

    // Status toast (temporary messages)
    target_system::render_status_toast(state, backend);
}

/// Render quick menu - practical options
fn render_menu(state: &DemoState, backend: &mut dyn RenderBackend) {
    let cx = 0.5;
    let cy = 0.5;

    // Semi-transparent overlay
    backend.draw_hud_rect(0.35, 0.35, 0.3, 0.3, Color::Rgb(15, 18, 22));

    // Menu title
    backend.draw_hud_text(cx - 0.04, cy - 0.1, "Quick Menu", Color::White);

    // Menu items
    let items = [
        ('📅', "Calendar"),
        ('💬', "Messages"),
        ('🧭', "Navigate"),
        ('⚙', "Settings"),
    ];

    for (i, (icon, label)) in items.iter().enumerate() {
        let y = cy - 0.04 + (i as f32 * 0.04);
        let is_selected = state.menu_selection == Some(i);

        let color = if is_selected { Color::HUD_CYAN } else { Color::Grey };
        let indicator = if is_selected { "▸" } else { " " };

        let line = format!("{} {} {}", indicator, icon, label);
        backend.draw_hud_text(cx - 0.06, y, &line, color);
    }

    // Instructions
    backend.draw_hud_text(cx - 0.08, cy + 0.12, "↑↓ Navigate  ↵ Select  Esc Close", Color::Rgb(60, 65, 70));
}
