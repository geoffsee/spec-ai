//! Color palette definitions

use crate::renderer::Color;

/// A color palette for theming
#[derive(Debug, Clone)]
pub struct Palette {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub foreground: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self::agent_007()
    }
}

impl Palette {
    /// Agent 007 theme (gold, silver, dark blue)
    pub fn agent_007() -> Self {
        Self {
            primary: Color::GOLD,
            secondary: Color::SILVER,
            accent: Color::HUD_CYAN,
            background: Color::Rgb(5, 7, 12),
            foreground: Color::White,
            success: Color::STATUS_GREEN,
            warning: Color::Yellow,
            error: Color::ALERT_RED,
            info: Color::GLASS_TINT,
        }
    }

    /// Tactical theme (green, dark)
    pub fn tactical() -> Self {
        Self {
            primary: Color::STATUS_GREEN,
            secondary: Color::DarkGreen,
            accent: Color::Green,
            background: Color::Rgb(0, 10, 0),
            foreground: Color::STATUS_GREEN,
            success: Color::STATUS_GREEN,
            warning: Color::Yellow,
            error: Color::ALERT_RED,
            info: Color::Green,
        }
    }

    /// Cyberpunk theme (cyan, magenta)
    pub fn cyberpunk() -> Self {
        Self {
            primary: Color::HUD_CYAN,
            secondary: Color::Magenta,
            accent: Color::Rgb(255, 0, 100),
            background: Color::Rgb(10, 0, 20),
            foreground: Color::White,
            success: Color::HUD_CYAN,
            warning: Color::Yellow,
            error: Color::Rgb(255, 0, 50),
            info: Color::Magenta,
        }
    }
}
