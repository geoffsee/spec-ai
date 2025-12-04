//! Status indicator widgets

use std::time::Duration;

use crate::spatial::{Bounds, Point3D, SpatialAnchor, Transform};
use crate::renderer::{RenderBackend, Color};
use crate::input::OpticalEvent;
use crate::context::{DisplayContext, Priority};
use crate::widget::OpticalWidget;

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    pub fn color(&self) -> Color {
        match self {
            AlertSeverity::Info => Color::HUD_CYAN,
            AlertSeverity::Warning => Color::Yellow,
            AlertSeverity::Critical => Color::ALERT_RED,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "ℹ",
            AlertSeverity::Warning => "⚠",
            AlertSeverity::Critical => "⛔",
        }
    }
}

/// Types of status indicators
#[derive(Debug, Clone)]
pub enum IndicatorType {
    /// Circular gauge (0-100%)
    Gauge {
        value: f32,
        max: f32,
        color: Color,
    },
    /// Progress bar
    Bar {
        value: f32,
        max: f32,
        horizontal: bool,
    },
    /// Icon with status
    Icon {
        icon: char,
        active: bool,
    },
    /// Numeric display
    Numeric {
        value: f32,
        label: String,
        precision: u8,
    },
    /// Alert indicator
    Alert {
        message: String,
        severity: AlertSeverity,
    },
}

/// A status indicator widget
pub struct StatusIndicator {
    id: String,
    anchor: SpatialAnchor,
    indicator_type: IndicatorType,
    label: Option<String>,
    visibility: f32,
    priority: Priority,
}

impl StatusIndicator {
    /// Create a new status indicator
    pub fn new(id: impl Into<String>, indicator_type: IndicatorType) -> Self {
        let id_str = id.into();
        Self {
            anchor: SpatialAnchor::screen_space(&id_str, 0.0, 0.0),
            id: id_str,
            indicator_type,
            label: None,
            visibility: 1.0,
            priority: Priority::Normal,
        }
    }

    /// Create a gauge indicator
    pub fn gauge(id: impl Into<String>, value: f32, max: f32, color: Color) -> Self {
        Self::new(id, IndicatorType::Gauge { value, max, color })
    }

    /// Create a bar indicator
    pub fn bar(id: impl Into<String>, value: f32, max: f32, horizontal: bool) -> Self {
        Self::new(id, IndicatorType::Bar { value, max, horizontal })
    }

    /// Create an icon indicator
    pub fn icon(id: impl Into<String>, icon: char, active: bool) -> Self {
        Self::new(id, IndicatorType::Icon { icon, active })
    }

    /// Create a numeric indicator
    pub fn numeric(id: impl Into<String>, value: f32, label: impl Into<String>) -> Self {
        Self::new(id, IndicatorType::Numeric {
            value,
            label: label.into(),
            precision: 0,
        })
    }

    /// Create an alert indicator
    pub fn alert(id: impl Into<String>, message: impl Into<String>, severity: AlertSeverity) -> Self {
        Self::new(id, IndicatorType::Alert {
            message: message.into(),
            severity,
        })
    }

    /// Set screen position
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.anchor = SpatialAnchor::screen_space(&self.id, x, y);
        self
    }

    /// Set label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Update the indicator value
    pub fn set_value(&mut self, value: f32) {
        match &mut self.indicator_type {
            IndicatorType::Gauge { value: v, .. } => *v = value,
            IndicatorType::Bar { value: v, .. } => *v = value,
            IndicatorType::Numeric { value: v, .. } => *v = value,
            _ => {}
        }
    }
}

impl OpticalWidget for StatusIndicator {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        Bounds::point(Point3D::ORIGIN)
    }

    fn anchor(&self) -> &SpatialAnchor {
        &self.anchor
    }

    fn update(&mut self, _dt: Duration, _ctx: &DisplayContext) {}

    fn handle_event(&mut self, _event: &OpticalEvent) -> bool {
        false
    }

    fn render(&self, backend: &mut dyn RenderBackend, _camera: &Transform) {
        if self.visibility < 0.1 {
            return;
        }

        let Some((x, y)) = self.anchor.screen_coords() else {
            return;
        };

        match &self.indicator_type {
            IndicatorType::Gauge { value, max, color } => {
                let pct = (value / max * 100.0) as u8;
                let text = format!("{}%", pct);
                backend.draw_hud_text(x, y, &text, *color);
            }
            IndicatorType::Bar { value, max, horizontal: _ } => {
                let pct = value / max;
                let bar_width = 0.1;
                let filled = (pct * 10.0) as usize;
                let bar: String = "█".repeat(filled) + &"░".repeat(10 - filled);
                backend.draw_hud_text(x, y, &bar, Color::STATUS_GREEN);
            }
            IndicatorType::Icon { icon, active } => {
                let color = if *active { Color::STATUS_GREEN } else { Color::Grey };
                backend.draw_hud_text(x, y, &icon.to_string(), color);
            }
            IndicatorType::Numeric { value, label, precision } => {
                let text = format!("{}: {:.*}", label, *precision as usize, value);
                backend.draw_hud_text(x, y, &text, Color::White);
            }
            IndicatorType::Alert { message, severity } => {
                let text = format!("{} {}", severity.symbol(), message);
                backend.draw_hud_text(x, y, &text, severity.color());
            }
        }

        // Draw label if present
        if let Some(ref label) = self.label {
            backend.draw_hud_text(x, y + 0.02, label, Color::Grey);
        }
    }

    fn visibility(&self) -> f32 {
        self.visibility
    }

    fn set_visibility(&mut self, visibility: f32) {
        self.visibility = visibility;
    }

    fn priority(&self) -> Priority {
        self.priority
    }
}
