//! Display modes for context-appropriate UI

use super::InformationDensity;

/// Display modes for different contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Idle/standby - minimal HUD
    Idle,
    /// Active exploration - standard HUD
    Exploration,
    /// High alert - enhanced situational awareness
    Alert,
    /// Combat/action - streamlined tactical display
    Combat,
    /// Conversation - focus on dialogue
    Dialogue,
    /// Analysis - detailed information display
    Analysis,
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::Exploration
    }
}

impl DisplayMode {
    /// Get the default information density for this mode
    pub fn default_density(&self) -> InformationDensity {
        match self {
            DisplayMode::Idle => InformationDensity::Minimal,
            DisplayMode::Exploration => InformationDensity::Normal,
            DisplayMode::Alert => InformationDensity::High,
            DisplayMode::Combat => InformationDensity::Low,
            DisplayMode::Dialogue => InformationDensity::Low,
            DisplayMode::Analysis => InformationDensity::Maximum,
        }
    }

    /// Get mode color theme
    pub fn theme_color(&self) -> crate::renderer::Color {
        use crate::renderer::Color;
        match self {
            DisplayMode::Idle => Color::Grey,
            DisplayMode::Exploration => Color::HUD_CYAN,
            DisplayMode::Alert => Color::Yellow,
            DisplayMode::Combat => Color::ALERT_RED,
            DisplayMode::Dialogue => Color::STATUS_GREEN,
            DisplayMode::Analysis => Color::GLASS_TINT,
        }
    }

    /// Get mode icon
    pub fn icon(&self) -> char {
        match self {
            DisplayMode::Idle => '◯',
            DisplayMode::Exploration => '◉',
            DisplayMode::Alert => '⚠',
            DisplayMode::Combat => '⊕',
            DisplayMode::Dialogue => '◈',
            DisplayMode::Analysis => '◇',
        }
    }

    /// Get mode name
    pub fn name(&self) -> &'static str {
        match self {
            DisplayMode::Idle => "IDLE",
            DisplayMode::Exploration => "EXPLORE",
            DisplayMode::Alert => "ALERT",
            DisplayMode::Combat => "COMBAT",
            DisplayMode::Dialogue => "DIALOGUE",
            DisplayMode::Analysis => "ANALYSIS",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        assert_eq!(DisplayMode::default(), DisplayMode::Exploration);
    }

    #[test]
    fn test_mode_density() {
        assert_eq!(DisplayMode::Idle.default_density(), InformationDensity::Minimal);
        assert_eq!(DisplayMode::Analysis.default_density(), InformationDensity::Maximum);
    }
}
