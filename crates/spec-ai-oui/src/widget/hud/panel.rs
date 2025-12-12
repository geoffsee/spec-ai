//! HUD Panel widget

use std::time::Duration;

use crate::context::{DisplayContext, Priority};
use crate::input::OpticalEvent;
use crate::renderer::{Color, RenderBackend};
use crate::spatial::{Bounds, Point3D, SpatialAnchor, Transform};
use crate::widget::OpticalWidget;

/// A floating HUD panel with glass-like appearance
pub struct HudPanel {
    id: String,
    anchor: SpatialAnchor,
    width: f32,
    height: f32,
    title: Option<String>,
    content_lines: Vec<(String, Color)>,
    border_color: Color,
    visibility: f32,
    priority: Priority,
}

impl HudPanel {
    /// Create a new HUD panel
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            anchor: SpatialAnchor::screen_space("panel", 0.5, 0.5),
            width: 0.3,
            height: 0.2,
            title: None,
            content_lines: Vec::new(),
            border_color: Color::HUD_CYAN,
            visibility: 1.0,
            priority: Priority::Normal,
        }
    }

    /// Set the panel's screen-space position
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.anchor = SpatialAnchor::screen_space(&self.id, x, y);
        self
    }

    /// Set the panel size (normalized 0-1)
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the panel title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add a content line
    pub fn add_line(mut self, text: impl Into<String>, color: Color) -> Self {
        self.content_lines.push((text.into(), color));
        self
    }

    /// Set border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Clear all content lines
    pub fn clear_content(&mut self) {
        self.content_lines.clear();
    }

    /// Set content lines
    pub fn set_content(&mut self, lines: Vec<(String, Color)>) {
        self.content_lines = lines;
    }
}

impl OpticalWidget for HudPanel {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        // Screen-space bounds approximated as a point
        Bounds::point(Point3D::new(0.0, 0.0, 1.0))
    }

    fn anchor(&self) -> &SpatialAnchor {
        &self.anchor
    }

    fn update(&mut self, _dt: Duration, _ctx: &DisplayContext) {
        // Panels are mostly static, could add animations here
    }

    fn handle_event(&mut self, _event: &OpticalEvent) -> bool {
        false // Panels don't handle events by default
    }

    fn render(&self, backend: &mut dyn RenderBackend, _camera: &Transform) {
        if self.visibility < 0.1 {
            return;
        }

        if let Some((x, y)) = self.anchor.screen_coords() {
            // Draw panel border
            backend.draw_hud_rect(x, y, self.width, self.height, self.border_color);

            // Draw title if present
            let mut current_y = y + 0.01;
            if let Some(ref title) = self.title {
                let title_x = x + 0.01;
                backend.draw_hud_text(title_x, current_y, title, Color::GOLD);
                current_y += 0.03;
            }

            // Draw content lines
            for (line, color) in &self.content_lines {
                backend.draw_hud_text(x + 0.01, current_y, line, *color);
                current_y += 0.025;
            }
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
