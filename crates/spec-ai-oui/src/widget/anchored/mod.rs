//! World-anchored widgets
//!
//! Widgets that are fixed in 3D space (markers, waypoints, labels).

mod marker;
mod waypoint;
mod label;

pub use marker::{PoiMarker, MarkerCategory};
pub use waypoint::Waypoint;
pub use label::WorldLabel;
