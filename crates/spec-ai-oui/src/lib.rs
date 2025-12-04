//! spec-ai-oui: Optical User Interface framework for AR/glasses displays
//!
//! This crate provides a complete optical UI framework with:
//! - 3D spatial coordinate system with anchoring
//! - Abstract renderer backend (terminal simulation + future AR)
//! - Gaze, gesture, head tracking, and voice input abstraction
//! - Context-aware HUD with dynamic information density
//! - Optical widgets optimized for AR/glasses displays
//! - Audio feedback integration

pub mod spatial;
pub mod renderer;
pub mod input;
pub mod widget;
pub mod layout;
pub mod animation;
pub mod context;
pub mod audio;
pub mod app;
pub mod theme;

// Re-export commonly used types
pub use spatial::{Point3D, Vector3D, Quaternion, Transform, Bounds, SpatialAnchor, AnchorType};
pub use renderer::{RenderBackend, SurfaceCapabilities, Color, RenderGlyph};
pub use input::{OpticalEvent, GestureType, GestureEvent};
pub use widget::OpticalWidget;
pub use layout::{SpatialConstraint, AttentionZone};
pub use context::{DisplayContext, DisplayMode, InformationDensity, Priority};
pub use app::OpticalApp;
