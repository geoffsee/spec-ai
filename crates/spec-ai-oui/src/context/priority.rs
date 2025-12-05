//! Priority levels for content display

use super::InformationDensity;

/// Content priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// Always visible (alerts, critical warnings)
    Critical = 0,
    /// High importance (active objectives)
    High = 1,
    /// Normal importance
    Normal = 2,
    /// Low importance (supplementary info)
    Low = 3,
    /// Optional (only in high-density mode)
    Optional = 4,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Normal
    }
}

impl Priority {
    /// Check if this priority is visible at the given density
    pub fn is_visible_at(&self, density: InformationDensity) -> bool {
        match (self, density) {
            (Priority::Critical, _) => true,
            (
                Priority::High,
                InformationDensity::Low
                | InformationDensity::Normal
                | InformationDensity::High
                | InformationDensity::Maximum,
            ) => true,
            (
                Priority::Normal,
                InformationDensity::Normal | InformationDensity::High | InformationDensity::Maximum,
            ) => true,
            (Priority::Low, InformationDensity::High | InformationDensity::Maximum) => true,
            (Priority::Optional, InformationDensity::Maximum) => true,
            _ => false,
        }
    }

    /// Get minimum density required to display this priority
    pub fn min_density(&self) -> InformationDensity {
        match self {
            Priority::Critical => InformationDensity::Minimal,
            Priority::High => InformationDensity::Low,
            Priority::Normal => InformationDensity::Normal,
            Priority::Low => InformationDensity::High,
            Priority::Optional => InformationDensity::Maximum,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critical_always_visible() {
        assert!(Priority::Critical.is_visible_at(InformationDensity::Minimal));
        assert!(Priority::Critical.is_visible_at(InformationDensity::Maximum));
    }

    #[test]
    fn test_optional_only_at_max() {
        assert!(!Priority::Optional.is_visible_at(InformationDensity::High));
        assert!(Priority::Optional.is_visible_at(InformationDensity::Maximum));
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical < Priority::High);
        assert!(Priority::High < Priority::Normal);
        assert!(Priority::Normal < Priority::Low);
        assert!(Priority::Low < Priority::Optional);
    }
}
