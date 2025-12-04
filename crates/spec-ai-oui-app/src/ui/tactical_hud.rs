//! Status bar and navigation elements
//!
//! Ergonomic design principles from clinical optometry:
//! - Minimal peripheral distraction (subtle, low-contrast edges)
//! - Critical info in upper peripheral zone (naturally glanceable)
//! - Compass centered for quick reference without gaze shift
//! - Muted colors to reduce eye strain

use spec_ai_oui::renderer::{RenderBackend, Color};
use crate::state::DemoState;

/// Render the top status bar - designed for minimal eye strain
pub fn render_status_bar(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Left: Time and date (most frequently checked)
    // Using muted cyan - comfortable for extended viewing
    backend.draw_hud_text(0.01, 0.01, &state.current_time, Color::HUD_CYAN);

    // Only show date at higher density levels (reduce clutter)
    if state.density >= spec_ai_oui::InformationDensity::Normal {
        backend.draw_hud_text(0.08, 0.01, &state.context.date, Color::Grey);
    }

    // Center: Mode indicator (subtle, only when not in default mode)
    if state.mode != spec_ai_oui::DisplayMode::Navigate {
        let mode_icon = state.mode.icon();
        let mode_name = state.mode.name();
        let mode_text = format!("{} {}", mode_icon, mode_name);
        backend.draw_hud_text(0.45, 0.01, &mode_text, state.mode.theme_color());
    }

    // Right: System status icons (battery, connectivity)
    // Using subtle indicators - only draw attention when needed
    let battery_icon = if state.system.is_charging {
        '⚡'
    } else if state.system.battery_percent > 50 {
        '●'
    } else if state.system.battery_percent > 20 {
        '◐'
    } else {
        '○'
    };

    let battery_color = if state.system.battery_percent > 50 {
        Color::Grey  // Healthy - muted
    } else if state.system.battery_percent > 20 {
        Color::Yellow  // Attention needed
    } else {
        Color::ALERT_RED  // Critical
    };

    let battery_text = format!("{} {}%", battery_icon, state.system.battery_percent);
    backend.draw_hud_text(0.88, 0.01, &battery_text, battery_color);

    // WiFi indicator (only show if disconnected - otherwise invisible)
    if !state.system.wifi_connected {
        backend.draw_hud_text(0.85, 0.01, "⊘", Color::Yellow);
    }
}

/// Render compass bar - centered for quick glance navigation
pub fn render_compass(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Only render compass in Navigate mode or higher density
    if state.mode == spec_ai_oui::DisplayMode::Focus && state.density < spec_ai_oui::InformationDensity::High {
        return;
    }

    let y = 0.05;
    let cx = 0.5;
    let bar_width = 0.3;  // Narrower for less visual clutter

    // Subtle background - very low contrast
    backend.draw_hud_rect(cx - bar_width/2.0, y, bar_width, 0.03, Color::Rgb(15, 20, 25));

    // Cardinal directions - muted except North
    let cardinals = [("N", 0.0), ("E", 90.0), ("S", 180.0), ("W", 270.0)];

    for (label, bearing) in cardinals {
        let relative = normalize_angle(bearing - state.heading);
        if relative.abs() < 50.0 {
            let offset = (relative / 50.0) * (bar_width / 2.0);
            let x = cx + offset;
            // North gets subtle highlight, others very muted
            let color = if label == "N" {
                Color::White
            } else {
                Color::Rgb(80, 90, 100)  // Very subtle grey
            };
            backend.draw_hud_text(x, y + 0.005, label, color);
        }
    }

    // Current heading - subtle readout below
    if state.density >= spec_ai_oui::InformationDensity::Normal {
        let heading_text = format!("{:03.0}°", state.heading);
        backend.draw_hud_text(cx - 0.015, y + 0.035, &heading_text, Color::Grey);
    }

    // POI markers on compass - subtle indicators
    for poi in &state.points_of_interest {
        let bearing = (poi.position.x.atan2(poi.position.z)).to_degrees();
        let bearing = if bearing < 0.0 { bearing + 360.0 } else { bearing };
        let relative = normalize_angle(bearing - state.heading);

        if relative.abs() < 50.0 {
            let offset = (relative / 50.0) * (bar_width / 2.0);
            let x = cx + offset;
            // Selected POI is highlighted, others subtle
            let icon = if poi.selected { "◆" } else { "·" };
            let color = if poi.selected { poi.category.color() } else { Color::DarkGrey };
            backend.draw_hud_text(x, y + 0.018, icon, color);
        }
    }
}

/// Render upcoming event reminder - peripheral awareness zone
pub fn render_next_event(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Only show if there's an imminent event (within 60 minutes)
    if let Some(event) = state.context.upcoming_events.iter()
        .find(|e| e.minutes_until > 0 && e.minutes_until <= 60)
    {
        let y = 0.10;

        // Urgency determines visibility
        let prefix_str;
        let color = if event.minutes_until <= 5 {
            prefix_str = "NOW".to_string();
            Color::ALERT_RED
        } else if event.minutes_until <= 15 {
            prefix_str = format!("{}m", event.minutes_until);
            Color::Yellow
        } else {
            prefix_str = format!("{}m", event.minutes_until);
            Color::Grey
        };

        // Compact format: "15m Team Standup"
        let event_text = format!("{} {}", prefix_str, event.title);
        backend.draw_hud_text(0.01, y, &event_text, color);

        // Location hint at higher density
        if state.density >= spec_ai_oui::InformationDensity::High {
            if let Some(loc) = &event.location {
                backend.draw_hud_text(0.01, y + 0.025, loc, Color::DarkGrey);
            }
        }
    }
}

/// Normalize angle to -180..180
fn normalize_angle(mut angle: f32) -> f32 {
    while angle > 180.0 { angle -= 360.0; }
    while angle < -180.0 { angle += 360.0; }
    angle
}
