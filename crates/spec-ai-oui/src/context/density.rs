//! Information density management

use super::{AttentionState, Priority};

/// Information density levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InformationDensity {
    /// Minimal - only critical info visible
    Minimal = 0,
    /// Low - important elements only
    Low = 1,
    /// Normal - standard display
    Normal = 2,
    /// High - detailed information
    High = 3,
    /// Maximum - all available data
    Maximum = 4,
}

impl Default for InformationDensity {
    fn default() -> Self {
        Self::Normal
    }
}

impl InformationDensity {
    /// Check if a priority is visible at this density
    pub fn is_visible(&self, priority: Priority) -> bool {
        priority.is_visible_at(*self)
    }

    /// Get the visibility threshold for this density
    pub fn visibility_threshold(&self) -> Priority {
        match self {
            InformationDensity::Minimal => Priority::Critical,
            InformationDensity::Low => Priority::High,
            InformationDensity::Normal => Priority::Normal,
            InformationDensity::High => Priority::Low,
            InformationDensity::Maximum => Priority::Optional,
        }
    }
}

/// Manages dynamic information density
#[derive(Debug, Clone)]
pub struct DensityManager {
    /// Current density level
    current: InformationDensity,
    /// Target density level (for smooth transitions)
    target: InformationDensity,
    /// Transition progress (0.0 - 1.0)
    transition_progress: f32,
    /// Auto-adjustment enabled
    auto_adjust: bool,
}

impl Default for DensityManager {
    fn default() -> Self {
        Self {
            current: InformationDensity::Normal,
            target: InformationDensity::Normal,
            transition_progress: 1.0,
            auto_adjust: true,
        }
    }
}

impl DensityManager {
    /// Create a new density manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current density
    pub fn current(&self) -> InformationDensity {
        self.current
    }

    /// Set target density
    pub fn set_density(&mut self, density: InformationDensity) {
        if self.target != density {
            self.target = density;
            self.transition_progress = 0.0;
        }
    }

    /// Enable or disable auto-adjustment
    pub fn set_auto_adjust(&mut self, enabled: bool) {
        self.auto_adjust = enabled;
    }

    /// Update density based on attention state
    pub fn update(&mut self, attention: &AttentionState, dt: f32) {
        // Progress transitions
        if self.transition_progress < 1.0 {
            self.transition_progress = (self.transition_progress + dt * 2.0).min(1.0);
            if self.transition_progress >= 1.0 {
                self.current = self.target;
            }
        }

        // Auto-adjust based on attention
        if self.auto_adjust {
            let new_target = self.calculate_target_density(attention);
            if new_target != self.target {
                self.target = new_target;
                self.transition_progress = 0.0;
            }
        }
    }

    /// Calculate target density based on attention state
    fn calculate_target_density(&self, attention: &AttentionState) -> InformationDensity {
        // High focus = more detail visible
        // Low focus / disengaged = minimal info

        if !attention.is_engaged {
            return InformationDensity::Minimal;
        }

        if attention.focus_level > 0.8 {
            InformationDensity::High
        } else if attention.focus_level > 0.5 {
            InformationDensity::Normal
        } else if attention.focus_level > 0.2 {
            InformationDensity::Low
        } else {
            InformationDensity::Minimal
        }
    }

    /// Check if a priority should be visible
    pub fn should_display(&self, priority: Priority) -> bool {
        self.current.is_visible(priority)
    }

    /// Get visibility factor (for smooth transitions)
    pub fn visibility_factor(&self, priority: Priority) -> f32 {
        let current_visible = self.current.is_visible(priority);
        let target_visible = self.target.is_visible(priority);

        match (current_visible, target_visible) {
            (true, true) => 1.0,
            (false, false) => 0.0,
            (true, false) => 1.0 - self.transition_progress,
            (false, true) => self.transition_progress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_density_ordering() {
        assert!(InformationDensity::Minimal < InformationDensity::Low);
        assert!(InformationDensity::Low < InformationDensity::Normal);
        assert!(InformationDensity::Normal < InformationDensity::High);
        assert!(InformationDensity::High < InformationDensity::Maximum);
    }

    #[test]
    fn test_visibility_at_density() {
        assert!(InformationDensity::Minimal.is_visible(Priority::Critical));
        assert!(!InformationDensity::Minimal.is_visible(Priority::Normal));
        assert!(InformationDensity::Maximum.is_visible(Priority::Optional));
    }

    #[test]
    fn test_density_manager_transition() {
        let mut manager = DensityManager::new();
        manager.set_density(InformationDensity::High);

        assert_eq!(manager.current(), InformationDensity::Normal);

        // Simulate time passing
        for _ in 0..10 {
            manager.update(&AttentionState::default(), 0.1);
        }

        assert_eq!(manager.current(), InformationDensity::High);
    }
}
