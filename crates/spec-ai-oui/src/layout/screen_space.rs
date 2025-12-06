//! Screen-space layout for HUD elements

/// Screen-space layout positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenLayout {
    /// Top-left corner
    TopLeft,
    /// Top center
    TopCenter,
    /// Top-right corner
    TopRight,
    /// Center left
    CenterLeft,
    /// Center
    Center,
    /// Center right
    CenterRight,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom center
    BottomCenter,
    /// Bottom-right corner
    BottomRight,
}

impl ScreenLayout {
    /// Get normalized screen coordinates (0-1)
    pub fn coords(&self) -> (f32, f32) {
        match self {
            ScreenLayout::TopLeft => (0.02, 0.02),
            ScreenLayout::TopCenter => (0.5, 0.02),
            ScreenLayout::TopRight => (0.98, 0.02),
            ScreenLayout::CenterLeft => (0.02, 0.5),
            ScreenLayout::Center => (0.5, 0.5),
            ScreenLayout::CenterRight => (0.98, 0.5),
            ScreenLayout::BottomLeft => (0.02, 0.98),
            ScreenLayout::BottomCenter => (0.5, 0.98),
            ScreenLayout::BottomRight => (0.98, 0.98),
        }
    }
}
