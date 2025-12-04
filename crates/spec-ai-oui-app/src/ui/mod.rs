//! UI rendering for the OUI demo

mod mission_brief;
mod tactical_hud;
mod agent_status;
mod target_system;

use spec_ai_oui::renderer::{RenderBackend, Color};

use crate::state::DemoState;

/// Render the complete demo UI
pub fn render_demo(state: &DemoState, backend: &mut dyn RenderBackend) {
    let caps = backend.capabilities();

    // Clear with dark background
    backend.clear(Color::Rgb(5, 7, 12));

    // Render tactical HUD elements
    tactical_hud::render_status_bar(state, backend);
    tactical_hud::render_compass(state, backend);

    // Render agent status
    agent_status::render_agent_status(state, backend);

    // Render targets
    target_system::render_targets(state, backend);
    target_system::render_reticle(state, backend);

    // Render mission briefing overlay
    if state.show_briefing {
        mission_brief::render_briefing(state, backend);
    }

    // Render menu overlay
    if state.show_menu {
        render_menu(state, backend);
    }

    // Render status message
    if let Some(ref msg) = state.status_message {
        backend.draw_hud_text(0.3, 0.95, msg, Color::GOLD);
    }

    // Render controls hint
    backend.draw_hud_text(0.01, 0.97, "Tab:Focus M:Menu T:Target +/-:Density Ctrl+Q:Quit", Color::DarkGrey);
}

/// Render radial menu
fn render_menu(state: &DemoState, backend: &mut dyn RenderBackend) {
    let cx = 0.5;
    let cy = 0.5;

    // Semi-transparent overlay (just darken edges)
    backend.draw_hud_rect(0.3, 0.3, 0.4, 0.4, Color::Rgb(20, 20, 30));

    // Menu title
    backend.draw_hud_text(cx - 0.05, cy - 0.12, "QUICK MENU", Color::GOLD);

    // Menu items in a list (simplified from radial)
    let items = [
        ('🗺', "Map"),
        ('📡', "Comms"),
        ('🎒', "Inventory"),
        ('⚙', "Settings"),
    ];

    for (i, (icon, label)) in items.iter().enumerate() {
        let y = cy - 0.06 + (i as f32 * 0.05);
        let is_selected = state.menu_selection == Some(i);

        let color = if is_selected { Color::GOLD } else { Color::White };
        let indicator = if is_selected { ">" } else { " " };

        let line = format!("{} {} {}", indicator, icon, label);
        backend.draw_hud_text(cx - 0.08, y, &line, color);
    }

    // Instructions
    backend.draw_hud_text(cx - 0.1, cy + 0.15, "↑↓:Navigate Enter:Select Esc:Close", Color::Grey);
}
