//! Rendering surface capabilities and color types

/// Rendering surface capabilities
#[derive(Debug, Clone)]
pub struct SurfaceCapabilities {
    /// Width in logical units
    pub width: u32,
    /// Height in logical units
    pub height: u32,
    /// Supports depth/3D rendering
    pub supports_depth: bool,
    /// Supports alpha transparency
    pub supports_alpha: bool,
    /// Horizontal field of view in degrees (for AR)
    pub fov_horizontal: Option<f32>,
    /// Vertical field of view in degrees (for AR)
    pub fov_vertical: Option<f32>,
}

impl Default for SurfaceCapabilities {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            supports_depth: false,
            supports_alpha: false,
            fov_horizontal: None,
            fov_vertical: None,
        }
    }
}

/// Color representation supporting multiple formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Reset to default
    Reset,
    /// Standard ANSI color
    Black,
    DarkGrey,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    White,
    Grey,
    /// 256-color palette
    AnsiValue(u8),
    /// True color RGB
    Rgb(u8, u8, u8),
}

impl Color {
    /// Agent 007 theme colors
    pub const GOLD: Self = Color::Rgb(218, 165, 32);
    pub const SILVER: Self = Color::Rgb(192, 192, 192);
    pub const MISSION_BLUE: Self = Color::Rgb(30, 60, 114);
    pub const ALERT_RED: Self = Color::Rgb(220, 20, 60);
    pub const STATUS_GREEN: Self = Color::Rgb(50, 205, 50);
    pub const HUD_CYAN: Self = Color::Rgb(0, 255, 255);
    pub const GLASS_TINT: Self = Color::Rgb(100, 149, 237);

    /// Convert to crossterm color
    pub fn to_crossterm(&self) -> crossterm::style::Color {
        match self {
            Color::Reset => crossterm::style::Color::Reset,
            Color::Black => crossterm::style::Color::Black,
            Color::DarkGrey => crossterm::style::Color::DarkGrey,
            Color::Red => crossterm::style::Color::Red,
            Color::DarkRed => crossterm::style::Color::DarkRed,
            Color::Green => crossterm::style::Color::Green,
            Color::DarkGreen => crossterm::style::Color::DarkGreen,
            Color::Yellow => crossterm::style::Color::Yellow,
            Color::DarkYellow => crossterm::style::Color::DarkYellow,
            Color::Blue => crossterm::style::Color::Blue,
            Color::DarkBlue => crossterm::style::Color::DarkBlue,
            Color::Magenta => crossterm::style::Color::Magenta,
            Color::DarkMagenta => crossterm::style::Color::DarkMagenta,
            Color::Cyan => crossterm::style::Color::Cyan,
            Color::DarkCyan => crossterm::style::Color::DarkCyan,
            Color::White => crossterm::style::Color::White,
            Color::Grey => crossterm::style::Color::Grey,
            Color::AnsiValue(v) => crossterm::style::Color::AnsiValue(*v),
            Color::Rgb(r, g, b) => crossterm::style::Color::Rgb {
                r: *r,
                g: *g,
                b: *b,
            },
        }
    }

    /// Blend two colors with alpha (0.0 = self, 1.0 = other)
    pub fn blend(&self, other: &Color, alpha: f32) -> Color {
        match (self.to_rgb(), other.to_rgb()) {
            (Some((r1, g1, b1)), Some((r2, g2, b2))) => {
                let r = (r1 as f32 * (1.0 - alpha) + r2 as f32 * alpha) as u8;
                let g = (g1 as f32 * (1.0 - alpha) + g2 as f32 * alpha) as u8;
                let b = (b1 as f32 * (1.0 - alpha) + b2 as f32 * alpha) as u8;
                Color::Rgb(r, g, b)
            }
            _ => {
                if alpha > 0.5 {
                    *other
                } else {
                    *self
                }
            }
        }
    }

    /// Convert to RGB tuple if possible
    pub fn to_rgb(&self) -> Option<(u8, u8, u8)> {
        match self {
            Color::Rgb(r, g, b) => Some((*r, *g, *b)),
            Color::Black => Some((0, 0, 0)),
            Color::White => Some((255, 255, 255)),
            Color::Red => Some((255, 0, 0)),
            Color::Green => Some((0, 255, 0)),
            Color::Blue => Some((0, 0, 255)),
            Color::Yellow => Some((255, 255, 0)),
            Color::Cyan => Some((0, 255, 255)),
            Color::Magenta => Some((255, 0, 255)),
            Color::Grey => Some((128, 128, 128)),
            Color::DarkGrey => Some((64, 64, 64)),
            Color::DarkRed => Some((128, 0, 0)),
            Color::DarkGreen => Some((0, 128, 0)),
            Color::DarkBlue => Some((0, 0, 128)),
            Color::DarkYellow => Some((128, 128, 0)),
            Color::DarkCyan => Some((0, 128, 128)),
            Color::DarkMagenta => Some((128, 0, 128)),
            _ => None,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Reset
    }
}
