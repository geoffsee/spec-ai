//! Agent status display

use spec_ai_oui::renderer::{RenderBackend, Color};
use crate::state::DemoState;

/// Render agent status in bottom-left
pub fn render_agent_status(state: &DemoState, backend: &mut dyn RenderBackend) {
    let x = 0.01;
    let y = 0.85;

    // Health bar
    let health_pct = state.agent.health / 100.0;
    let health_bars = (health_pct * 10.0) as usize;
    let health_color = if health_pct > 0.6 { Color::STATUS_GREEN }
        else if health_pct > 0.3 { Color::Yellow }
        else { Color::ALERT_RED };
    let health_bar = "█".repeat(health_bars) + &"░".repeat(10 - health_bars);
    backend.draw_hud_text(x, y, "HP", Color::Grey);
    backend.draw_hud_text(x + 0.03, y, &health_bar, health_color);
    backend.draw_hud_text(x + 0.14, y, &format!("{:.0}%", state.agent.health), health_color);

    // Shield bar
    let shield_pct = state.agent.shields / 100.0;
    let shield_bars = (shield_pct * 10.0) as usize;
    let shield_bar = "█".repeat(shield_bars) + &"░".repeat(10 - shield_bars);
    backend.draw_hud_text(x, y + 0.025, "SH", Color::Grey);
    backend.draw_hud_text(x + 0.03, y + 0.025, &shield_bar, Color::HUD_CYAN);
    backend.draw_hud_text(x + 0.14, y + 0.025, &format!("{:.0}%", state.agent.shields), Color::HUD_CYAN);

    // Ammo
    backend.draw_hud_text(x, y + 0.05, &format!("AMMO: {}", state.agent.ammo), Color::White);

    // Gadgets
    let gadget_y = y + 0.075;
    for (i, gadget) in state.agent.gadgets.iter().enumerate() {
        let gx = x + (i as f32 * 0.06);
        let color = if gadget.ready { Color::STATUS_GREEN } else { Color::Grey };
        let key = format!("[{}]", i + 1);
        backend.draw_hud_text(gx, gadget_y, &key, Color::DarkGrey);
        backend.draw_hud_text(gx + 0.025, gadget_y, &gadget.icon.to_string(), color);
    }
}
