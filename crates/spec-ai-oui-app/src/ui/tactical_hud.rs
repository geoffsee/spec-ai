//! Top bar, person panel, rapport indicators

use spec_ai_oui::renderer::{RenderBackend, Color};
use crate::state::DemoState;

pub fn render_status_bar(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Recording indicator
    if state.recording.active {
        backend.draw_hud_rect(0.0, 0.0, 1.0, 0.005, Color::ALERT_RED);
        backend.draw_hud_text(0.42, 0.01, &format!("● REC {:02}:{:02}", state.recording.duration_secs / 60, state.recording.duration_secs % 60), Color::ALERT_RED);
    }
    // Time, mode
    backend.draw_hud_text(0.01, 0.01, &state.context.current_time, Color::Grey);
    let mode_text = format!("{} {}", state.mode.icon(), state.mode.name());
    backend.draw_hud_text(0.10, 0.01, &mode_text, state.mode.theme_color());
    // Private indicator
    if state.system.private_mode { backend.draw_hud_text(0.25, 0.01, "◈ PRIVATE", Color::Rgb(128, 0, 128)); }
    // Next event countdown
    if let Some(e) = state.next_event() {
        let color = if e.minutes_until <= 5 { Color::ALERT_RED } else if e.minutes_until <= 15 { Color::Yellow } else { Color::Grey };
        backend.draw_hud_text(0.70, 0.01, &format!("{}m {}", e.minutes_until, e.title), color);
    }
    // Battery
    backend.draw_hud_text(0.94, 0.01, &format!("{}%", state.system.battery_percent), if state.system.battery_percent > 20 { Color::Grey } else { Color::ALERT_RED });
}

pub fn render_person_panel(state: &DemoState, backend: &mut dyn RenderBackend) {
    let x = 0.58; let y = 0.05;
    if let Some(p) = state.selected_person() {
        backend.draw_hud_text(x, y, &p.name, Color::White);
        backend.draw_hud_text(x + 0.22, y, p.relationship.label(), p.relationship.color());
        backend.draw_hud_text(x + 0.30, y, &format!("{}", p.reliability.icon()), p.reliability.color());
        if let Some(t) = &p.title { backend.draw_hud_text(x, y + 0.025, t, Color::DarkGrey); }
        // Emotional state + engagement
        backend.draw_hud_text(x, y + 0.05, &format!("{} {}", p.emotional_state.icon(), p.emotional_state.label()), p.emotional_state.color());
        let bars = (p.engagement * 10.0) as usize;
        backend.draw_hud_text(x + 0.12, y + 0.05, &("█".repeat(bars) + &"░".repeat(10 - bars)), Color::HUD_CYAN);
        // Comm style tip
        if state.density >= spec_ai_oui::InformationDensity::Normal { backend.draw_hud_text(x, y + 0.08, p.comm_style.tip(), Color::Rgb(80, 85, 90)); }
        // Last interaction
        if state.density >= spec_ai_oui::InformationDensity::High { if let Some(last) = &p.last_interaction { backend.draw_hud_text(x, y + 0.11, last, Color::Rgb(60, 65, 70)); } }
        // Shared context
        if state.density >= spec_ai_oui::InformationDensity::High && !p.shared_context.is_empty() {
            for (i, ctx) in p.shared_context.iter().take(2).enumerate() { backend.draw_hud_text(x, y + 0.14 + (i as f32 * 0.02), &format!("· {}", ctx), Color::DarkGrey); }
        }
    } else {
        backend.draw_hud_text(x, y, "No selection", Color::DarkGrey);
        backend.draw_hud_text(x, y + 0.025, "S: select", Color::Rgb(50, 55, 60));
    }
}

pub fn render_rapport(state: &DemoState, backend: &mut dyn RenderBackend) {
    if state.selected_person().is_none() || state.density < spec_ai_oui::InformationDensity::Normal { return; }
    let x = 0.58; let y = 0.28;
    backend.draw_hud_text(x, y, "Rapport", Color::Grey);
    for (i, r) in state.rapport.iter().enumerate() {
        let iy = y + 0.03 + (i as f32 * 0.03);
        backend.draw_hud_text(x, iy, &r.metric, Color::DarkGrey);
        backend.draw_hud_text(x + 0.10, iy, &("●".repeat((r.level * 5.0) as usize) + &"○".repeat(5 - (r.level * 5.0) as usize)), Color::HUD_CYAN);
        backend.draw_hud_text(x + 0.20, iy, &r.trend.icon().to_string(), r.trend.color());
    }
}

pub fn render_people_list(state: &DemoState, backend: &mut dyn RenderBackend) {
    if state.density < spec_ai_oui::InformationDensity::High { return; }
    let x = 0.58; let y = 0.42;
    backend.draw_hud_text(x, y, "Nearby", Color::Grey);
    for (i, p) in state.people.iter().take(4).enumerate() {
        let py = y + 0.03 + (i as f32 * 0.035);
        backend.draw_hud_text(x, py, if p.selected { "▸" } else { " " }, Color::HUD_CYAN);
        backend.draw_hud_text(x + 0.02, py, &p.name, if p.selected { Color::White } else { Color::DarkGrey });
        backend.draw_hud_text(x + 0.22, py, &format!("{:.1}m", p.distance_meters), Color::Rgb(60, 65, 70));
    }
}
