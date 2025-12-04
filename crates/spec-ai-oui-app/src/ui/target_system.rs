//! Target tracking and reticle

use spec_ai_oui::renderer::{RenderBackend, Color};
use crate::state::DemoState;

/// Render target markers
pub fn render_targets(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Show target info in bottom-right
    let x = 0.75;
    let y = 0.85;

    // Find locked target
    if let Some(target) = state.mission.targets.iter().find(|t| t.locked) {
        // Target header
        backend.draw_hud_text(x, y, "◆ TARGET LOCK", Color::ALERT_RED);

        // Target name
        backend.draw_hud_text(x, y + 0.025, &target.name, target.threat_level.color());

        // Threat level
        let threat = match target.threat_level {
            crate::state::ThreatLevel::None => "NEUTRAL",
            crate::state::ThreatLevel::Low => "LOW THREAT",
            crate::state::ThreatLevel::Medium => "MED THREAT",
            crate::state::ThreatLevel::High => "HIGH THREAT",
            crate::state::ThreatLevel::Critical => "CRITICAL",
        };
        backend.draw_hud_text(x, y + 0.05, threat, target.threat_level.color());

        // Lock progress
        if target.lock_progress < 1.0 {
            let progress = (target.lock_progress * 10.0) as usize;
            let bar = "▓".repeat(progress) + &"░".repeat(10 - progress);
            backend.draw_hud_text(x, y + 0.075, "LOCKING:", Color::Yellow);
            backend.draw_hud_text(x + 0.08, y + 0.075, &bar, Color::Yellow);
        } else {
            backend.draw_hud_text(x, y + 0.075, "LOCK: COMPLETE", Color::ALERT_RED);
        }

        // Distance (simplified)
        let distance = (target.position.x.powi(2) + target.position.z.powi(2)).sqrt();
        let dist_text = if distance >= 1000.0 {
            format!("DIST: {:.1}km", distance / 1000.0)
        } else {
            format!("DIST: {:.0}m", distance)
        };
        backend.draw_hud_text(x, y + 0.10, &dist_text, Color::Grey);
    } else {
        backend.draw_hud_text(x, y, "◇ NO TARGET", Color::Grey);
        backend.draw_hud_text(x, y + 0.025, "Press T to lock", Color::DarkGrey);
    }
}

/// Render center reticle
pub fn render_reticle(state: &DemoState, backend: &mut dyn RenderBackend) {
    let cx = 0.5;
    let cy = 0.5;

    // Determine reticle color based on lock status
    let locked_target = state.mission.targets.iter().any(|t| t.locked && t.lock_progress >= 1.0);
    let locking = state.mission.targets.iter().any(|t| t.locked && t.lock_progress < 1.0);

    let color = if locked_target {
        Color::ALERT_RED
    } else if locking {
        Color::Yellow
    } else {
        Color::HUD_CYAN
    };

    // Draw tactical reticle
    backend.draw_hud_text(cx - 0.02, cy - 0.02, "┌", color);
    backend.draw_hud_text(cx + 0.02, cy - 0.02, "┐", color);
    backend.draw_hud_text(cx - 0.02, cy + 0.02, "└", color);
    backend.draw_hud_text(cx + 0.02, cy + 0.02, "┘", color);

    // Center dot
    if locked_target {
        backend.draw_hud_text(cx, cy, "◉", Color::ALERT_RED);
    } else {
        backend.draw_hud_text(cx, cy, "·", color);
    }

    // Gaze indicator (small dot showing simulated eye position)
    let (gx, gy) = state.gaze_pos;
    if (gx - cx).abs() > 0.05 || (gy - cy).abs() > 0.05 {
        backend.draw_hud_text(gx, gy, "○", Color::Rgb(100, 100, 100));
    }
}
