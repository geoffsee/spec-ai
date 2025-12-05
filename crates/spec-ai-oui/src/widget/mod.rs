//! Optical widget system for AR/glasses displays
//!
//! Provides widgets optimized for optical displays:
//! - HUD elements (panels, indicators, compass)
//! - Floating cards and menus
//! - World-anchored markers and waypoints
//! - Visual effects

pub mod anchored;
pub mod effects;
pub mod floating;
pub mod hud;
mod traits;

pub use traits::{OpticalWidget, StatefulOpticalWidget};
