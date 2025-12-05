//! Display modes for the super OUI

use super::InformationDensity;

/// Display modes combining all use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Ambient - minimal HUD, passive monitoring
    Ambient,
    /// Social - conversation assistance, rapport tracking
    Social,
    /// Meeting - calendar, attendees, agenda focus
    Meeting,
    /// Research - maximum info, documents, fact-checking
    Research,
    /// Recording - active capture mode
    Recording,
    /// Navigation - directions, POIs, location context
    Navigation,
    /// Private - secure mode, hides sensitive data
    Private,
    /// Focus - do not disturb, minimal interruptions
    Focus,
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::Ambient
    }
}

impl DisplayMode {
    /// Get the default information density for this mode
    pub fn default_density(&self) -> InformationDensity {
        match self {
            DisplayMode::Ambient => InformationDensity::Minimal,
            DisplayMode::Social => InformationDensity::Normal,
            DisplayMode::Meeting => InformationDensity::Normal,
            DisplayMode::Research => InformationDensity::Maximum,
            DisplayMode::Recording => InformationDensity::Low,
            DisplayMode::Navigation => InformationDensity::Normal,
            DisplayMode::Private => InformationDensity::Low,
            DisplayMode::Focus => InformationDensity::Minimal,
        }
    }

    /// Get mode color theme
    pub fn theme_color(&self) -> crate::renderer::Color {
        use crate::renderer::Color;
        match self {
            DisplayMode::Ambient => Color::Grey,
            DisplayMode::Social => Color::HUD_CYAN,
            DisplayMode::Meeting => Color::STATUS_GREEN,
            DisplayMode::Research => Color::White,
            DisplayMode::Recording => Color::ALERT_RED,
            DisplayMode::Navigation => Color::Yellow,
            DisplayMode::Private => Color::Rgb(128, 0, 128),
            DisplayMode::Focus => Color::Rgb(60, 60, 60),
        }
    }

    /// Get mode icon
    pub fn icon(&self) -> char {
        match self {
            DisplayMode::Ambient => '○',
            DisplayMode::Social => '◉',
            DisplayMode::Meeting => '◎',
            DisplayMode::Research => '◇',
            DisplayMode::Recording => '●',
            DisplayMode::Navigation => '→',
            DisplayMode::Private => '◈',
            DisplayMode::Focus => '◐',
        }
    }

    /// Get mode name
    pub fn name(&self) -> &'static str {
        match self {
            DisplayMode::Ambient => "AMBIENT",
            DisplayMode::Social => "SOCIAL",
            DisplayMode::Meeting => "MEETING",
            DisplayMode::Research => "RESEARCH",
            DisplayMode::Recording => "REC",
            DisplayMode::Navigation => "NAV",
            DisplayMode::Private => "PRIVATE",
            DisplayMode::Focus => "FOCUS",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        assert_eq!(DisplayMode::default(), DisplayMode::Ambient);
    }

    #[test]
    fn test_mode_density() {
        assert_eq!(
            DisplayMode::Ambient.default_density(),
            InformationDensity::Minimal
        );
        assert_eq!(
            DisplayMode::Research.default_density(),
            InformationDensity::Maximum
        );
    }
}
