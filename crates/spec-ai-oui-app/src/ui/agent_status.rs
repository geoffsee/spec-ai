//! Notifications, overlays, quick actions

use spec_ai_oui::renderer::{RenderBackend, Color};
use crate::state::DemoState;

pub fn render_notifications(state: &DemoState, backend: &mut dyn RenderBackend) {
    let notifs: Vec<_> = state.notifications.iter().filter(|n| !n.dismissed && !n.context_relevant).take(2).collect();
    if notifs.is_empty() { return; }
    let x = 0.01; let y = 0.58;
    backend.draw_hud_text(x, y, &format!("Notifications ({})", state.active_notification_count()), Color::Grey);
    for (i, n) in notifs.iter().enumerate() {
        let ny = y + 0.03 + (i as f32 * 0.06);
        backend.draw_hud_text(x, ny, "●", n.priority.color());
        backend.draw_hud_text(x + 0.02, ny, &format!("{} · {}", n.source, n.timestamp), Color::DarkGrey);
        backend.draw_hud_text(x + 0.02, ny + 0.02, &n.title, Color::White);
        if state.density >= spec_ai_oui::InformationDensity::Normal {
            backend.draw_hud_text(x + 0.02, ny + 0.04, &(if n.preview.len() > 25 { format!("{}...", &n.preview[..22]) } else { n.preview.clone() }), Color::Grey);
        }
    }
}

pub fn render_quick_actions(state: &DemoState, backend: &mut dyn RenderBackend) {
    if state.density < spec_ai_oui::InformationDensity::Normal { return; }
    let y = 0.95; let sx = 0.32;
    for (i, a) in state.quick_actions.iter().enumerate() {
        let x = sx + (i as f32 * 0.09);
        let col = if i == 0 && state.recording.active { Color::ALERT_RED } else { Color::Grey };
        backend.draw_hud_text(x, y, &a.shortcut, Color::Rgb(50, 55, 60));
        backend.draw_hud_text(x + 0.035, y, &a.icon.to_string(), col);
    }
}

pub fn render_help(state: &DemoState, backend: &mut dyn RenderBackend) {
    if state.density < spec_ai_oui::InformationDensity::Maximum && state.tick > 200 { return; }
    backend.draw_hud_text(0.10, 0.97, "S:Select R:Record F:Photo N:Note V:Verify P:Private C:Cal I:Info M:Menu Space:Mode", Color::Rgb(45, 50, 55));
}

pub fn render_calendar_overlay(state: &DemoState, backend: &mut dyn RenderBackend) {
    if !state.show_calendar { return; }
    backend.draw_hud_rect(0.18, 0.18, 0.64, 0.64, Color::Rgb(12, 15, 20));
    let x = 0.22; let y = 0.22;
    backend.draw_hud_text(x, y, "Schedule", Color::White);
    backend.draw_hud_text(x, y + 0.03, &state.context.date, Color::Grey);
    for (i, e) in state.events.iter().enumerate() {
        let ey = y + 0.08 + (i as f32 * 0.10);
        backend.draw_hud_text(x, ey, &e.time, Color::HUD_CYAN);
        backend.draw_hud_text(x + 0.10, ey, &e.title, Color::White);
        if let Some(loc) = &e.location { backend.draw_hud_text(x + 0.10, ey + 0.025, loc, Color::Grey); }
        let until = if e.minutes_until < 60 { format!("in {}m", e.minutes_until) } else { format!("in {}h", e.minutes_until / 60) };
        backend.draw_hud_text(x + 0.40, ey, &until, if e.minutes_until <= 15 { Color::Yellow } else { Color::DarkGrey });
        if !e.attendees.is_empty() { backend.draw_hud_text(x + 0.10, ey + 0.045, &e.attendees.join(", "), Color::DarkGrey); }
    }
    backend.draw_hud_text(x, y + 0.50, "C or ESC to close", Color::Rgb(60, 65, 70));
}

pub fn render_research_overlay(state: &DemoState, backend: &mut dyn RenderBackend) {
    if !state.show_research { return; }
    backend.draw_hud_rect(0.18, 0.18, 0.64, 0.64, Color::Rgb(12, 15, 20));
    let x = 0.22; let y = 0.22;
    backend.draw_hud_text(x, y, "Research", Color::White);
    if let Some(p) = state.selected_person() { backend.draw_hud_text(x + 0.10, y, &p.name, Color::Grey); }
    for (i, d) in state.research_docs.iter().enumerate() {
        let dy = y + 0.06 + (i as f32 * 0.08);
        backend.draw_hud_text(x, dy, &format!("[{}]", d.doc_type), if d.doc_type == "PDF" { Color::ALERT_RED } else { Color::HUD_CYAN });
        backend.draw_hud_text(x + 0.08, dy, &d.title, Color::White);
        backend.draw_hud_rect(x + 0.40, dy + 0.005, d.relevance * 0.1, 0.01, Color::STATUS_GREEN);
        backend.draw_hud_text(x + 0.02, dy + 0.025, &d.snippet, Color::DarkGrey);
    }
    backend.draw_hud_text(x, y + 0.50, "I or ESC to close", Color::Rgb(60, 65, 70));
}

pub fn render_menu_overlay(state: &DemoState, backend: &mut dyn RenderBackend) {
    if !state.show_menu { return; }
    let cx = 0.5; let cy = 0.5;
    backend.draw_hud_rect(0.32, 0.32, 0.36, 0.36, Color::Rgb(15, 18, 22));
    backend.draw_hud_text(cx - 0.05, cy - 0.12, "Super Menu", Color::White);
    let items = [('📄', "Research"), ('📅', "Calendar"), ('●', "Record"), ('◈', "Private"), ('👥', "People"), ('⚙', "Settings")];
    for (i, (icon, label)) in items.iter().enumerate() {
        let y = cy - 0.06 + (i as f32 * 0.04);
        let sel = state.menu_selection == Some(i);
        backend.draw_hud_text(cx - 0.08, y, &format!("{} {} {}", if sel { "▸" } else { " " }, icon, label), if sel { Color::HUD_CYAN } else { Color::Grey });
    }
    backend.draw_hud_text(cx - 0.10, cy + 0.14, "↑↓ Nav  ↵ Select  Esc Close", Color::Rgb(60, 65, 70));
}
