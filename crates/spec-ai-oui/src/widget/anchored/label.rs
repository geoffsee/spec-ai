//! World-anchored text label

use crate::context::{DisplayContext, Priority};
use crate::input::OpticalEvent;
use crate::renderer::{Color, RenderBackend};
use crate::spatial::{Bounds, Point3D, SpatialAnchor, Transform};
use crate::widget::OpticalWidget;
use std::time::Duration;

/// A text label anchored to a world position
pub struct WorldLabel {
    id: String,
    anchor: SpatialAnchor,
    text: String,
    color: Color,
    visibility: f32,
}

impl WorldLabel {
    pub fn new(id: impl Into<String>, position: Point3D, text: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            anchor: SpatialAnchor::world_space(&id_str, position).with_visibility_distance(50.0),
            id: id_str,
            text: text.into(),
            color: Color::White,
            visibility: 1.0,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl OpticalWidget for WorldLabel {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        Bounds::point(self.anchor.world_position(&Transform::identity()))
    }

    fn anchor(&self) -> &SpatialAnchor {
        &self.anchor
    }

    fn update(&mut self, _dt: Duration, _ctx: &DisplayContext) {}

    fn handle_event(&mut self, _event: &OpticalEvent) -> bool {
        false
    }

    fn render(&self, backend: &mut dyn RenderBackend, camera: &Transform) {
        let anchor_visibility = self.anchor.calculate_visibility(camera);
        let effective_visibility = self.visibility * anchor_visibility;

        if effective_visibility < 0.1 {
            return;
        }

        let world_pos = self.anchor.world_position(camera);
        let Some((sx, sy)) = backend.project(world_pos, camera) else {
            return;
        };

        let x = (sx + 1.0) / 2.0;
        let y = (1.0 - sy) / 2.0;

        backend.draw_hud_text(x, y, &self.text, self.color);
    }

    fn visibility(&self) -> f32 {
        self.visibility
    }

    fn set_visibility(&mut self, visibility: f32) {
        self.visibility = visibility;
    }
}
