//! Glass/holographic theme

use super::Palette;

/// Glass/holographic theme settings
#[derive(Debug, Clone)]
pub struct GlassTheme {
    /// Base color palette
    pub palette: Palette,
    /// Border opacity
    pub border_opacity: f32,
    /// Background opacity
    pub background_opacity: f32,
    /// Glow intensity
    pub glow_intensity: f32,
    /// Enable scan lines effect
    pub scan_lines: bool,
}

impl Default for GlassTheme {
    fn default() -> Self {
        Self {
            palette: Palette::agent_007(),
            border_opacity: 0.8,
            background_opacity: 0.3,
            glow_intensity: 0.5,
            scan_lines: false,
        }
    }
}

impl GlassTheme {
    /// High visibility theme
    pub fn high_visibility() -> Self {
        Self {
            palette: Palette::agent_007(),
            border_opacity: 1.0,
            background_opacity: 0.5,
            glow_intensity: 0.8,
            scan_lines: false,
        }
    }

    /// Minimal theme
    pub fn minimal() -> Self {
        Self {
            palette: Palette::agent_007(),
            border_opacity: 0.5,
            background_opacity: 0.1,
            glow_intensity: 0.2,
            scan_lines: false,
        }
    }

    /// Retro CRT theme
    pub fn retro_crt() -> Self {
        Self {
            palette: Palette::tactical(),
            border_opacity: 0.9,
            background_opacity: 0.2,
            glow_intensity: 0.7,
            scan_lines: true,
        }
    }
}
