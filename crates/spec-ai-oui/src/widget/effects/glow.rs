//! Glow effect (placeholder)

/// Glow effect configuration
#[derive(Debug, Clone)]
pub struct GlowEffect {
    pub color: crate::renderer::Color,
    pub intensity: f32,
    pub radius: f32,
}

impl Default for GlowEffect {
    fn default() -> Self {
        Self {
            color: crate::renderer::Color::HUD_CYAN,
            intensity: 1.0,
            radius: 0.1,
        }
    }
}
