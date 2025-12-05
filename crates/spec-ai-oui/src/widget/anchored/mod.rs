//! World-anchored widgets
//!
//! Widgets that are fixed in 3D space (markers, waypoints, labels).

mod label;
mod marker;
mod waypoint;

pub use label::WorldLabel;
pub use marker::{MarkerCategory, PoiMarker};
pub use waypoint::Waypoint;
