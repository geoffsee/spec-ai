//! Minimal OUI - Two panel layout
//!
//! Layout:
//! - Upper left: Menu (Mode, Alerts, Settings)
//! - Upper right: Boxed event feed (or alternate views)

use crate::state::{DemoState, Focus, MenuItem, View};
use spec_ai_oui::renderer::{Color, RenderBackend};

/// Render the minimal OUI
pub fn render_demo(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Dark background
    backend.clear(Color::Rgb(8, 10, 14));

    // Render menu on upper left
    render_menu(state, backend);

    // Render main content on upper right
    render_content(state, backend);

    // Help hint at bottom
    render_help(state, backend);
}

/// Upper left: rolling menu
fn render_menu(state: &DemoState, backend: &mut dyn RenderBackend) {
    let x = 0.02;
    let y = 0.04;

    let focused = state.focus == Focus::Menu;
    let border_color = if focused {
        Color::HUD_CYAN
    } else {
        Color::Rgb(40, 45, 50)
    };

    // Draw box outline (simulated with lines)
    backend.draw_hud_text(x, y - 0.02, "┌─────────┐", border_color);

    for (i, item) in MenuItem::all().iter().enumerate() {
        let iy = y + (i as f32 * 0.04);
        let selected = state.menu_index == i;

        // Selection indicator
        let prefix = if selected && focused { "▸" } else { " " };

        // Color based on selection and focus
        let text_color = if selected && focused {
            Color::HUD_CYAN
        } else if selected {
            Color::White
        } else {
            Color::Grey
        };

        backend.draw_hud_text(x, iy, "│", border_color);
        backend.draw_hud_text(x + 0.01, iy, prefix, Color::HUD_CYAN);
        backend.draw_hud_text(x + 0.02, iy, item.label(), text_color);
        backend.draw_hud_text(x + 0.10, iy, "│", border_color);
    }

    let bottom_y = y + (MenuItem::all().len() as f32 * 0.04);
    backend.draw_hud_text(x, bottom_y, "└─────────┘", border_color);
}

/// Upper right: event feed or alternate view
fn render_content(state: &DemoState, backend: &mut dyn RenderBackend) {
    let x = 0.55;
    let y = 0.04;
    let width = 0.43;
    let height = 0.35;

    let focused = state.focus == Focus::Events;
    let border_color = if focused {
        Color::HUD_CYAN
    } else {
        Color::Rgb(40, 45, 50)
    };

    // Draw box
    backend.draw_hud_rect(x - 0.01, y - 0.02, width + 0.02, height, Color::Rgb(12, 14, 18));

    // Title
    let title = state.view.label();
    backend.draw_hud_text(x, y, title, border_color);

    match state.view {
        View::Events => render_event_feed(state, backend, x, y + 0.04, focused),
        View::Mode => render_mode_view(state, backend, x, y + 0.04),
        View::Alerts => render_alerts_view(state, backend, x, y + 0.04),
        View::Settings => render_settings_view(state, backend, x, y + 0.04),
    }
}

/// Render the rolling event feed
fn render_event_feed(state: &DemoState, backend: &mut dyn RenderBackend, x: f32, y: f32, focused: bool) {
    let visible_count = 5;

    for (i, event) in state
        .events
        .iter()
        .skip(state.scroll_offset)
        .take(visible_count)
        .enumerate()
    {
        let actual_index = state.scroll_offset + i;
        let ey = y + (i as f32 * 0.05);
        let selected = state.event_index == actual_index;

        // Priority indicator
        backend.draw_hud_text(x, ey, event.priority.indicator(), event.priority.color());

        // Selection highlight
        let text_color = if selected && focused {
            Color::HUD_CYAN
        } else if selected {
            Color::White
        } else {
            Color::Grey
        };

        // Timestamp
        backend.draw_hud_text(x + 0.02, ey, &event.timestamp, Color::DarkGrey);

        // Title
        backend.draw_hud_text(x + 0.08, ey, &event.title, text_color);

        // Detail on second line if selected
        if selected {
            backend.draw_hud_text(x + 0.08, ey + 0.025, &event.detail, Color::Rgb(80, 85, 90));
        }
    }

    // Scroll indicator
    if state.events.len() > visible_count {
        let scroll_y = y + (visible_count as f32 * 0.05);
        let shown = format!(
            "{}-{}/{}",
            state.scroll_offset + 1,
            (state.scroll_offset + visible_count).min(state.events.len()),
            state.events.len()
        );
        backend.draw_hud_text(x + 0.30, scroll_y, &shown, Color::DarkGrey);
    }
}

/// Mode view placeholder
fn render_mode_view(_state: &DemoState, backend: &mut dyn RenderBackend, x: f32, y: f32) {
    backend.draw_hud_text(x, y, "Display modes:", Color::Grey);
    backend.draw_hud_text(x, y + 0.04, "  Ambient", Color::DarkGrey);
    backend.draw_hud_text(x, y + 0.08, "  Focus", Color::DarkGrey);
    backend.draw_hud_text(x, y + 0.12, "  Private", Color::DarkGrey);
    backend.draw_hud_text(x, y + 0.18, "(placeholder)", Color::Rgb(50, 55, 60));
}

/// Alerts view placeholder
fn render_alerts_view(_state: &DemoState, backend: &mut dyn RenderBackend, x: f32, y: f32) {
    backend.draw_hud_text(x, y, "Alert settings:", Color::Grey);
    backend.draw_hud_text(x, y + 0.04, "  High priority only", Color::DarkGrey);
    backend.draw_hud_text(x, y + 0.08, "  Sound: On", Color::DarkGrey);
    backend.draw_hud_text(x, y + 0.12, "  Vibration: On", Color::DarkGrey);
    backend.draw_hud_text(x, y + 0.18, "(placeholder)", Color::Rgb(50, 55, 60));
}

/// Settings view placeholder
fn render_settings_view(_state: &DemoState, backend: &mut dyn RenderBackend, x: f32, y: f32) {
    backend.draw_hud_text(x, y, "Settings:", Color::Grey);
    backend.draw_hud_text(x, y + 0.04, "  Ring sensitivity", Color::DarkGrey);
    backend.draw_hud_text(x, y + 0.08, "  Scroll speed", Color::DarkGrey);
    backend.draw_hud_text(x, y + 0.12, "  Theme", Color::DarkGrey);
    backend.draw_hud_text(x, y + 0.18, "(placeholder)", Color::Rgb(50, 55, 60));
}

/// Help hint
fn render_help(state: &DemoState, backend: &mut dyn RenderBackend) {
    let help = if state.tick < 300 {
        "↑↓: Navigate  Tab: Switch panel  Enter: Select  Esc: Back  Q: Quit"
    } else {
        ""
    };
    backend.draw_hud_text(0.02, 0.96, help, Color::Rgb(45, 50, 55));
}
