//! Tactical HUD elements

use spec_ai_oui::renderer::{RenderBackend, Color};
use crate::state::DemoState;

/// Render the top status bar
pub fn render_status_bar(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Left: App name and mode
    let mode_icon = state.mode.icon();
    let mode_name = state.mode.name();
    let left_text = format!("SPEC-AI {} {}", mode_icon, mode_name);
    backend.draw_hud_text(0.01, 0.01, &left_text, Color::HUD_CYAN);

    // Center: Mission codename
    let center_x = 0.4;
    backend.draw_hud_text(center_x, 0.01, &state.mission.codename, Color::GOLD);

    // Right: Density level
    let density_text = match state.density {
        spec_ai_oui::InformationDensity::Minimal => "MIN",
        spec_ai_oui::InformationDensity::Low => "LOW",
        spec_ai_oui::InformationDensity::Normal => "NRM",
        spec_ai_oui::InformationDensity::High => "HI",
        spec_ai_oui::InformationDensity::Maximum => "MAX",
    };
    let right_text = format!("DENSITY:{}", density_text);
    backend.draw_hud_text(0.85, 0.01, &right_text, Color::Grey);

    // Separator line
    backend.draw_hud_text(0.0, 0.03, "─".repeat(80).as_str(), Color::DarkGrey);
}

/// Render compass bar
pub fn render_compass(state: &DemoState, backend: &mut dyn RenderBackend) {
    let y = 0.05;
    let cx = 0.5;
    let bar_width = 0.4;

    // Draw compass bar background
    backend.draw_hud_rect(cx - bar_width/2.0, y, bar_width, 0.04, Color::Rgb(20, 25, 35));

    // Cardinal directions
    let cardinals = [("N", 0.0), ("NE", 45.0), ("E", 90.0), ("SE", 135.0),
                     ("S", 180.0), ("SW", 225.0), ("W", 270.0), ("NW", 315.0)];

    for (label, bearing) in cardinals {
        let relative = normalize_angle(bearing - state.heading);
        if relative.abs() < 60.0 {
            let offset = (relative / 60.0) * (bar_width / 2.0);
            let x = cx + offset;
            let color = if label == "N" { Color::ALERT_RED } else { Color::White };
            backend.draw_hud_text(x, y + 0.01, label, color);
        }
    }

    // Current heading readout
    let heading_text = format!("{:03.0}°", state.heading);
    backend.draw_hud_text(cx - 0.02, y + 0.045, &heading_text, Color::HUD_CYAN);

    // Draw target markers on compass
    for target in &state.mission.targets {
        // Calculate bearing to target (simplified - just use x position)
        let bearing = (target.position.x.atan2(target.position.z)).to_degrees();
        let bearing = if bearing < 0.0 { bearing + 360.0 } else { bearing };
        let relative = normalize_angle(bearing - state.heading);

        if relative.abs() < 60.0 {
            let offset = (relative / 60.0) * (bar_width / 2.0);
            let x = cx + offset;
            let icon = if target.locked { "◆" } else { "◇" };
            backend.draw_hud_text(x, y + 0.025, icon, target.threat_level.color());
        }
    }
}

/// Normalize angle to -180..180
fn normalize_angle(mut angle: f32) -> f32 {
    while angle > 180.0 { angle -= 360.0; }
    while angle < -180.0 { angle += 360.0; }
    angle
}
