//! Contextual information and point-of-interest display
//!
//! Ergonomic design principles:
//! - Information appears in natural peripheral zones
//! - Selected items get focus, others fade to reduce distraction
//! - Gaze indicator is subtle to avoid attention competition
//! - Distance/direction info uses natural eye movement patterns

use spec_ai_oui::renderer::{RenderBackend, Color};
use crate::state::DemoState;

/// Render contextual information panel (right side - peripheral zone)
pub fn render_contextual_info(state: &DemoState, backend: &mut dyn RenderBackend) {
    let x = 0.75;
    let y = 0.15;

    // Show selected POI details
    if let Some(poi) = state.points_of_interest.iter().find(|p| p.selected) {
        // POI name with category indicator
        let header = format!("{} {}", poi.category.icon(), poi.name);
        backend.draw_hud_text(x, y, &header, poi.category.color());

        // Distance - formatted naturally
        let dist_text = if poi.distance_meters >= 1000.0 {
            format!("{:.1} km away", poi.distance_meters / 1000.0)
        } else {
            format!("{:.0} m away", poi.distance_meters)
        };
        backend.draw_hud_text(x, y + 0.025, &dist_text, Color::Grey);

        // Details (at higher density)
        if state.density >= spec_ai_oui::InformationDensity::Normal {
            for (i, detail) in poi.details.iter().take(2).enumerate() {
                let dy = y + 0.05 + (i as f32 * 0.025);
                backend.draw_hud_text(x, dy, detail, Color::DarkGrey);
            }
        }

        // Action hint
        backend.draw_hud_text(x, y + 0.12, "↵ Navigate", Color::Rgb(60, 70, 80));
    } else {
        // No selection - show hint
        backend.draw_hud_text(x, y, "No selection", Color::DarkGrey);
        backend.draw_hud_text(x, y + 0.025, "P to select nearby", Color::Rgb(50, 55, 60));
    }
}

/// Render notifications panel (left peripheral zone)
pub fn render_notifications(state: &DemoState, backend: &mut dyn RenderBackend) {
    // Only show at Normal density or higher
    if state.density < spec_ai_oui::InformationDensity::Normal {
        return;
    }

    let x = 0.01;
    let base_y = 0.75;

    // Show unread count if any
    let unread = state.unread_count();
    if unread == 0 {
        return;
    }

    // Notification header
    let header = if unread == 1 {
        "1 notification".to_string()
    } else {
        format!("{} notifications", unread)
    };
    backend.draw_hud_text(x, base_y, &header, Color::Grey);

    // Show most recent unread notification
    if let Some(notif) = state.notifications.iter().find(|n| !n.read) {
        let y = base_y + 0.025;

        // Priority indicator + source
        let indicator = notif.priority.icon();
        let source_text = format!("{} {}", indicator, notif.source);
        backend.draw_hud_text(x, y, &source_text, notif.priority.color());

        // Title
        backend.draw_hud_text(x, y + 0.025, &notif.title, Color::White);

        // Preview (truncated for ergonomics)
        let preview = if notif.preview.len() > 30 {
            format!("{}...", &notif.preview[..27])
        } else {
            notif.preview.clone()
        };
        backend.draw_hud_text(x, y + 0.05, &preview, Color::Grey);

        // Action hint
        backend.draw_hud_text(x, y + 0.08, "N dismiss", Color::Rgb(50, 55, 60));
    }
}

/// Render center reticle - minimal, ergonomic design
pub fn render_reticle(state: &DemoState, backend: &mut dyn RenderBackend) {
    let cx = 0.5;
    let cy = 0.5;

    // In Focus mode, no reticle (reduce visual noise)
    if state.mode == spec_ai_oui::DisplayMode::Focus {
        return;
    }

    // Very subtle center indicator - just a small dot
    // Only visible enough to confirm gaze tracking is working
    let selected = state.points_of_interest.iter().any(|p| p.selected);

    if selected {
        // Slightly more visible when something is selected
        backend.draw_hud_text(cx, cy, "·", Color::HUD_CYAN);
    } else {
        // Nearly invisible - just a hint
        backend.draw_hud_text(cx, cy, "·", Color::Rgb(40, 45, 50));
    }

    // Gaze position indicator (when gaze is away from center)
    let (gx, gy) = state.gaze_pos;
    let distance = ((gx - cx).powi(2) + (gy - cy).powi(2)).sqrt();

    // Only show gaze indicator if significantly off-center
    if distance > 0.1 {
        // Very subtle - just confirms where system thinks you're looking
        backend.draw_hud_text(gx, gy, "○", Color::Rgb(35, 40, 45));
    }
}

/// Render status toast message (center-bottom, temporary)
pub fn render_status_toast(state: &DemoState, backend: &mut dyn RenderBackend) {
    if let Some(msg) = &state.status_message {
        let cx = 0.5;
        let y = 0.92;

        // Center the message approximately
        let offset = (msg.len() as f32 * 0.005).min(0.15);

        // Subtle background
        backend.draw_hud_rect(cx - offset - 0.02, y - 0.01, offset * 2.0 + 0.04, 0.04, Color::Rgb(25, 30, 35));

        // Message text
        backend.draw_hud_text(cx - offset, y, msg, Color::White);
    }
}
