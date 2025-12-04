//! Optical widget system for AR/glasses displays
//!
//! Provides widgets optimized for optical displays:
//! - HUD elements (panels, indicators, compass)
//! - Floating cards and menus
//! - World-anchored markers and waypoints
//! - Visual effects

mod traits;
pub mod hud;
pub mod floating;
pub mod anchored;
pub mod effects;

pub use traits::{OpticalWidget, StatefulOpticalWidget};
