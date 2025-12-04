//! Mission briefing panel

use spec_ai_oui::renderer::{RenderBackend, Color};
use crate::state::{DemoState, MissionStatus};

/// Render mission briefing overlay
pub fn render_briefing(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Panel dimensions
    let x = 0.15;
    let y = 0.1;
    let width = 0.7;
    let height = 0.75;

    // Draw panel background
    backend.draw_hud_rect(x, y, width, height, Color::GOLD);

    // Header
    let header = format!("MISSION BRIEFING: {}", state.mission.codename);
    backend.draw_hud_text(x + 0.02, y + 0.02, &header, Color::GOLD);
    backend.draw_hud_text(x + 0.02, y + 0.05, "─".repeat(40).as_str(), Color::DarkGrey);

    // Status
    let status_text = match state.mission.status {
        MissionStatus::Briefing => "STATUS: BRIEFING",
        MissionStatus::Active => "STATUS: ACTIVE",
        MissionStatus::Complete => "STATUS: COMPLETE",
        MissionStatus::Failed => "STATUS: FAILED",
    };
    let status_color = match state.mission.status {
        MissionStatus::Briefing => Color::HUD_CYAN,
        MissionStatus::Active => Color::STATUS_GREEN,
        MissionStatus::Complete => Color::GOLD,
        MissionStatus::Failed => Color::ALERT_RED,
    };
    backend.draw_hud_text(x + 0.02, y + 0.08, status_text, status_color);

    // Objective
    backend.draw_hud_text(x + 0.02, y + 0.12, "PRIMARY OBJECTIVE:", Color::HUD_CYAN);
    backend.draw_hud_text(x + 0.04, y + 0.15, &state.mission.objective, Color::White);

    // Intel
    backend.draw_hud_text(x + 0.02, y + 0.20, "INTEL:", Color::HUD_CYAN);
    for (i, intel) in state.mission.intel.iter().enumerate() {
        let intel_y = y + 0.23 + (i as f32 * 0.03);
        let intel_line = format!("• {}", intel);
        backend.draw_hud_text(x + 0.04, intel_y, &intel_line, Color::White);
    }

    // Targets
    backend.draw_hud_text(x + 0.02, y + 0.35, "KNOWN TARGETS:", Color::HUD_CYAN);
    for (i, target) in state.mission.targets.iter().enumerate() {
        let target_y = y + 0.38 + (i as f32 * 0.04);
        let threat = match target.threat_level {
            crate::state::ThreatLevel::None => "NEUTRAL",
            crate::state::ThreatLevel::Low => "LOW",
            crate::state::ThreatLevel::Medium => "MEDIUM",
            crate::state::ThreatLevel::High => "HIGH",
            crate::state::ThreatLevel::Critical => "CRITICAL",
        };
        let target_line = format!("• {} - Threat: {}", target.name, threat);
        backend.draw_hud_text(x + 0.04, target_y, &target_line, target.threat_level.color());
    }

    // Agent equipment
    backend.draw_hud_text(x + 0.02, y + 0.55, "EQUIPMENT:", Color::HUD_CYAN);
    for (i, gadget) in state.agent.gadgets.iter().enumerate() {
        let gadget_y = y + 0.58 + (i as f32 * 0.03);
        let status = if gadget.ready { "READY" } else { "COOLDOWN" };
        let color = if gadget.ready { Color::STATUS_GREEN } else { Color::Grey };
        let gadget_line = format!("• {} {} - {}", gadget.icon, gadget.name, status);
        backend.draw_hud_text(x + 0.04, gadget_y, &gadget_line, color);
    }

    // Action prompt
    if state.mission.status == MissionStatus::Briefing {
        backend.draw_hud_text(x + 0.02, y + height - 0.05, "─".repeat(40).as_str(), Color::DarkGrey);
        backend.draw_hud_text(x + 0.15, y + height - 0.02, "[ PRESS ENTER TO BEGIN MISSION ]", Color::GOLD);
    } else {
        backend.draw_hud_text(x + 0.25, y + height - 0.02, "[ B to close ]", Color::Grey);
    }
}
