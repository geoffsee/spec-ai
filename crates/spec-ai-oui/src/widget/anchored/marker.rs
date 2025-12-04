//! Point of interest marker

use std::time::Duration;
use crate::spatial::{Bounds, Point3D, SpatialAnchor, Transform};
use crate::renderer::{RenderBackend, Color};
use crate::input::OpticalEvent;
use crate::context::{DisplayContext, Priority};
use crate::widget::OpticalWidget;

/// Marker categories for different POI types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerCategory {
    Objective,
    Threat,
    Friendly,
    Resource,
    Information,
}

impl MarkerCategory {
    pub fn color(&self) -> Color {
        match self {
            MarkerCategory::Objective => Color::GOLD,
            MarkerCategory::Threat => Color::ALERT_RED,
            MarkerCategory::Friendly => Color::STATUS_GREEN,
            MarkerCategory::Resource => Color::Blue,
            MarkerCategory::Information => Color::HUD_CYAN,
        }
    }

    pub fn icon(&self) -> char {
        match self {
            MarkerCategory::Objective => '★',
            MarkerCategory::Threat => '⚠',
            MarkerCategory::Friendly => '●',
            MarkerCategory::Resource => '◆',
            MarkerCategory::Information => 'ℹ',
        }
    }
}

/// A point of interest marker in world space
pub struct PoiMarker {
    id: String,
    anchor: SpatialAnchor,
    label: String,
    category: MarkerCategory,
    show_distance: bool,
    visibility: f32,
}

impl PoiMarker {
    pub fn new(id: impl Into<String>, position: Point3D, category: MarkerCategory) -> Self {
        let id_str = id.into();
        Self {
            anchor: SpatialAnchor::world_space(&id_str, position)
                .with_visibility_distance(100.0)
                .with_fade_distance(80.0),
            id: id_str,
            label: String::new(),
            category,
            show_distance: true,
            visibility: 1.0,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn hide_distance(mut self) -> Self {
        self.show_distance = false;
        self
    }
}

impl OpticalWidget for PoiMarker {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        Bounds::sphere(self.anchor.world_position(&Transform::identity()), 1.0)
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

        // Convert to screen coordinates
        let x = (sx + 1.0) / 2.0;
        let y = (1.0 - sy) / 2.0;

        let color = self.category.color();
        let icon = self.category.icon().to_string();

        // Draw marker icon
        backend.draw_hud_text(x, y, &icon, color);

        // Draw label if present
        if !self.label.is_empty() {
            backend.draw_hud_text(x + 0.02, y, &self.label, Color::White);
        }

        // Draw distance
        if self.show_distance {
            let distance = camera.position.distance(&world_pos);
            let dist_text = if distance >= 1000.0 {
                format!("{:.1}km", distance / 1000.0)
            } else {
                format!("{:.0}m", distance)
            };
            backend.draw_hud_text(x + 0.02, y + 0.02, &dist_text, Color::Grey);
        }
    }

    fn visibility(&self) -> f32 {
        self.visibility
    }

    fn set_visibility(&mut self, visibility: f32) {
        self.visibility = visibility;
    }

    fn priority(&self) -> Priority {
        match self.category {
            MarkerCategory::Objective => Priority::High,
            MarkerCategory::Threat => Priority::Critical,
            MarkerCategory::Friendly => Priority::Normal,
            MarkerCategory::Resource => Priority::Normal,
            MarkerCategory::Information => Priority::Low,
        }
    }
}
