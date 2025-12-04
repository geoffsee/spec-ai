//! Super OUI - Combined Intelligence Assistant
//!
//! Layout:
//! - Top: Status bar (time, mode, recording, private, next event, battery)
//! - Left: Conversation cues, hooks, fact-checks
//! - Right: Person profile, emotional state, rapport, people list
//! - Left-bottom: Notification stream
//! - Center: Context alerts, reticle
//! - Bottom: Quick actions, help
//! - Overlays: Calendar, Research, Menu

mod tactical_hud;
mod agent_status;
mod target_system;

use spec_ai_oui::renderer::{RenderBackend, Color};

use crate::state::DemoState;

/// Render the complete super OUI
pub fn render_demo(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Dark background
    backend.clear(Color::Rgb(8, 10, 14));

    // Top status bar
    tactical_hud::render_status_bar(state, backend);

    // Left: Conversation cues and fact-checks
    target_system::render_cues(state, backend);
    target_system::render_fact_checks(state, backend);

    // Right: Person profile and rapport
    tactical_hud::render_person_panel(state, backend);
    tactical_hud::render_rapport(state, backend);
    tactical_hud::render_people_list(state, backend);

    // Left-bottom: Notifications
    agent_status::render_notifications(state, backend);

    // Center: Context alert and reticle
    target_system::render_context_alert(state, backend);
    target_system::render_reticle(state, backend);

    // Bottom: Quick actions
    agent_status::render_quick_actions(state, backend);
    agent_status::render_help(state, backend);

    // Overlays (order matters - later renders on top)
    agent_status::render_calendar_overlay(state, backend);
    agent_status::render_research_overlay(state, backend);
    agent_status::render_menu_overlay(state, backend);

    // Status toast (always on top)
    target_system::render_toast(state, backend);
}
