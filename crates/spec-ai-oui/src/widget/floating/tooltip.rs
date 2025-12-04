//! Tooltip widget

use std::time::Duration;
use crate::spatial::{Bounds, Point3D, SpatialAnchor, Transform};
use crate::renderer::{RenderBackend, Color};
use crate::input::OpticalEvent;
use crate::context::{DisplayContext, Priority};
use crate::widget::OpticalWidget;

/// A contextual tooltip
pub struct Tooltip {
    id: String,
    anchor: SpatialAnchor,
    text: String,
    color: Color,
    visibility: f32,
}

impl Tooltip {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            anchor: SpatialAnchor::screen_space(&id_str, 0.5, 0.5),
            id: id_str,
            text: text.into(),
            color: Color::White,
            visibility: 0.0, // Start hidden
        }
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.anchor = SpatialAnchor::screen_space(&self.id, x, y);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn show(&mut self) {
        self.visibility = 1.0;
    }

    pub fn hide(&mut self) {
        self.visibility = 0.0;
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl OpticalWidget for Tooltip {
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

        // Draw tooltip background
        let width = (self.text.len() as f32 * 0.01).max(0.1);
        backend.draw_hud_rect(x - 0.01, y - 0.01, width + 0.02, 0.03, Color::DarkGrey);

        // Draw text
        backend.draw_hud_text(x, y, &self.text, self.color);
    }

    fn visibility(&self) -> f32 {
        self.visibility
    }

    fn set_visibility(&mut self, visibility: f32) {
        self.visibility = visibility;
    }

    fn priority(&self) -> Priority {
        Priority::Low
    }
}
