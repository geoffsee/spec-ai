//! Scan line effect (placeholder)

/// Scan line animation effect
#[derive(Debug, Clone)]
pub struct ScanLineEffect {
    pub speed: f32,
    pub color: crate::renderer::Color,
    pub position: f32,
}

impl Default for ScanLineEffect {
    fn default() -> Self {
        Self {
            speed: 1.0,
            color: crate::renderer::Color::HUD_CYAN,
            position: 0.0,
        }
    }
}

impl ScanLineEffect {
    pub fn update(&mut self, dt: f32) {
        self.position = (self.position + self.speed * dt) % 1.0;
    }
}
