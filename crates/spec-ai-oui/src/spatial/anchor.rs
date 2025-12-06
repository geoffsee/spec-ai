//! Spatial anchoring system for optical UI elements

use super::{Point3D, Transform, Vector3D};

/// Types of spatial anchoring for UI elements
#[derive(Debug, Clone, PartialEq)]
pub enum AnchorType {
    /// Fixed in world space (e.g., POI marker attached to a location)
    WorldSpace {
        /// Position in world coordinates
        position: Point3D,
        /// Whether the element should always face the camera
        look_at_camera: bool,
    },

    /// Fixed relative to the user's head/view (e.g., HUD elements)
    HeadSpace {
        /// Offset from head position in head-local coordinates
        offset: Vector3D,
    },

    /// Attached to a tracked object (e.g., label on a recognized item)
    ObjectAttached {
        /// Identifier of the tracked object
        object_id: String,
        /// Offset from the object's transform
        offset: Transform,
    },

    /// Screen space position (2D overlay, always faces camera)
    ScreenSpace {
        /// X position (0.0 = left, 1.0 = right)
        x: f32,
        /// Y position (0.0 = top, 1.0 = bottom)
        y: f32,
    },
}

/// A spatial anchor that defines where a UI element should be positioned
#[derive(Debug, Clone)]
pub struct SpatialAnchor {
    /// Unique identifier for this anchor
    pub id: String,
    /// Type of anchoring
    pub anchor_type: AnchorType,
    /// Maximum distance at which the element is visible (None = always visible)
    pub visibility_distance: Option<f32>,
    /// Distance at which the element starts to fade (for smooth transitions)
    pub fade_distance: Option<f32>,
}

impl SpatialAnchor {
    /// Create a new world-space anchor
    pub fn world_space(id: impl Into<String>, position: Point3D) -> Self {
        Self {
            id: id.into(),
            anchor_type: AnchorType::WorldSpace {
                position,
                look_at_camera: true,
            },
            visibility_distance: None,
            fade_distance: None,
        }
    }

    /// Create a new head-space anchor (HUD element)
    pub fn head_space(id: impl Into<String>, offset: Vector3D) -> Self {
        Self {
            id: id.into(),
            anchor_type: AnchorType::HeadSpace { offset },
            visibility_distance: None,
            fade_distance: None,
        }
    }

    /// Create a new screen-space anchor
    pub fn screen_space(id: impl Into<String>, x: f32, y: f32) -> Self {
        Self {
            id: id.into(),
            anchor_type: AnchorType::ScreenSpace { x, y },
            visibility_distance: None,
            fade_distance: None,
        }
    }

    /// Create a new object-attached anchor
    pub fn object_attached(id: impl Into<String>, object_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            anchor_type: AnchorType::ObjectAttached {
                object_id: object_id.into(),
                offset: Transform::identity(),
            },
            visibility_distance: None,
            fade_distance: None,
        }
    }

    /// Set the visibility distance
    pub fn with_visibility_distance(mut self, distance: f32) -> Self {
        self.visibility_distance = Some(distance);
        self
    }

    /// Set the fade distance
    pub fn with_fade_distance(mut self, distance: f32) -> Self {
        self.fade_distance = Some(distance);
        self
    }

    /// Calculate the world position of this anchor given the camera transform
    pub fn world_position(&self, camera: &Transform) -> Point3D {
        match &self.anchor_type {
            AnchorType::WorldSpace { position, .. } => *position,
            AnchorType::HeadSpace { offset } => camera.transform_point(offset.to_point()),
            AnchorType::ObjectAttached { offset, .. } => {
                // In a real implementation, this would look up the object's transform
                // For now, just use the offset as a world position
                offset.position
            }
            AnchorType::ScreenSpace { .. } => {
                // Screen space anchors don't have a meaningful world position
                // Return camera position as a fallback
                camera.position
            }
        }
    }

    /// Calculate visibility based on distance from camera
    pub fn calculate_visibility(&self, camera: &Transform) -> f32 {
        let distance = self.world_position(camera).distance(&camera.position);

        match (self.visibility_distance, self.fade_distance) {
            (None, _) => 1.0,
            (Some(max_dist), None) => {
                if distance <= max_dist {
                    1.0
                } else {
                    0.0
                }
            }
            (Some(max_dist), Some(fade_dist)) => {
                if distance <= fade_dist {
                    1.0
                } else if distance >= max_dist {
                    0.0
                } else {
                    1.0 - (distance - fade_dist) / (max_dist - fade_dist)
                }
            }
        }
    }

    /// Check if this is a screen-space anchor
    pub fn is_screen_space(&self) -> bool {
        matches!(self.anchor_type, AnchorType::ScreenSpace { .. })
    }

    /// Get screen-space coordinates if this is a screen-space anchor
    pub fn screen_coords(&self) -> Option<(f32, f32)> {
        match self.anchor_type {
            AnchorType::ScreenSpace { x, y } => Some((x, y)),
            _ => None,
        }
    }
}

impl Default for SpatialAnchor {
    fn default() -> Self {
        Self::head_space("default", Vector3D::FORWARD * 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_space_anchor() {
        let anchor = SpatialAnchor::world_space("poi1", Point3D::new(10.0, 0.0, 5.0));
        let camera = Transform::identity();
        let pos = anchor.world_position(&camera);
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 0.0);
        assert_eq!(pos.z, 5.0);
    }

    #[test]
    fn test_head_space_anchor() {
        let anchor = SpatialAnchor::head_space("hud", Vector3D::new(0.0, 0.0, 1.0));
        let mut camera = Transform::identity();
        camera.position = Point3D::new(5.0, 0.0, 0.0);

        let pos = anchor.world_position(&camera);
        assert_eq!(pos.x, 5.0);
        assert_eq!(pos.z, 1.0);
    }

    #[test]
    fn test_visibility_fade() {
        let anchor = SpatialAnchor::world_space("test", Point3D::new(0.0, 0.0, 10.0))
            .with_visibility_distance(20.0)
            .with_fade_distance(10.0);

        let camera = Transform::identity();
        let visibility = anchor.calculate_visibility(&camera);
        assert_eq!(visibility, 1.0); // At edge of fade distance

        let mut camera_far = Transform::identity();
        camera_far.position = Point3D::new(0.0, 0.0, -5.0);
        let visibility_mid = anchor.calculate_visibility(&camera_far);
        assert!(visibility_mid > 0.0 && visibility_mid < 1.0); // In fade zone
    }

    #[test]
    fn test_screen_space() {
        let anchor = SpatialAnchor::screen_space("status", 0.5, 0.1);
        assert!(anchor.is_screen_space());
        assert_eq!(anchor.screen_coords(), Some((0.5, 0.1)));
    }
}
