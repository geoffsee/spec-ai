//! spec-ai-oui: Optical User Interface framework for AR/glasses displays
//!
//! This crate provides a complete optical UI framework with:
//! - 3D spatial coordinate system with anchoring
//! - Abstract renderer backend (terminal simulation + future AR)
//! - Gaze, gesture, head tracking, and voice input abstraction
//! - Context-aware HUD with dynamic information density
//! - Optical widgets optimized for AR/glasses displays
//! - Audio feedback integration

pub mod animation;
pub mod app;
pub mod audio;
pub mod context;
pub mod input;
pub mod layout;
pub mod renderer;
pub mod spatial;
pub mod theme;
pub mod widget;

// Re-export commonly used types
pub use app::OpticalApp;
pub use context::{DisplayContext, DisplayMode, InformationDensity, Priority};
pub use input::{GestureEvent, GestureType, OpticalEvent};
pub use layout::{AttentionZone, SpatialConstraint};
pub use renderer::{Color, RenderBackend, RenderGlyph, SurfaceCapabilities};
pub use spatial::{AnchorType, Bounds, Point3D, Quaternion, SpatialAnchor, Transform, Vector3D};
pub use widget::OpticalWidget;
