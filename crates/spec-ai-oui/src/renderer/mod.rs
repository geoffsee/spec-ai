//! Rendering backend abstraction for optical UI
//!
//! Provides a trait-based abstraction over different rendering backends:
//! - Terminal backend for development/simulation
//! - Future AR device backends

mod backend;
mod surface;
pub mod terminal;

pub use backend::{RenderBackend, RenderError, RenderGlyph};
pub use surface::{Color, SurfaceCapabilities};
