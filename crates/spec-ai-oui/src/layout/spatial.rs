//! Spatial layout constraints

use crate::spatial::{Bounds, Point3D};

/// Spatial layout constraint
#[derive(Debug, Clone)]
pub enum SpatialConstraint {
    /// Fixed distance from reference point
    Distance { from: Point3D, distance: f32 },
    /// Keep within bounds
    WithinBounds(Bounds),
    /// Minimum separation from other elements
    Separation { min_distance: f32 },
    /// Face toward camera
    FaceCamera,
    /// Stay in front of camera
    StayInView { margin: f32 },
}
