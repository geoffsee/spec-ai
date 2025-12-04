//! Spatial layout system for optical UI
//!
//! Provides constraint-based layout for 3D/2D positioning.

mod spatial;
mod screen_space;
mod zone;

pub use spatial::SpatialConstraint;
pub use zone::AttentionZone;
pub use screen_space::ScreenLayout;
