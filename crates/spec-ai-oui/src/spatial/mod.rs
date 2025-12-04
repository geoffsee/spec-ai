//! Spatial primitives for 3D optical UI positioning
//!
//! Uses a right-handed coordinate system:
//! - X: Right (+) / Left (-)
//! - Y: Up (+) / Down (-)
//! - Z: Forward (+) / Backward (-)

mod point3d;
mod vector3d;
mod quaternion;
mod transform;
mod anchor;
mod bounds;

pub use point3d::Point3D;
pub use vector3d::Vector3D;
pub use quaternion::Quaternion;
pub use transform::Transform;
pub use anchor::{SpatialAnchor, AnchorType};
pub use bounds::Bounds;
