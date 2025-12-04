//! Tweening/interpolation

use std::time::Duration;
use super::Easing;

/// A tween animation
#[derive(Debug, Clone)]
pub struct Tween {
    /// Start value
    pub start: f32,
    /// End value
    pub end: f32,
    /// Duration
    pub duration: Duration,
    /// Current progress (0-1)
    pub progress: f32,
    /// Easing function
    pub easing: Easing,
    /// Whether the tween is complete
    pub complete: bool,
}

impl Tween {
    /// Create a new tween
    pub fn new(start: f32, end: f32, duration: Duration) -> Self {
        Self {
            start,
            end,
            duration,
            progress: 0.0,
            easing: Easing::Linear,
            complete: false,
        }
    }

    /// Set easing function
    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Update the tween
    pub fn update(&mut self, dt: Duration) {
        if self.complete {
            return;
        }

        self.progress += dt.as_secs_f32() / self.duration.as_secs_f32();
        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.complete = true;
        }
    }

    /// Get current value
    pub fn value(&self) -> f32 {
        let t = self.easing.apply(self.progress);
        self.start + (self.end - self.start) * t
    }

    /// Reset the tween
    pub fn reset(&mut self) {
        self.progress = 0.0;
        self.complete = false;
    }

    /// Reverse the tween
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.start, &mut self.end);
        self.progress = 1.0 - self.progress;
        self.complete = false;
    }
}
