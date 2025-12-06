//! Compass widget for navigation

use std::time::Duration;

use crate::context::{DisplayContext, Priority};
use crate::input::OpticalEvent;
use crate::renderer::{Color, RenderBackend};
use crate::spatial::{Bounds, Point3D, SpatialAnchor, Transform};
use crate::widget::OpticalWidget;

/// A waypoint on the compass
#[derive(Debug, Clone)]
pub struct CompassWaypoint {
    /// Waypoint label
    pub label: String,
    /// Bearing in degrees (0 = North, 90 = East)
    pub bearing: f32,
    /// Distance (optional)
    pub distance: Option<f32>,
    /// Icon character
    pub icon: char,
    /// Priority level
    pub priority: Priority,
    /// Color
    pub color: Color,
}

impl CompassWaypoint {
    pub fn new(label: impl Into<String>, bearing: f32) -> Self {
        Self {
            label: label.into(),
            bearing,
            distance: None,
            icon: '◆',
            priority: Priority::Normal,
            color: Color::GOLD,
        }
    }

    pub fn with_distance(mut self, distance: f32) -> Self {
        self.distance = Some(distance);
        self
    }

    pub fn with_icon(mut self, icon: char) -> Self {
        self.icon = icon;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

/// Compass widget showing heading and waypoints
pub struct Compass {
    id: String,
    anchor: SpatialAnchor,
    heading: f32,
    waypoints: Vec<CompassWaypoint>,
    visibility: f32,
    show_cardinal: bool,
}

impl Compass {
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            anchor: SpatialAnchor::screen_space(&id_str, 0.5, 0.02),
            id: id_str,
            heading: 0.0,
            waypoints: Vec::new(),
            visibility: 1.0,
            show_cardinal: true,
        }
    }

    /// Set the compass position
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.anchor = SpatialAnchor::screen_space(&self.id, x, y);
        self
    }

    /// Set current heading
    pub fn set_heading(&mut self, heading: f32) {
        self.heading = heading % 360.0;
    }

    /// Add a waypoint
    pub fn add_waypoint(&mut self, waypoint: CompassWaypoint) {
        self.waypoints.push(waypoint);
    }

    /// Clear all waypoints
    pub fn clear_waypoints(&mut self) {
        self.waypoints.clear();
    }

    /// Calculate the relative bearing of a waypoint
    fn relative_bearing(&self, waypoint_bearing: f32) -> f32 {
        let mut relative = waypoint_bearing - self.heading;
        while relative < -180.0 {
            relative += 360.0;
        }
        while relative > 180.0 {
            relative -= 360.0;
        }
        relative
    }
}

impl OpticalWidget for Compass {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        Bounds::point(Point3D::ORIGIN)
    }

    fn anchor(&self) -> &SpatialAnchor {
        &self.anchor
    }

    fn update(&mut self, _dt: Duration, ctx: &DisplayContext) {
        // Could update heading from context/camera here
    }

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

        // Compass bar width in screen space
        let bar_width = 0.4;
        let bar_start = x - bar_width / 2.0;

        // Draw compass bar background
        backend.draw_hud_rect(bar_start, y, bar_width, 0.04, Color::DarkGrey);

        // Draw cardinal directions
        if self.show_cardinal {
            let cardinals = [("N", 0.0), ("E", 90.0), ("S", 180.0), ("W", 270.0)];
            for (label, bearing) in cardinals {
                let relative = self.relative_bearing(bearing);
                if relative.abs() < 60.0 {
                    let offset = (relative / 60.0) * (bar_width / 2.0);
                    let label_x = x + offset;
                    let color = if label == "N" {
                        Color::ALERT_RED
                    } else {
                        Color::White
                    };
                    backend.draw_hud_text(label_x, y + 0.01, label, color);
                }
            }
        }

        // Draw waypoints
        for waypoint in &self.waypoints {
            let relative = self.relative_bearing(waypoint.bearing);
            if relative.abs() < 60.0 {
                let offset = (relative / 60.0) * (bar_width / 2.0);
                let marker_x = x + offset;
                backend.draw_hud_text(
                    marker_x,
                    y + 0.025,
                    &waypoint.icon.to_string(),
                    waypoint.color,
                );

                // Draw distance if available
                if let Some(dist) = waypoint.distance {
                    let dist_text = if dist >= 1000.0 {
                        format!("{:.1}km", dist / 1000.0)
                    } else {
                        format!("{}m", dist as u32)
                    };
                    backend.draw_hud_text(marker_x, y + 0.04, &dist_text, Color::Grey);
                }
            }
        }

        // Draw heading value
        let heading_text = format!("{:03.0}°", self.heading);
        backend.draw_hud_text(x - 0.02, y - 0.02, &heading_text, Color::HUD_CYAN);
    }

    fn visibility(&self) -> f32 {
        self.visibility
    }

    fn set_visibility(&mut self, visibility: f32) {
        self.visibility = visibility;
    }
}
