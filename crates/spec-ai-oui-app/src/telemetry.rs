//! OpenTelemetry data model for UI state derivation
//!
//! This module defines the data structures that represent OpenTelemetry
//! telemetry data (spans, logs, metrics) in a form suitable for UI rendering.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Status of a span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error,
}

impl Default for SpanStatus {
    fn default() -> Self {
        Self::Unset
    }
}

/// A single span from a trace
#[derive(Debug, Clone)]
pub struct SpanData {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: SpanKind,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub status: SpanStatus,
    pub attributes: HashMap<String, String>,
    pub service_name: String,
}

impl SpanData {
    /// Duration of the span (if completed)
    pub fn duration(&self) -> Option<Duration> {
        self.end_time.map(|end| {
            end.duration_since(self.start_time)
                .unwrap_or(Duration::ZERO)
        })
    }

    /// Whether this span is still active
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }

    /// Short display name for the span
    pub fn display_name(&self) -> &str {
        &self.name
    }
}

/// Kind of span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpanKind {
    #[default]
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

impl SpanKind {
    pub fn symbol(&self) -> &'static str {
        match self {
            SpanKind::Internal => "○",
            SpanKind::Server => "◉",
            SpanKind::Client => "◌",
            SpanKind::Producer => "▶",
            SpanKind::Consumer => "◀",
        }
    }
}

/// Severity level for log records
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl Default for Severity {
    fn default() -> Self {
        Self::Info
    }
}

impl Severity {
    pub fn symbol(&self) -> &'static str {
        match self {
            Severity::Trace => "T",
            Severity::Debug => "D",
            Severity::Info => "I",
            Severity::Warn => "W",
            Severity::Error => "E",
            Severity::Fatal => "F",
        }
    }
}

/// A log record from OpenTelemetry
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub timestamp: SystemTime,
    pub severity: Severity,
    pub body: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub attributes: HashMap<String, String>,
    pub service_name: String,
}

/// A metric data point
#[derive(Debug, Clone)]
pub struct MetricData {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub value: MetricValue,
    pub attributes: HashMap<String, String>,
    pub timestamp: SystemTime,
    pub service_name: String,
}

/// Metric value types
#[derive(Debug, Clone)]
pub enum MetricValue {
    Gauge(f64),
    Counter(u64),
    Histogram { sum: f64, count: u64, buckets: Vec<(f64, u64)> },
}

/// A telemetry event that can be displayed in the UI
#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    SpanStarted(SpanData),
    SpanEnded(SpanData),
    Log(LogRecord),
    Metric(MetricData),
}

impl TelemetryEvent {
    /// Get the timestamp for this event
    pub fn timestamp(&self) -> SystemTime {
        match self {
            TelemetryEvent::SpanStarted(span) => span.start_time,
            TelemetryEvent::SpanEnded(span) => span.end_time.unwrap_or(span.start_time),
            TelemetryEvent::Log(log) => log.timestamp,
            TelemetryEvent::Metric(metric) => metric.timestamp,
        }
    }

    /// Get the service name for this event
    pub fn service_name(&self) -> &str {
        match self {
            TelemetryEvent::SpanStarted(span) | TelemetryEvent::SpanEnded(span) => &span.service_name,
            TelemetryEvent::Log(log) => &log.service_name,
            TelemetryEvent::Metric(metric) => &metric.service_name,
        }
    }

    /// Get a short summary of the event for display
    pub fn summary(&self) -> String {
        match self {
            TelemetryEvent::SpanStarted(span) => format!("▶ {}", span.name),
            TelemetryEvent::SpanEnded(span) => {
                let dur = span.duration().map(|d| format!(" ({:.1}ms)", d.as_secs_f64() * 1000.0)).unwrap_or_default();
                let status = match span.status {
                    SpanStatus::Ok => "✓",
                    SpanStatus::Error => "✗",
                    SpanStatus::Unset => "■",
                };
                format!("{} {}{}", status, span.name, dur)
            }
            TelemetryEvent::Log(log) => {
                let body = if log.body.len() > 40 {
                    format!("{}...", &log.body[..40])
                } else {
                    log.body.clone()
                };
                format!("[{}] {}", log.severity.symbol(), body)
            }
            TelemetryEvent::Metric(metric) => {
                let val = match &metric.value {
                    MetricValue::Gauge(v) => format!("{:.2}", v),
                    MetricValue::Counter(v) => format!("{}", v),
                    MetricValue::Histogram { sum, count, .. } => format!("avg={:.2}", sum / *count as f64),
                };
                format!("📊 {}: {}", metric.name, val)
            }
        }
    }

    /// Priority level for the event (for filtering/sorting)
    pub fn priority(&self) -> u8 {
        match self {
            TelemetryEvent::SpanStarted(_) => 1,
            TelemetryEvent::SpanEnded(span) => match span.status {
                SpanStatus::Error => 3,
                SpanStatus::Ok => 1,
                SpanStatus::Unset => 1,
            },
            TelemetryEvent::Log(log) => match log.severity {
                Severity::Fatal | Severity::Error => 3,
                Severity::Warn => 2,
                _ => 1,
            },
            TelemetryEvent::Metric(_) => 1,
        }
    }
}

/// Trace view - spans organized by trace
#[derive(Debug, Clone, Default)]
pub struct Trace {
    pub trace_id: String,
    pub root_span: Option<String>,
    pub spans: HashMap<String, SpanData>,
}

impl Trace {
    pub fn new(trace_id: String) -> Self {
        Self {
            trace_id,
            root_span: None,
            spans: HashMap::new(),
        }
    }

    pub fn add_span(&mut self, span: SpanData) {
        if span.parent_span_id.is_none() && self.root_span.is_none() {
            self.root_span = Some(span.span_id.clone());
        }
        self.spans.insert(span.span_id.clone(), span);
    }

    pub fn update_span(&mut self, span: SpanData) {
        self.spans.insert(span.span_id.clone(), span);
    }

    /// Get the root span
    pub fn root(&self) -> Option<&SpanData> {
        self.root_span.as_ref().and_then(|id| self.spans.get(id))
    }

    /// Get children of a span
    pub fn children(&self, span_id: &str) -> Vec<&SpanData> {
        self.spans
            .values()
            .filter(|s| s.parent_span_id.as_deref() == Some(span_id))
            .collect()
    }

    /// Duration of the entire trace (root span duration)
    pub fn duration(&self) -> Option<Duration> {
        self.root().and_then(|s| s.duration())
    }

    /// Is the trace still active?
    pub fn is_active(&self) -> bool {
        self.spans.values().any(|s| s.is_active())
    }

    /// Service name from the root span
    pub fn service_name(&self) -> Option<&str> {
        self.root().map(|s| s.service_name.as_str())
    }
}

/// Statistics derived from telemetry data
#[derive(Debug, Clone, Default)]
pub struct TelemetryStats {
    pub total_spans: usize,
    pub active_spans: usize,
    pub error_spans: usize,
    pub total_logs: usize,
    pub error_logs: usize,
    pub warn_logs: usize,
    pub services: Vec<String>,
}
