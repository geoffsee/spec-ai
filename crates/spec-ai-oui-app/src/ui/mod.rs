//! OpenTelemetry visualization UI
//!
//! Layout:
//! - Upper left: Menu (Traces, Spans, Services)
//! - Upper right: Event feed or filtered views
//! - Bottom: Stats bar

use crate::state::{AppState, ContentItem, Focus, MenuItem, View};
use crate::telemetry::SpanStatus;
use spec_ai_oui::renderer::{Color, RenderBackend};

/// Render the OUI app
pub fn render_app(state: &AppState, backend: &mut dyn RenderBackend) {
    // Dark background
    backend.clear(Color::Rgb(8, 10, 14));

    // Render menu on upper left
    render_menu(state, backend);

    // Render main content on upper right
    render_content(state, backend);

    // Stats bar at bottom
    render_stats(state, backend);

    // Help hint
    render_help(state, backend);
}

/// Upper left: menu panel
fn render_menu(state: &AppState, backend: &mut dyn RenderBackend) {
    let x = 0.02;
    let y = 0.04;

    let focused = state.focus == Focus::Menu;
    let border_color = if focused {
        Color::HUD_CYAN
    } else {
        Color::Rgb(40, 45, 50)
    };

    // Draw box outline
    backend.draw_hud_text(x, y - 0.02, "┌──────────┐", border_color);

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
        backend.draw_hud_text(x + 0.11, iy, "│", border_color);
    }

    let bottom_y = y + (MenuItem::all().len() as f32 * 0.04);
    backend.draw_hud_text(x, bottom_y, "└──────────┘", border_color);
}

/// Upper right: content panel
fn render_content(state: &AppState, backend: &mut dyn RenderBackend) {
    let x = 0.55;
    let y = 0.04;
    let width = 0.43;
    let height = 0.45;

    let focused = state.focus == Focus::Content;
    let border_color = if focused {
        Color::HUD_CYAN
    } else {
        Color::Rgb(40, 45, 50)
    };

    // Draw box background
    backend.draw_hud_rect(x - 0.01, y - 0.02, width + 0.02, height, Color::Rgb(12, 14, 18));

    // Title with count
    let count = state.content_len();
    let title = format!("{} ({})", state.view.label(), count);
    backend.draw_hud_text(x, y, &title, border_color);

    match state.view {
        View::Feed => render_feed(state, backend, x, y + 0.04, focused),
        View::Traces => render_traces(state, backend, x, y + 0.04, focused),
        View::Spans => render_spans(state, backend, x, y + 0.04, focused),
        View::Services => render_services(state, backend, x, y + 0.04, focused),
    }
}

/// Render the event feed
fn render_feed(state: &AppState, backend: &mut dyn RenderBackend, x: f32, y: f32, focused: bool) {
    let visible_count = 6;

    if state.feed_events.is_empty() {
        backend.draw_hud_text(x, y, "Waiting for telemetry...", Color::DarkGrey);
        return;
    }

    for (i, event) in state
        .feed_events
        .iter()
        .skip(state.scroll_offset)
        .take(visible_count)
        .enumerate()
    {
        let actual_index = state.scroll_offset + i;
        let ey = y + (i as f32 * 0.05);
        let selected = state.content_index == actual_index;

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

        // Title (truncated)
        let title = truncate(&event.title, 25);
        backend.draw_hud_text(x + 0.10, ey, &title, text_color);

        // Detail on second line if selected
        if selected {
            let detail = truncate(&event.detail, 35);
            backend.draw_hud_text(x + 0.10, ey + 0.025, &detail, Color::Rgb(80, 85, 90));
        }
    }

    // Scroll indicator
    if state.feed_events.len() > visible_count {
        let scroll_y = y + (visible_count as f32 * 0.05);
        let shown = format!(
            "{}-{}/{}",
            state.scroll_offset + 1,
            (state.scroll_offset + visible_count).min(state.feed_events.len()),
            state.feed_events.len()
        );
        backend.draw_hud_text(x + 0.30, scroll_y, &shown, Color::DarkGrey);
    }
}

/// Render traces view
fn render_traces(state: &AppState, backend: &mut dyn RenderBackend, x: f32, y: f32, focused: bool) {
    let visible_count = 6;
    let traces: Vec<_> = state.traces.values().collect();

    if traces.is_empty() {
        backend.draw_hud_text(x, y, "No traces yet...", Color::DarkGrey);
        return;
    }

    for (i, trace) in traces.iter().skip(state.scroll_offset).take(visible_count).enumerate() {
        let actual_index = state.scroll_offset + i;
        let ty = y + (i as f32 * 0.05);
        let selected = state.content_index == actual_index;

        // Status indicator
        let (indicator, ind_color) = if trace.is_active() {
            ("◉", Color::Yellow)
        } else if trace.spans.values().any(|s| s.status == SpanStatus::Error) {
            ("✗", Color::Red)
        } else {
            ("✓", Color::Green)
        };
        backend.draw_hud_text(x, ty, indicator, ind_color);

        let text_color = if selected && focused {
            Color::HUD_CYAN
        } else if selected {
            Color::White
        } else {
            Color::Grey
        };

        // Trace ID (shortened)
        let trace_id = truncate(&trace.trace_id, 12);
        backend.draw_hud_text(x + 0.02, ty, &trace_id, text_color);

        // Span count
        let span_count = format!("{} spans", trace.spans.len());
        backend.draw_hud_text(x + 0.16, ty, &span_count, Color::DarkGrey);

        // Duration if available
        if let Some(dur) = trace.duration() {
            let dur_str = format!("{:.1}ms", dur.as_secs_f64() * 1000.0);
            backend.draw_hud_text(x + 0.28, ty, &dur_str, Color::DarkGrey);
        }

        // Service name if selected
        if selected {
            if let Some(service) = trace.service_name() {
                backend.draw_hud_text(x + 0.02, ty + 0.025, service, Color::Rgb(80, 85, 90));
            }
        }
    }
}

/// Render spans view
fn render_spans(state: &AppState, backend: &mut dyn RenderBackend, x: f32, y: f32, focused: bool) {
    let visible_count = 6;
    let items = state.content_items();
    let spans: Vec<_> = items
        .iter()
        .filter_map(|item| {
            if let ContentItem::Span(span) = item {
                Some(span)
            } else {
                None
            }
        })
        .collect();

    if spans.is_empty() {
        backend.draw_hud_text(x, y, "No spans yet...", Color::DarkGrey);
        return;
    }

    for (i, span) in spans.iter().skip(state.scroll_offset).take(visible_count).enumerate() {
        let actual_index = state.scroll_offset + i;
        let sy = y + (i as f32 * 0.05);
        let selected = state.content_index == actual_index;

        // Status indicator
        let (indicator, ind_color) = match span.status {
            SpanStatus::Ok => ("✓", Color::Green),
            SpanStatus::Error => ("✗", Color::Red),
            SpanStatus::Unset => ("○", Color::Grey),
        };
        backend.draw_hud_text(x, sy, indicator, ind_color);

        let text_color = if selected && focused {
            Color::HUD_CYAN
        } else if selected {
            Color::White
        } else {
            Color::Grey
        };

        // Kind symbol
        backend.draw_hud_text(x + 0.02, sy, span.kind.symbol(), Color::DarkGrey);

        // Span name
        let name = truncate(&span.name, 20);
        backend.draw_hud_text(x + 0.04, sy, &name, text_color);

        // Duration
        if let Some(dur) = span.duration() {
            let dur_str = format!("{:.1}ms", dur.as_secs_f64() * 1000.0);
            backend.draw_hud_text(x + 0.28, sy, &dur_str, Color::DarkGrey);
        }

        // Service name if selected
        if selected {
            backend.draw_hud_text(x + 0.04, sy + 0.025, &span.service_name, Color::Rgb(80, 85, 90));
        }
    }
}

/// Render services view
fn render_services(state: &AppState, backend: &mut dyn RenderBackend, x: f32, y: f32, focused: bool) {
    let visible_count = 6;
    let services: Vec<_> = state.services.values().collect();

    if services.is_empty() {
        backend.draw_hud_text(x, y, "No services yet...", Color::DarkGrey);
        return;
    }

    for (i, service) in services.iter().skip(state.scroll_offset).take(visible_count).enumerate() {
        let actual_index = state.scroll_offset + i;
        let sy = y + (i as f32 * 0.05);
        let selected = state.content_index == actual_index;

        // Health indicator based on error rate
        let error_rate = if service.span_count > 0 {
            service.error_count as f64 / service.span_count as f64
        } else {
            0.0
        };
        let (indicator, ind_color) = if error_rate > 0.1 {
            ("●", Color::Red)
        } else if error_rate > 0.0 {
            ("●", Color::Yellow)
        } else {
            ("●", Color::Green)
        };
        backend.draw_hud_text(x, sy, indicator, ind_color);

        let text_color = if selected && focused {
            Color::HUD_CYAN
        } else if selected {
            Color::White
        } else {
            Color::Grey
        };

        // Service name
        let name = truncate(&service.name, 20);
        backend.draw_hud_text(x + 0.02, sy, &name, text_color);

        // Span count
        let count = format!("{} spans", service.span_count);
        backend.draw_hud_text(x + 0.24, sy, &count, Color::DarkGrey);

        // Error count if any
        if service.error_count > 0 {
            let errors = format!("{} err", service.error_count);
            backend.draw_hud_text(x + 0.34, sy, &errors, Color::Red);
        }
    }
}

/// Stats bar at bottom
fn render_stats(state: &AppState, backend: &mut dyn RenderBackend) {
    let y = 0.90;

    // Stats summary
    let stats = &state.stats;
    let spans_str = format!(
        "Spans: {} ({} active, {} err)",
        stats.total_spans, stats.active_spans, stats.error_spans
    );
    backend.draw_hud_text(0.02, y, &spans_str, Color::Grey);

    // Traces count
    let traces_str = format!("Traces: {}", state.traces.len());
    backend.draw_hud_text(0.45, y, &traces_str, Color::Grey);

    // Services count
    let services_str = format!("Services: {}", state.services.len());
    backend.draw_hud_text(0.65, y, &services_str, Color::Grey);

    // OTLP status indicator
    backend.draw_hud_text(0.85, y, "OTLP ●", Color::Green);
}

/// Help hint
fn render_help(state: &AppState, backend: &mut dyn RenderBackend) {
    let help = if state.tick < 300 {
        "j/k: Navigate  Tab: Switch panel  Enter: Select  Esc: Back  Q: Quit"
    } else {
        ""
    };
    backend.draw_hud_text(0.02, 0.96, help, Color::Rgb(45, 50, 55));
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        s.chars().take(max - 2).collect::<String>() + ".."
    } else {
        s.to_string()
    }
}
