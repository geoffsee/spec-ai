//! Render backend trait definition

use crate::spatial::{Point3D, Transform};
use super::surface::{Color, SurfaceCapabilities};

/// Error type for rendering operations
#[derive(Debug, Clone)]
pub enum RenderError {
    /// Backend initialization failed
    InitError(String),
    /// Frame rendering failed
    FrameError(String),
    /// Terminal-specific error
    TerminalError(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::InitError(msg) => write!(f, "Init error: {}", msg),
            RenderError::FrameError(msg) => write!(f, "Frame error: {}", msg),
            RenderError::TerminalError(msg) => write!(f, "Terminal error: {}", msg),
        }
    }
}

impl std::error::Error for RenderError {}

/// A glyph to render at a 3D position
#[derive(Debug, Clone)]
pub struct RenderGlyph {
    /// The symbol/character to render
    pub symbol: String,
    /// Position in 3D space
    pub position: Point3D,
    /// Foreground color
    pub color: Color,
    /// Alpha/opacity (0.0 - 1.0)
    pub alpha: f32,
    /// Scale factor
    pub scale: f32,
}

impl RenderGlyph {
    pub fn new(symbol: impl Into<String>, position: Point3D) -> Self {
        Self {
            symbol: symbol.into(),
            position,
            color: Color::White,
            alpha: 1.0,
            scale: 1.0,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

/// Backend rendering trait for optical UI
pub trait RenderBackend: Send + Sync {
    /// Get surface capabilities
    fn capabilities(&self) -> SurfaceCapabilities;

    /// Begin a new render frame
    fn begin_frame(&mut self) -> Result<(), RenderError>;

    /// End frame and present to display
    fn end_frame(&mut self) -> Result<(), RenderError>;

    /// Clear the render surface with a color
    fn clear(&mut self, color: Color);

    /// Render a 3D positioned glyph
    fn draw_glyph(&mut self, glyph: &RenderGlyph, camera: &Transform);

    /// Render a line between two 3D points
    fn draw_line(&mut self, from: Point3D, to: Point3D, color: Color, alpha: f32, camera: &Transform);

    /// Render a 2D HUD element (screen-space)
    fn draw_hud_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color);

    /// Render text at screen-space position
    fn draw_hud_text(&mut self, x: f32, y: f32, text: &str, color: Color);

    /// Project a 3D point to screen coordinates
    fn project(&self, point: Point3D, camera: &Transform) -> Option<(f32, f32)>;

    /// Check if a 3D point is visible from the camera
    fn is_visible(&self, point: Point3D, camera: &Transform) -> bool;

    /// Get current camera transform
    fn camera(&self) -> &Transform;

    /// Set camera transform
    fn set_camera(&mut self, camera: Transform);
}
