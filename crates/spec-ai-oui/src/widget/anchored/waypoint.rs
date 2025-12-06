//! Navigation waypoint widget

use crate::context::{DisplayContext, Priority};
use crate::input::OpticalEvent;
use crate::renderer::{Color, RenderBackend};
use crate::spatial::{Bounds, Point3D, SpatialAnchor, Transform};
use crate::widget::OpticalWidget;
use std::time::Duration;

/// A navigation waypoint with path visualization
pub struct Waypoint {
    id: String,
    anchor: SpatialAnchor,
    label: String,
    path: Option<Vec<Point3D>>,
    eta: Option<Duration>,
    visibility: f32,
    color: Color,
}

impl Waypoint {
    pub fn new(id: impl Into<String>, position: Point3D) -> Self {
        let id_str = id.into();
        Self {
            anchor: SpatialAnchor::world_space(&id_str, position),
            id: id_str,
            label: String::new(),
            path: None,
            eta: None,
            visibility: 1.0,
            color: Color::GOLD,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_path(mut self, path: Vec<Point3D>) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_eta(mut self, eta: Duration) -> Self {
        self.eta = Some(eta);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl OpticalWidget for Waypoint {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        Bounds::sphere(self.anchor.world_position(&Transform::identity()), 2.0)
    }

    fn anchor(&self) -> &SpatialAnchor {
        &self.anchor
    }

    fn update(&mut self, _dt: Duration, _ctx: &DisplayContext) {}

    fn handle_event(&mut self, _event: &OpticalEvent) -> bool {
        false
    }

    fn render(&self, backend: &mut dyn RenderBackend, camera: &Transform) {
        if self.visibility < 0.1 {
            return;
        }

        let world_pos = self.anchor.world_position(camera);

        // Draw path if available
        if let Some(ref path) = self.path {
            let mut prev = camera.position;
            for &point in path {
                backend.draw_line(prev, point, self.color, 0.5, camera);
                prev = point;
            }
            backend.draw_line(prev, world_pos, self.color, 0.5, camera);
        }

        // Draw waypoint marker
        let Some((sx, sy)) = backend.project(world_pos, camera) else {
            return;
        };

        let x = (sx + 1.0) / 2.0;
        let y = (1.0 - sy) / 2.0;

        // Destination marker
        backend.draw_hud_text(x, y, "⬡", self.color);

        // Label
        if !self.label.is_empty() {
            backend.draw_hud_text(x + 0.02, y, &self.label, Color::White);
        }

        // Distance and ETA
        let distance = camera.position.distance(&world_pos);
        let dist_text = if distance >= 1000.0 {
            format!("{:.1}km", distance / 1000.0)
        } else {
            format!("{:.0}m", distance)
        };

        let info_text = if let Some(eta) = self.eta {
            let mins = eta.as_secs() / 60;
            let secs = eta.as_secs() % 60;
            format!("{} | {:02}:{:02}", dist_text, mins, secs)
        } else {
            dist_text
        };

        backend.draw_hud_text(x + 0.02, y + 0.02, &info_text, Color::Grey);
    }

    fn visibility(&self) -> f32 {
        self.visibility
    }

    fn set_visibility(&mut self, visibility: f32) {
        self.visibility = visibility;
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}
