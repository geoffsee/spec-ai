//! Conversation cues, fact-checks, context alerts

use spec_ai_oui::renderer::{RenderBackend, Color};
use crate::state::{DemoState, CueType};

pub fn render_cues(state: &DemoState, backend: &mut dyn RenderBackend) {
    if state.selected_person().is_none() || state.density < spec_ai_oui::InformationDensity::Low { return; }
    let x = 0.01; let y = 0.05;
    backend.draw_hud_text(x, y, "Cues", Color::Grey);
    for (i, cue) in state.cues.iter().take(5).enumerate() {
        let cy = y + 0.035 + (i as f32 * 0.05);
        backend.draw_hud_text(x, cy, &cue.cue_type.icon().to_string(), cue.cue_type.color());
        let col = if cue.cue_type == CueType::Avoid { Color::Rgb(120, 80, 80) } else { Color::White };
        let content = if cue.content.len() > 30 { format!("{}...", &cue.content[..27]) } else { cue.content.clone() };
        backend.draw_hud_text(x + 0.02, cy, &content, col);
    }
    // Hooks
    if let Some(p) = state.selected_person() {
        if !p.hooks.is_empty() && state.density >= spec_ai_oui::InformationDensity::Normal {
            let hy = y + 0.30;
            backend.draw_hud_text(x, hy, "Hooks:", Color::STATUS_GREEN);
            for (i, hook) in p.hooks.iter().take(2).enumerate() {
                backend.draw_hud_text(x, hy + 0.025 + (i as f32 * 0.025), &format!("→ {}", if hook.len() > 30 { &hook[..27] } else { hook }), Color::DarkGrey);
            }
        }
    }
}

pub fn render_fact_checks(state: &DemoState, backend: &mut dyn RenderBackend) {
    if state.fact_checks.is_empty() || state.density < spec_ai_oui::InformationDensity::Normal { return; }
    let x = 0.01; let y = 0.42;
    backend.draw_hud_text(x, y, "Fact Check", Color::Grey);
    for (i, fc) in state.fact_checks.iter().take(2).enumerate() {
        let fy = y + 0.03 + (i as f32 * 0.08);
        backend.draw_hud_text(x, fy, &format!("{} {}", fc.verdict.icon(), fc.verdict.label()), fc.verdict.color());
        let claim = if fc.claim.len() > 35 { format!("\"{}...\"", &fc.claim[..32]) } else { format!("\"{}\"", fc.claim) };
        backend.draw_hud_text(x, fy + 0.025, &claim, Color::White);
        backend.draw_hud_text(x + 0.30, fy, &fc.timestamp, Color::Rgb(60, 65, 70));
    }
}

pub fn render_context_alert(state: &DemoState, backend: &mut dyn RenderBackend) {
    if let Some(alert) = state.context_alert() {
        let cx = 0.22; let y = 0.46;
        backend.draw_hud_rect(cx - 0.01, y - 0.01, 0.56, 0.07, Color::Rgb(20, 30, 25));
        backend.draw_hud_text(cx, y, "◆", Color::STATUS_GREEN);
        backend.draw_hud_text(cx + 0.02, y, &alert.title, Color::STATUS_GREEN);
        backend.draw_hud_text(cx + 0.02, y + 0.025, &alert.preview, Color::White);
        backend.draw_hud_text(cx + 0.40, y + 0.04, "D dismiss", Color::Rgb(50, 55, 60));
    }
}

pub fn render_reticle(state: &DemoState, backend: &mut dyn RenderBackend) {
    let cx = 0.5; let cy = 0.5;
    if state.system.private_mode { backend.draw_hud_text(cx, cy, "◈", Color::Rgb(128, 0, 128)); return; }
    if state.recording.active { backend.draw_hud_text(cx, cy, "●", Color::ALERT_RED); return; }
    let col = if state.selected_person().is_some() { Color::HUD_CYAN } else { Color::Rgb(40, 45, 50) };
    backend.draw_hud_text(cx, cy, "·", col);
    let (gx, gy) = state.gaze_pos;
    if ((gx - cx).powi(2) + (gy - cy).powi(2)).sqrt() > 0.1 { backend.draw_hud_text(gx, gy, "○", Color::Rgb(35, 40, 45)); }
}

pub fn render_toast(state: &DemoState, backend: &mut dyn RenderBackend) {
    if let Some(msg) = &state.status_message {
        let cx = 0.5; let y = 0.92;
        let offset = (msg.len() as f32 * 0.005).min(0.15);
        backend.draw_hud_rect(cx - offset - 0.02, y - 0.01, offset * 2.0 + 0.04, 0.04, Color::Rgb(25, 30, 35));
        backend.draw_hud_text(cx - offset, y, msg, Color::White);
    }
}
