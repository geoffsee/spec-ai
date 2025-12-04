//! Information card widget

use std::time::Duration;
use crate::spatial::{Bounds, Point3D, SpatialAnchor, Transform, Vector3D};
use crate::renderer::{RenderBackend, Color};
use crate::input::OpticalEvent;
use crate::context::{DisplayContext, Priority};
use crate::widget::OpticalWidget;

/// Content types for info cards
#[derive(Debug, Clone)]
pub enum CardContent {
    /// Simple text
    Text(String),
    /// Key-value pairs
    KeyValue(Vec<(String, String)>),
    /// Progress indicator
    Progress { value: f32, max: f32, label: String },
    /// List of items
    List(Vec<String>),
}

/// A section of an info card
#[derive(Debug, Clone)]
pub struct CardSection {
    pub header: Option<String>,
    pub content: CardContent,
    pub priority: Priority,
}

impl CardSection {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            header: None,
            content: CardContent::Text(content.into()),
            priority: Priority::Normal,
        }
    }

    pub fn key_value(pairs: Vec<(String, String)>) -> Self {
        Self {
            header: None,
            content: CardContent::KeyValue(pairs),
            priority: Priority::Normal,
        }
    }

    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }
}

/// Floating information card (mission briefing style)
pub struct InfoCard {
    id: String,
    anchor: SpatialAnchor,
    title: String,
    sections: Vec<CardSection>,
    border_color: Color,
    expanded: bool,
    visibility: f32,
    priority: Priority,
    width: f32,
}

impl InfoCard {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            anchor: SpatialAnchor::head_space(&id_str, Vector3D::new(0.0, 0.0, 2.0)),
            id: id_str,
            title: title.into(),
            sections: Vec::new(),
            border_color: Color::GOLD,
            expanded: true,
            visibility: 1.0,
            priority: Priority::Normal,
            width: 0.4,
        }
    }

    /// Set world-space position
    pub fn world_position(mut self, position: Point3D) -> Self {
        self.anchor = SpatialAnchor::world_space(&self.id, position);
        self
    }

    /// Set screen-space position
    pub fn screen_position(mut self, x: f32, y: f32) -> Self {
        self.anchor = SpatialAnchor::screen_space(&self.id, x, y);
        self
    }

    /// Add a section
    pub fn add_section(mut self, section: CardSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Set border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Toggle expanded state
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }
}

impl OpticalWidget for InfoCard {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        Bounds::sphere(Point3D::ORIGIN, 0.5)
    }

    fn anchor(&self) -> &SpatialAnchor {
        &self.anchor
    }

    fn update(&mut self, _dt: Duration, _ctx: &DisplayContext) {}

    fn handle_event(&mut self, event: &OpticalEvent) -> bool {
        // Could handle expand/collapse gestures here
        false
    }

    fn render(&self, backend: &mut dyn RenderBackend, camera: &Transform) {
        if self.visibility < 0.1 {
            return;
        }

        // Get render position based on anchor type
        let (x, y) = if let Some(coords) = self.anchor.screen_coords() {
            coords
        } else {
            // World/head space - project to screen
            let world_pos = self.anchor.world_position(camera);
            if let Some((sx, sy)) = backend.project(world_pos, camera) {
                ((sx + 1.0) / 2.0, (1.0 - sy) / 2.0)
            } else {
                return; // Not visible
            }
        };

        // Calculate height based on content
        let mut height = 0.04; // Title
        if self.expanded {
            for section in &self.sections {
                height += 0.02; // Section header/spacing
                match &section.content {
                    CardContent::Text(_) => height += 0.03,
                    CardContent::KeyValue(pairs) => height += pairs.len() as f32 * 0.025,
                    CardContent::Progress { .. } => height += 0.03,
                    CardContent::List(items) => height += items.len() as f32 * 0.02,
                }
            }
        }

        // Draw card border
        backend.draw_hud_rect(x, y, self.width, height, self.border_color);

        // Draw title
        backend.draw_hud_text(x + 0.01, y + 0.01, &self.title, Color::GOLD);

        if !self.expanded {
            backend.draw_hud_text(x + self.width - 0.02, y + 0.01, "▶", Color::Grey);
            return;
        }

        backend.draw_hud_text(x + self.width - 0.02, y + 0.01, "▼", Color::Grey);

        // Draw sections
        let mut current_y = y + 0.04;
        for section in &self.sections {
            // Section header
            if let Some(ref header) = section.header {
                backend.draw_hud_text(x + 0.01, current_y, header, Color::HUD_CYAN);
                current_y += 0.02;
            }

            // Section content
            match &section.content {
                CardContent::Text(text) => {
                    backend.draw_hud_text(x + 0.01, current_y, text, Color::White);
                    current_y += 0.03;
                }
                CardContent::KeyValue(pairs) => {
                    for (key, value) in pairs {
                        let line = format!("{}: {}", key, value);
                        backend.draw_hud_text(x + 0.01, current_y, &line, Color::White);
                        current_y += 0.025;
                    }
                }
                CardContent::Progress { value, max, label } => {
                    let pct = value / max;
                    let filled = (pct * 20.0) as usize;
                    let bar = "█".repeat(filled) + &"░".repeat(20 - filled);
                    let text = format!("{} [{}]", label, bar);
                    backend.draw_hud_text(x + 0.01, current_y, &text, Color::STATUS_GREEN);
                    current_y += 0.03;
                }
                CardContent::List(items) => {
                    for item in items {
                        let line = format!("• {}", item);
                        backend.draw_hud_text(x + 0.01, current_y, &line, Color::White);
                        current_y += 0.02;
                    }
                }
            }

            current_y += 0.01; // Spacing between sections
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
