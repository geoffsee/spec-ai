//! Attention zones for context-aware layout

/// Attention zones based on visual field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionZone {
    /// Center of vision (high detail, small area)
    Foveal,
    /// Parafoveal region (moderate detail)
    ParaFoveal,
    /// Peripheral vision (low detail)
    Peripheral,
    /// Off-screen (audio cues only)
    OffScreen,
}

impl AttentionZone {
    /// Get the angle from center for this zone (in degrees)
    pub fn angle_range(&self) -> (f32, f32) {
        match self {
            AttentionZone::Foveal => (0.0, 5.0),
            AttentionZone::ParaFoveal => (5.0, 15.0),
            AttentionZone::Peripheral => (15.0, 60.0),
            AttentionZone::OffScreen => (60.0, 180.0),
        }
    }

    /// Get visibility multiplier for this zone
    pub fn visibility_multiplier(&self) -> f32 {
        match self {
            AttentionZone::Foveal => 1.0,
            AttentionZone::ParaFoveal => 0.8,
            AttentionZone::Peripheral => 0.5,
            AttentionZone::OffScreen => 0.0,
        }
    }

    /// Determine zone from angle (degrees from center)
    pub fn from_angle(angle: f32) -> Self {
        if angle <= 5.0 {
            AttentionZone::Foveal
        } else if angle <= 15.0 {
            AttentionZone::ParaFoveal
        } else if angle <= 60.0 {
            AttentionZone::Peripheral
        } else {
            AttentionZone::OffScreen
        }
    }
}
