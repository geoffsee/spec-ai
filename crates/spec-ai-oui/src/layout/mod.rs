//! Spatial layout system for optical UI
//!
//! Provides constraint-based layout for 3D/2D positioning.

mod screen_space;
mod spatial;
mod zone;

pub use screen_space::ScreenLayout;
pub use spatial::SpatialConstraint;
pub use zone::AttentionZone;
