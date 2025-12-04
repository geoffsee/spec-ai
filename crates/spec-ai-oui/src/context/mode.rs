//! Display modes for context-appropriate UI

use super::InformationDensity;

/// Display modes for different real-world contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Standby - minimal HUD, battery saving
    Standby,
    /// Navigate - walking, driving, exploring
    Navigate,
    /// Focus - deep work, do not disturb
    Focus,
    /// Meeting - conversation, presentation mode
    Meeting,
    /// Research - detailed information display
    Research,
    /// Notification - urgent message or alert
    Notification,
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::Navigate
    }
}

impl DisplayMode {
    /// Get the default information density for this mode
    pub fn default_density(&self) -> InformationDensity {
        match self {
            DisplayMode::Standby => InformationDensity::Minimal,
            DisplayMode::Navigate => InformationDensity::Normal,
            DisplayMode::Focus => InformationDensity::Low,
            DisplayMode::Meeting => InformationDensity::Low,
            DisplayMode::Research => InformationDensity::Maximum,
            DisplayMode::Notification => InformationDensity::High,
        }
    }

    /// Get mode color theme
    pub fn theme_color(&self) -> crate::renderer::Color {
        use crate::renderer::Color;
        match self {
            DisplayMode::Standby => Color::Grey,
            DisplayMode::Navigate => Color::HUD_CYAN,
            DisplayMode::Focus => Color::GLASS_TINT,
            DisplayMode::Meeting => Color::STATUS_GREEN,
            DisplayMode::Research => Color::White,
            DisplayMode::Notification => Color::Yellow,
        }
    }

    /// Get mode icon
    pub fn icon(&self) -> char {
        match self {
            DisplayMode::Standby => '○',
            DisplayMode::Navigate => '◎',
            DisplayMode::Focus => '●',
            DisplayMode::Meeting => '◉',
            DisplayMode::Research => '◇',
            DisplayMode::Notification => '◆',
        }
    }

    /// Get mode name
    pub fn name(&self) -> &'static str {
        match self {
            DisplayMode::Standby => "STANDBY",
            DisplayMode::Navigate => "NAVIGATE",
            DisplayMode::Focus => "FOCUS",
            DisplayMode::Meeting => "MEETING",
            DisplayMode::Research => "RESEARCH",
            DisplayMode::Notification => "ALERT",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        assert_eq!(DisplayMode::default(), DisplayMode::Navigate);
    }

    #[test]
    fn test_mode_density() {
        assert_eq!(DisplayMode::Standby.default_density(), InformationDensity::Minimal);
        assert_eq!(DisplayMode::Research.default_density(), InformationDensity::Maximum);
    }
}
