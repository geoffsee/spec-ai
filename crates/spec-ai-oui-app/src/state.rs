//! OUI state derived from OpenTelemetry data streams
//!
//! The UI state is built from incoming telemetry events (spans, logs, metrics).
//! Navigation state (focus, indices) is local, but the data displayed comes
//! from the telemetry stream.

use std::collections::{HashMap, VecDeque};
use std::time::SystemTime;

use spec_ai_oui::renderer::Color;

use crate::telemetry::{SpanData, SpanStatus, TelemetryEvent, TelemetryStats, Trace};

/// Menu items on the left
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Traces,
    Spans,
    Services,
}

impl MenuItem {
    pub fn all() -> &'static [MenuItem] {
        &[MenuItem::Traces, MenuItem::Spans, MenuItem::Services]
    }

    pub fn label(&self) -> &'static str {
        match self {
            MenuItem::Traces => "Traces",
            MenuItem::Spans => "Spans",
            MenuItem::Services => "Services",
        }
    }
}

/// The view currently displayed on the right
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Feed,
    Traces,
    Spans,
    Services,
}

impl View {
    pub fn label(&self) -> &'static str {
        match self {
            View::Feed => "Event Feed",
            View::Traces => "Traces",
            View::Spans => "Spans",
            View::Services => "Services",
        }
    }
}

/// A displayable event in the feed (derived from TelemetryEvent)
#[derive(Debug, Clone)]
pub struct FeedEvent {
    pub id: usize,
    pub title: String,
    pub detail: String,
    pub timestamp: String,
    pub priority: EventPriority,
    pub source: TelemetryEvent,
}

impl FeedEvent {
    pub fn from_telemetry(id: usize, event: TelemetryEvent) -> Self {
        let timestamp = format_time(event.timestamp());
        let priority = match event.priority() {
            3 => EventPriority::High,
            2 => EventPriority::Normal,
            _ => EventPriority::Low,
        };

        let (title, detail) = match &event {
            TelemetryEvent::SpanStarted(span) => (
                format!("▶ {}", span.name),
                format!("{} | {}", span.service_name, span.kind.symbol()),
            ),
            TelemetryEvent::SpanEnded(span) => {
                let status_icon = match span.status {
                    SpanStatus::Ok => "✓",
                    SpanStatus::Error => "✗",
                    SpanStatus::Unset => "■",
                };
                let dur = span
                    .duration()
                    .map(|d| format!("{:.1}ms", d.as_secs_f64() * 1000.0))
                    .unwrap_or_else(|| "?".to_string());
                (
                    format!("{} {}", status_icon, span.name),
                    format!("{} | {} | {}", span.service_name, span.kind.symbol(), dur),
                )
            }
            TelemetryEvent::Log(log) => (
                format!("[{}] {}", log.severity.symbol(), truncate(&log.body, 30)),
                log.service_name.clone(),
            ),
            TelemetryEvent::Metric(metric) => {
                let val = match &metric.value {
                    crate::telemetry::MetricValue::Gauge(v) => format!("{:.2}", v),
                    crate::telemetry::MetricValue::Counter(v) => format!("{}", v),
                    crate::telemetry::MetricValue::Histogram { sum, count, .. } => {
                        format!("avg={:.2}", sum / *count as f64)
                    }
                };
                (
                    format!("📊 {}: {}", metric.name, val),
                    metric.service_name.clone(),
                )
            }
        };

        Self {
            id,
            title,
            detail,
            timestamp,
            priority,
            source: event,
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max])
    } else {
        s.to_string()
    }
}

fn format_time(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPriority {
    Low,
    Normal,
    High,
}

impl EventPriority {
    pub fn color(&self) -> Color {
        match self {
            EventPriority::Low => Color::DarkGrey,
            EventPriority::Normal => Color::Grey,
            EventPriority::High => Color::Yellow,
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            EventPriority::Low => "○",
            EventPriority::Normal => "●",
            EventPriority::High => "◆",
        }
    }
}

/// Which panel has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Menu,
    Content,
}

/// Main application state - derived from telemetry stream
#[derive(Debug, Clone)]
pub struct AppState {
    // Animation/timing
    pub tick: u64,

    // Navigation state
    pub focus: Focus,
    pub view: View,
    pub menu_index: usize,
    pub content_index: usize,
    pub scroll_offset: usize,

    // Telemetry data (derived from stream)
    pub feed_events: VecDeque<FeedEvent>,
    pub traces: HashMap<String, Trace>,
    pub services: HashMap<String, ServiceStats>,
    pub stats: TelemetryStats,

    // Configuration
    pub max_feed_events: usize,
    pub event_counter: usize,
}

/// Stats per service
#[derive(Debug, Clone, Default)]
pub struct ServiceStats {
    pub name: String,
    pub span_count: usize,
    pub error_count: usize,
    pub last_seen: Option<SystemTime>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            focus: Focus::Menu,
            view: View::Feed,
            menu_index: 0,
            content_index: 0,
            scroll_offset: 0,
            feed_events: VecDeque::new(),
            traces: HashMap::new(),
            services: HashMap::new(),
            stats: TelemetryStats::default(),
            max_feed_events: 100,
            event_counter: 0,
        }
    }

    /// Process an incoming telemetry event
    pub fn process_telemetry(&mut self, event: TelemetryEvent) {
        // Update stats
        match &event {
            TelemetryEvent::SpanStarted(span) | TelemetryEvent::SpanEnded(span) => {
                self.stats.total_spans += 1;
                if span.is_active() {
                    self.stats.active_spans += 1;
                }
                if span.status == SpanStatus::Error {
                    self.stats.error_spans += 1;
                }

                // Update trace
                let trace = self
                    .traces
                    .entry(span.trace_id.clone())
                    .or_insert_with(|| Trace::new(span.trace_id.clone()));
                trace.add_span(span.clone());

                // Update service stats
                let service = self
                    .services
                    .entry(span.service_name.clone())
                    .or_insert_with(|| ServiceStats {
                        name: span.service_name.clone(),
                        ..Default::default()
                    });
                service.span_count += 1;
                if span.status == SpanStatus::Error {
                    service.error_count += 1;
                }
                service.last_seen = Some(span.start_time);
            }
            TelemetryEvent::Log(log) => {
                self.stats.total_logs += 1;
                if log.severity >= crate::telemetry::Severity::Error {
                    self.stats.error_logs += 1;
                } else if log.severity == crate::telemetry::Severity::Warn {
                    self.stats.warn_logs += 1;
                }
            }
            TelemetryEvent::Metric(_) => {}
        }

        // Update services list
        self.stats.services = self.services.keys().cloned().collect();

        // Add to feed
        self.event_counter += 1;
        let feed_event = FeedEvent::from_telemetry(self.event_counter, event);
        self.feed_events.push_front(feed_event);

        // Trim feed to max size
        while self.feed_events.len() > self.max_feed_events {
            self.feed_events.pop_back();
        }
    }

    /// Get the list of items for the current view
    pub fn content_items(&self) -> Vec<ContentItem> {
        match self.view {
            View::Feed => self
                .feed_events
                .iter()
                .map(|e| ContentItem::Event(e.clone()))
                .collect(),
            View::Traces => self
                .traces
                .values()
                .map(|t| ContentItem::Trace(t.clone()))
                .collect(),
            View::Spans => self
                .feed_events
                .iter()
                .filter_map(|e| match &e.source {
                    TelemetryEvent::SpanEnded(s) => Some(ContentItem::Span(s.clone())),
                    _ => None,
                })
                .collect(),
            View::Services => self
                .services
                .values()
                .cloned()
                .map(ContentItem::Service)
                .collect(),
        }
    }

    pub fn content_len(&self) -> usize {
        match self.view {
            View::Feed => self.feed_events.len(),
            View::Traces => self.traces.len(),
            View::Spans => self
                .feed_events
                .iter()
                .filter(|e| matches!(e.source, TelemetryEvent::SpanEnded(_)))
                .count(),
            View::Services => self.services.len(),
        }
    }

    /// Move selection up within current focus
    pub fn scroll_up(&mut self) {
        match self.focus {
            Focus::Menu => {
                let len = MenuItem::all().len();
                self.menu_index = if self.menu_index == 0 {
                    len - 1
                } else {
                    self.menu_index - 1
                };
            }
            Focus::Content => {
                if self.content_index > 0 {
                    self.content_index -= 1;
                    if self.content_index < self.scroll_offset {
                        self.scroll_offset = self.content_index;
                    }
                }
            }
        }
    }

    /// Move selection down within current focus
    pub fn scroll_down(&mut self) {
        match self.focus {
            Focus::Menu => {
                let len = MenuItem::all().len();
                self.menu_index = (self.menu_index + 1) % len;
            }
            Focus::Content => {
                let len = self.content_len();
                if self.content_index < len.saturating_sub(1) {
                    self.content_index += 1;
                    // Keep selected item visible (assume 5 visible)
                    if self.content_index >= self.scroll_offset + 5 {
                        self.scroll_offset = self.content_index.saturating_sub(4);
                    }
                }
            }
        }
    }

    /// Toggle focus between menu and content
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Menu => Focus::Content,
            Focus::Content => Focus::Menu,
        };
    }

    /// Select current item
    pub fn select(&mut self) {
        match self.focus {
            Focus::Menu => {
                let item = MenuItem::all()[self.menu_index];
                self.view = match item {
                    MenuItem::Traces => View::Traces,
                    MenuItem::Spans => View::Spans,
                    MenuItem::Services => View::Services,
                };
                self.focus = Focus::Content;
                self.content_index = 0;
                self.scroll_offset = 0;
            }
            Focus::Content => {
                // Could expand selected item, for now just go back to feed
                self.view = View::Feed;
            }
        }
    }

    /// Back to default feed view
    pub fn back(&mut self) {
        self.view = View::Feed;
        self.focus = Focus::Menu;
        self.content_index = 0;
        self.scroll_offset = 0;
    }

    pub fn selected_event(&self) -> Option<&FeedEvent> {
        self.feed_events.get(self.content_index)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Content item for display
#[derive(Debug, Clone)]
pub enum ContentItem {
    Event(FeedEvent),
    Trace(Trace),
    Span(SpanData),
    Service(ServiceStats),
}
