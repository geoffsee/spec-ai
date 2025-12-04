//! Context awareness system for optical UI
//!
//! Provides dynamic information density management based on:
//! - User attention and gaze patterns
//! - Activity level and display mode
//! - Priority-based content filtering

mod attention;
mod density;
mod priority;
mod mode;

pub use attention::AttentionState;
pub use density::{DensityManager, InformationDensity};
pub use priority::Priority;
pub use mode::DisplayMode;

use std::time::Duration;

/// Display context passed to widgets during update/render
#[derive(Debug, Clone)]
pub struct DisplayContext {
    /// Current display mode
    pub mode: DisplayMode,
    /// Current attention state
    pub attention: AttentionState,
    /// Current information density
    pub density: InformationDensity,
    /// Time since app start
    pub time: Duration,
    /// Time since last frame
    pub delta_time: Duration,
    /// Current tick count
    pub tick: u64,
}

impl Default for DisplayContext {
    fn default() -> Self {
        Self {
            mode: DisplayMode::Navigate,
            attention: AttentionState::default(),
            density: InformationDensity::Normal,
            time: Duration::ZERO,
            delta_time: Duration::from_millis(16),
            tick: 0,
        }
    }
}

impl DisplayContext {
    /// Create a new display context
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the context for a new frame
    pub fn update(&mut self, dt: Duration) {
        self.delta_time = dt;
        self.time += dt;
        self.tick = self.tick.wrapping_add(1);
    }

    /// Check if a priority level should be displayed at current density
    pub fn should_display(&self, priority: Priority) -> bool {
        priority.is_visible_at(self.density)
    }

    /// Get visibility multiplier for a priority level
    pub fn visibility_for(&self, priority: Priority) -> f32 {
        if self.should_display(priority) {
            1.0
        } else {
            0.0
        }
    }
}
