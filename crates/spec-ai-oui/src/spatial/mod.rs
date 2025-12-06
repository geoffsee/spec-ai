//! Spatial primitives for 3D optical UI positioning
//!
//! Uses a right-handed coordinate system:
//! - X: Right (+) / Left (-)
//! - Y: Up (+) / Down (-)
//! - Z: Forward (+) / Backward (-)

mod anchor;
mod bounds;
mod point3d;
mod quaternion;
mod transform;
mod vector3d;

pub use anchor::{AnchorType, SpatialAnchor};
pub use bounds::Bounds;
pub use point3d::Point3D;
pub use quaternion::Quaternion;
pub use transform::Transform;
pub use vector3d::Vector3D;
